use crate::{
    common::{self, EmulateOptions},
    debugger::Debugger,
    gga::{
        audio::{APU_REG_END, APU_REG_START},
        cpu::registers::Flag,
        dma::{Dma, DMA_END, DMA_START, DMA_WIDTH},
        graphics::{
            OAM_END, OAM_START, PALETTE_END, PALETTE_START, PPU_REG_END, PPU_REG_START, VRAM_END,
            VRAM_START,
        },
        input::{Input, INPUT_END, INPUT_START},
        memory::{BIOS_END, BIOS_START, KB, WRAM1_END, WRAM1_START, WRAM2_END, WRAM2_START},
        timer::{Timer, TIMER_END, TIMER_START, TIMER_WIDTH},
    },
    ggc::GGConfig,
    numutil::{hword, word, NumExt, U16Ext, U32Ext},
    Colour,
};
use audio::Apu;
use cartridge::Cartridge;
use cpu::Cpu;
use graphics::Ppu;
use memory::Memory;
use serde::{Deserialize, Serialize};
use std::{
    mem,
    sync::{atomic::Ordering, Arc},
};

mod audio;
mod cartridge;
pub mod cpu;
mod dma;
mod graphics;
mod input;
mod memory;
mod timer;

pub const CPU_CLOCK: f32 = 2u32.pow(24) as f32;

pub type GGADebugger = Debugger<u32>;

/// Console struct representing a GGA. Contains all state and is used for system
/// emulation.
#[derive(Deserialize, Serialize)]
pub struct GameGirlAdv {
    pub cpu: Cpu,
    pub memory: Memory,
    pub ppu: Ppu,
    pub apu: Apu,
    pub dma: [Dma; 4],
    pub timers: [Timer; 4],
    pub input: Input,
    pub cart: Cartridge,

    pub options: EmulateOptions,
    pub config: GGConfig,

    #[serde(skip)]
    #[serde(default)]
    pub debugger: Arc<GGADebugger>,

    /// Temporary for counting cycles in `advance_delta`.
    #[serde(skip)]
    #[serde(default)]
    clock: usize,

    /// Temporary used during instruction execution that counts
    /// the amount of cycles the CPU has to wait until it can
    /// execute the next instruction.
    wait_cycles: usize,
}

impl GameGirlAdv {
    /// Advance the system clock by the given delta in seconds.
    /// Might advance a few clocks more.
    pub fn advance_delta(&mut self, delta: f32) {
        if !self.options.running {
            return;
        }
        self.clock = 0;
        let target = (CPU_CLOCK * delta * self.options.speed_multiplier as f32) as usize;
        while self.clock < target {
            if self.debugger.breakpoint_hit.load(Ordering::Relaxed) {
                self.debugger.breakpoint_hit.store(false, Ordering::Relaxed);
                self.options.running = false;
                break;
            }
            self.advance();
        }
    }

    /// Step until the PPU has finished producing the current frame.
    /// Only used for rewinding since it causes audio desync very easily.
    pub fn produce_frame(&mut self) -> Option<Vec<Colour>> {
        if !self.options.running {
            return None;
        }

        while self.ppu.last_frame == None {
            if self.debugger.breakpoint_hit.load(Ordering::Relaxed) {
                self.debugger.breakpoint_hit.store(false, Ordering::Relaxed);
                self.options.running = false;
                return None;
            }
            self.advance();
        }

        self.ppu.last_frame.take()
    }

    /// Produce the next audio samples and write them to the given buffer.
    /// Writes zeroes if the system is not currently running
    /// and no audio should be played.
    pub fn produce_samples(&mut self, samples: &mut [f32]) {
        // Disable this for now, we don't have a working APU yet.
        // if !self.options.running {
        samples.fill(0.0);
        return;
        // }

        let target = samples.len() * self.options.speed_multiplier;
        while self.apu.buffer.len() < target {
            if self.debugger.breakpoint_hit.load(Ordering::Relaxed) {
                self.debugger.breakpoint_hit.store(false, Ordering::Relaxed);
                self.options.running = false;
                samples.fill(0.0);
                return;
            }
            self.advance();
        }

        let mut buffer = mem::take(&mut self.apu.buffer);
        if self.options.invert_audio_samples {
            // If rewinding, truncate and get rid of any excess samples to prevent
            // audio samples getting backed up
            for (src, dst) in buffer.into_iter().zip(samples.iter_mut().rev()) {
                *dst = src * self.config.volume;
            }
        } else {
            // Otherwise, store any excess samples back in the buffer for next time
            // while again not storing too many to avoid backing up.
            // This way can cause clipping if the console produces audio too fast,
            // however this is preferred to audio falling behind and eating
            // a lot of memory.
            for sample in buffer.drain(target..) {
                self.apu.buffer.push(sample);
            }
            self.apu.buffer.truncate(5_000);

            for (src, dst) in buffer
                .into_iter()
                .step_by(self.options.speed_multiplier)
                .zip(samples.iter_mut())
            {
                *dst = src * self.config.volume;
            }
        }
    }

    /// Step forward the emulated console including all subsystems.
    pub fn advance(&mut self) {
        Cpu::exec_next_inst(self)
    }

    /// Advance everything but the CPU by a clock cycle.
    fn advance_clock(&mut self) {
        self.clock += self.wait_cycles;
        // TODO Advance the system
        self.wait_cycles = 0;
    }

    /// Add wait cycles, which advance the system besides the CPU.
    fn add_wait_cycles(&mut self, count: usize) {
        self.wait_cycles += count;
    }

    /// Read a byte from the bus. Also enforces timing.
    fn read_byte(&mut self, addr: u32, kind: Access) -> u8 {
        self.add_wait_cycles(self.cart.wait_time(addr, kind));
        self.get_byte(addr)
    }

    /// Read a half-word from the bus (LE). Also enforces timing.
    /// Also handles unaligned reads, which is why ret is u32.
    fn read_hword(&mut self, addr: u32, kind: Access) -> u32 {
        self.add_wait_cycles(self.cart.wait_time(addr, kind));
        if addr.is_bit(0) {
            // Unaligned
            let val = self.get_hword(addr - 1);
            Cpu::ror_s0(val.u32(), 8)
        } else {
            // Aligned
            self.get_hword(addr).u32()
        }
    }

    /// Read a word from the bus (LE). Also enforces timing.
    fn read_word(&mut self, addr: u32, kind: Access) -> u32 {
        let addr = addr & !3; // Forcibly align

        self.add_wait_cycles(self.cart.wait_time(addr, kind));
        if (0x0800_0000..=0x0DFF_FFFF).contains(&addr) {
            // Cart bus is 16bit, word reads cause 2 stalls
            self.add_wait_cycles(self.cart.wait_time(addr, Access::Seq));
        }

        self.get_word(addr)
    }

    /// Read a word from the bus (LE). Also enforces timing.
    /// If address is unaligned, do LDR/SWP behavior.
    fn read_word_ldrswp(&mut self, addr: u32, kind: Access) -> u32 {
        let val = self.read_word(addr, kind);
        if addr & 3 != 0 {
            // Unaligned
            let by = (addr & 3) << 3;
            Cpu::ror_s0(val, by)
        } else {
            // Aligned
            val
        }
    }

    /// Read a byte from the bus. Does no timing-related things; simply fetches
    /// the value.
    fn get_byte(&self, addr: u32) -> u8 {
        let addr = addr.us();
        match addr {
            BIOS_START..=BIOS_END => self.memory.bios[addr - BIOS_START],
            WRAM1_START..=WRAM1_END => self.memory.wram1[addr - WRAM1_START],
            WRAM2_START..=WRAM2_END => self.memory.wram2[addr - WRAM2_START],

            0x0800_0000..=0x0DFF_FFFF => {
                self.cart.rom[(self.cart.rom.len() - 1).min(addr & 0x01FF_FFFF)]
            }

            PPU_REG_START..=PPU_REG_END => self.ppu.regs[addr - PPU_REG_START],
            PALETTE_START..=PALETTE_END => self.ppu.palette[addr - PALETTE_START],
            VRAM_START..=VRAM_END => self.ppu.vram[addr - VRAM_START],
            OAM_START..=OAM_END => self.ppu.oam[addr - OAM_START],

            APU_REG_START..=APU_REG_END => self.apu.regs[addr - APU_REG_START],
            DMA_START..=DMA_END => {
                let dma_idx = (addr - DMA_START) / DMA_WIDTH;
                self.dma[dma_idx].regs[addr - DMA_START - (dma_idx * DMA_WIDTH)]
            }
            TIMER_START..=TIMER_END => {
                let timer_idx = (addr - TIMER_START) / TIMER_WIDTH;
                self.timers[timer_idx].regs[addr - TIMER_START - (timer_idx * TIMER_WIDTH)]
            }
            INPUT_START..=INPUT_END => self.input.regs[addr - INPUT_START],

            0x04000200 => self.cpu.ie.low(),
            0x04000201 => self.cpu.ie.high(),
            0x04000202 => self.cpu.if_.low(),
            0x04000203 => self.cpu.if_.high(),
            0x04000204 => self.cart.waitcnt.low(),
            0x04000205 => self.cart.waitcnt.high(),
            0x04000208 => self.cpu.ime as u8,
            0x04000209..=0x0400020B => 0, // High unused bits of IME

            _ => 0xFF,
        }
    }

    /// Read a half-word from the bus (LE). Does no timing-related things;
    /// simply fetches the value.
    fn get_hword(&self, addr: u32) -> u16 {
        hword(self.get_byte(addr), self.get_byte(addr.wrapping_add(1)))
    }

    /// Read a word from the bus (LE). Does no timing-related things; simply
    /// fetches the value. Also does not handle unaligned reads (yet)
    pub fn get_word(&self, addr: u32) -> u32 {
        word(self.get_hword(addr), self.get_hword(addr.wrapping_add(2)))
    }

    /// Write a byte to the bus. Handles timing.
    fn write_byte(&mut self, addr: u32, value: u8, kind: Access) {
        self.add_wait_cycles(self.cart.wait_time(addr, kind));
        self.set_byte(addr, value)
    }

    /// Write a half-word from the bus (LE). Handles timing.
    fn write_hword(&mut self, addr: u32, value: u16, kind: Access) {
        self.add_wait_cycles(self.cart.wait_time(addr, kind));
        self.set_hword(addr, value)
    }

    /// Write a word from the bus (LE). Handles timing.
    fn write_word(&mut self, addr: u32, value: u32, kind: Access) {
        self.add_wait_cycles(self.cart.wait_time(addr, kind));
        if (0x0800_0000..=0x0DFF_FFFF).contains(&addr) {
            // Cart bus is 16bit, word writes cause 2 stalls
            self.add_wait_cycles(self.cart.wait_time(addr, Access::Seq));
        }

        self.set_word(addr, value)
    }

    /// Write a byte to the bus. Does no timing-related things; simply sets the
    /// value.
    fn set_byte(&mut self, addr: u32, value: u8) {
        let addr = addr.us();
        match addr {
            WRAM1_START..=WRAM1_END => self.memory.wram1[addr - WRAM1_START] = value,
            WRAM2_START..=WRAM2_END => self.memory.wram2[addr - WRAM2_START] = value,

            PPU_REG_START..=PPU_REG_END => self.ppu.regs[addr - PPU_REG_START] = value,
            PALETTE_START..=PALETTE_END => self.ppu.palette[addr - PALETTE_START] = value,
            VRAM_START..=VRAM_END => self.ppu.vram[addr - VRAM_START] = value,
            OAM_START..=OAM_END => self.ppu.oam[addr - OAM_START] = value,

            APU_REG_START..=APU_REG_END => self.apu.regs[addr - APU_REG_START] = value,
            DMA_START..=DMA_END => {
                let dma_idx = (addr - DMA_START) / DMA_WIDTH;
                self.dma[dma_idx].regs[addr - DMA_START - (dma_idx * DMA_WIDTH)] = value;
            }
            TIMER_START..=TIMER_END => {
                let timer_idx = (addr - TIMER_START) / TIMER_WIDTH;
                self.timers[timer_idx].regs[addr - TIMER_START - (timer_idx * TIMER_WIDTH)] = value;
            }
            INPUT_START..=INPUT_END => self.input.regs[addr - INPUT_START] = value,

            0x04000201 => self.cpu.ie = self.cpu.ie.set_low(value),
            0x04000202 => self.cpu.if_ = self.cpu.if_.set_high(value),
            0x04000203 => self.cpu.if_ = self.cpu.if_.set_low(value),
            0x04000200 => self.cpu.ie = self.cpu.ie.set_high(value),
            0x04000204 => self.cart.waitcnt = self.cart.waitcnt.set_low(value),
            0x04000205 => self.cart.waitcnt = self.cart.waitcnt.set_high(value),
            0x04000208 => self.cpu.ime = value & 1 != 0,

            _ => (),
        }
    }

    /// Write a half-word from the bus (LE). Does no timing-related things;
    /// simply sets the value.
    fn set_hword(&mut self, addr: u32, value: u16) {
        let addr = addr & !1; // Forcibly align: All write instructions do this
        self.set_byte(addr, value.low());
        self.set_byte(addr.wrapping_add(1), value.high());
    }

    /// Write a word from the bus (LE). Does no timing-related things; simply
    /// sets the value.
    fn set_word(&mut self, addr: u32, value: u32) {
        self.set_hword(addr, value.low());
        self.set_hword(addr.wrapping_add(2), value.high());
    }

    fn reg(&self, idx: u32) -> u32 {
        self.cpu.reg(idx)
    }

    fn low(&self, idx: u16) -> u32 {
        self.cpu.low(idx)
    }

    pub fn get_inst_mnemonic(&self, ptr: u32) -> String {
        if self.cpu.flag(Flag::Thumb) {
            let inst = self.get_hword(ptr);
            Self::get_mnemonic_thumb(inst)
        } else {
            let inst = self.get_word(ptr);
            Self::get_mnemonic_arm(inst)
        }
    }

    /// Create a save state that can be loaded with [load_state].
    pub fn save_state(&self) -> Vec<u8> {
        common::serialize(self, self.config.compress_savestates)
    }

    /// Load a state produced by [save_state].
    /// Will restore the current cartridge and debugger.
    pub fn load_state(&mut self, state: &[u8]) {
        if cfg!(target_arch = "wasm32") {
            // Currently crashes...
            return;
        }

        let old_self = mem::replace(
            self,
            common::deserialize(state, self.config.compress_savestates),
        );
        self.restore_from(old_self);
    }

    /// Restore state after a savestate load. `old_self` should be the
    /// system state before the state was loaded.
    pub fn restore_from(&mut self, old_self: Self) {
        self.cart.rom = old_self.cart.rom;
        self.options = old_self.options;
        self.config = old_self.config;
        self.debugger = old_self.debugger;
    }

    /// Reset the console; like power-cycling a real one.
    pub fn reset(&mut self) {
        let old_self = mem::take(self);
        self.restore_from(old_self);
        Cpu::fix_prefetch(self);
    }
}

impl Default for GameGirlAdv {
    fn default() -> Self {
        let mut gg = Self {
            cpu: Cpu::default(),
            memory: Memory {
                bios: include_bytes!("bios.bin"),
                wram1: [0; 256 * KB],
                wram2: [0; 32 * KB],
            },
            ppu: Ppu {
                regs: [0; 0x56],
                palette: [0; KB],
                vram: [0; 96 * KB],
                oam: [0; KB],
                last_frame: None,
            },
            apu: Apu {
                regs: [0; APU_REG_END - APU_REG_START],
                buffer: Vec::with_capacity(1000),
            },
            // Meh...
            dma: [
                Dma { regs: [0; 10] },
                Dma { regs: [0; 10] },
                Dma { regs: [0; 10] },
                Dma { regs: [0; 10] },
            ],
            timers: [
                Timer { regs: [0; 4] },
                Timer { regs: [0; 4] },
                Timer { regs: [0; 4] },
                Timer { regs: [0; 4] },
            ],
            input: Input { regs: [0; 4] },
            cart: Cartridge::default(),

            options: EmulateOptions::default(),
            config: GGConfig::default(),
            debugger: Arc::new(GGADebugger::default()),
            clock: 0,
            wait_cycles: 0,
        };

        // Skip bootrom for now
        gg.cpu.pc = 0x0800_0000;
        gg.cpu.sp[1] = 0x0300_7F00;
        gg.cpu.sp[3] = 0x0300_7F00;
        gg.cpu.sp[5] = 0x0300_7F00;
        gg
    }
}

/// Enum for the types of memory accesses; either sequential
/// or non-sequential.
#[derive(Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Access {
    Seq,
    NonSeq,
}
