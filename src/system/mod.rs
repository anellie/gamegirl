use crate::Colour;
use serde::{Deserialize, Serialize};
use std::mem;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::numutil::NumExt;
use crate::system::cpu::{Cpu, Interrupt};
use crate::system::io::addr::{IF, KEY1};
use crate::system::io::cartridge::Cartridge;
use crate::system::io::Mmu;

use self::debugger::Debugger;

pub mod cpu;
pub mod debugger;
pub mod io;

const T_CLOCK_HZ: usize = 4194304;
const M_CLOCK_HZ: f32 = T_CLOCK_HZ as f32 / 4.0;

/// The system and it's state.
/// Represents the entire console.
#[derive(Deserialize, Serialize)]
pub struct GameGirl {
    pub cpu: Cpu,
    pub mmu: Mmu,
    #[serde(skip)]
    #[serde(default)]
    pub debugger: Arc<Debugger>,
    pub config: GGOptions,

    /// Shift of t clocks, which is different in CGB double speed mode. Regular: 2, CGB 2x: 1.
    t_shift: u8,
    /// Temporary for keeping track of how many clocks elapsed in [advance_delta].
    clock: usize,
    /// If the system is running. If false, any calls to [advance_delta] and [produce_samples] do nothing.
    pub running: bool,
    /// If there is a ROM loaded / cartridge inserted.
    pub rom_loaded: bool,
    /// If the audio samples produced by [produce_samples] should be in reversed order.
    /// `true` while rewinding.
    pub invert_audio_samples: bool,
    /// Called when a frame is finished rendering. (End of VBlank)
    #[serde(skip)]
    #[serde(default = "frame_finished")]
    pub frame_finished: Box<dyn Fn(&GameGirl) + Send>,
}

impl GameGirl {
    /// Advance the system clock by the given delta in seconds.
    /// Might advance a few clocks more; especially if a GDMA transfer
    /// occurs at the wrong time.
    pub fn advance_delta(&mut self, delta: f32) {
        if !self.running {
            return;
        }
        self.clock = 0;
        let target = (M_CLOCK_HZ * delta) as usize;
        while self.clock < target {
            if self.debugger.breakpoint_hit.load(Ordering::Relaxed) {
                self.debugger.breakpoint_hit.store(false, Ordering::Relaxed);
                self.running = false;
                break;
            }
            self.advance();
        }
    }

    /// Step until the PPU has finished producing the current frame.
    /// Only used for rewinding since it causes audio desync very easily.
    pub fn produce_frame(&mut self) -> Option<Vec<Colour>> {
        if !self.running {
            return None;
        }

        while self.mmu.ppu.last_frame == None {
            if self.debugger.breakpoint_hit.load(Ordering::Relaxed) {
                self.debugger.breakpoint_hit.store(false, Ordering::Relaxed);
                self.running = false;
                return None;
            }
            self.advance();
        }

        self.mmu.ppu.last_frame.take()
    }

    /// Produce the next audio samples and write them to the given buffer.
    /// Writes zeroes if the system is not currently running
    /// and no audio should be played.
    pub fn produce_samples(&mut self, samples: &mut [f32]) {
        if !self.running {
            samples.fill(0.0);
            return;
        }

        while self.mmu.apu.buffer.len() < samples.len() {
            if self.debugger.breakpoint_hit.load(Ordering::Relaxed) {
                self.debugger.breakpoint_hit.store(false, Ordering::Relaxed);
                self.running = false;
                samples.fill(0.0);
                return;
            }
            self.advance();
        }

        let mut buffer = mem::take(&mut self.mmu.apu.buffer);
        if self.invert_audio_samples {
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
            for sample in buffer.drain(samples.len()..) {
                self.mmu.apu.buffer.push(sample);
            }
            self.mmu.apu.buffer.truncate(10_000);

            for (src, dst) in buffer.into_iter().zip(samples.iter_mut()) {
                *dst = src * self.config.volume;
            }
        }
    }

    /// Advance the system by a single CPU instruction.
    pub fn advance(&mut self) {
        Cpu::exec_next_inst(self)
    }

    /// Advance the MMU, which is everything except the CPU.
    fn advance_clock(&mut self, m_cycles: usize) {
        let t_cycles = m_cycles << self.t_shift;
        Mmu::step(self, m_cycles, t_cycles);
        self.clock += m_cycles
    }

    /// Switch between CGB 2x and normal speed mode.
    fn switch_speed(&mut self) {
        self.t_shift = if self.t_shift == 2 { 1 } else { 2 };
        self.mmu[KEY1] = (self.t_shift & 1) << 7;
        for _ in 0..=1024 {
            self.advance_clock(2)
        }
    }

    fn request_interrupt(&mut self, ir: Interrupt) {
        self.mmu[IF] = self.mmu[IF].set_bit(ir.to_index(), true) as u8;
    }

    fn arg8(&self) -> u8 {
        self.mmu.read(self.cpu.pc + 1)
    }

    fn arg16(&self) -> u16 {
        self.mmu.read16(self.cpu.pc + 1)
    }

    fn pop_stack(&mut self) -> u16 {
        let val = self.mmu.read16(self.cpu.sp);
        self.cpu.sp = self.cpu.sp.wrapping_add(2);
        val
    }

    fn push_stack(&mut self, value: u16) {
        self.cpu.sp = self.cpu.sp.wrapping_sub(2);
        self.mmu.write16(self.cpu.sp, value)
    }

    /// Reset the console, while keeping the current cartridge inserted.
    pub fn reset(&mut self) {
        self.cpu = Cpu::default();
        self.mmu = self.mmu.reset(&self.config);
        self.t_shift = 2;
    }

    /// Create a save state that can be loaded with [load_state].
    /// It is zstd-compressed bincode.
    /// PPU display output and the cartridge are not stored.
    pub fn save_state(&self) -> Vec<u8> {
        if cfg!(target_arch = "wasm32") {
            // Currently crashes when loading...
            return vec![];
        }
        if self.config.compress_savestates {
            let mut dest = vec![];
            let mut writer = zstd::stream::Encoder::new(&mut dest, 3).unwrap();
            bincode::serialize_into(&mut writer, self).unwrap();
            writer.finish().unwrap();
            dest
        } else {
            bincode::serialize(self).unwrap()
        }
    }

    /// Load a state produced by [save_state].
    /// Will restore the current cartridge and debugger.
    pub fn load_state(&mut self, state: &[u8]) {
        if cfg!(target_arch = "wasm32") {
            // Currently crashes...
            return;
        }
        let new_self = if self.config.compress_savestates {
            let decoder = zstd::stream::Decoder::new(state).unwrap();
            bincode::deserialize_from(decoder).unwrap()
        } else {
            bincode::deserialize(state).unwrap()
        };
        let old_self = mem::replace(self, new_self);
        self.debugger = old_self.debugger;
        self.mmu.cart.rom = old_self.mmu.cart.rom;
        self.frame_finished = old_self.frame_finished;
        self.mmu.bootrom = old_self.mmu.bootrom;
    }

    /// Create a new console with no cartridge loaded.
    pub fn new() -> Self {
        let debugger = Arc::new(Debugger::default());
        Self {
            cpu: Cpu::default(),
            mmu: Mmu::new(debugger.clone()),
            debugger,
            config: GGOptions::default(),

            t_shift: 2,
            clock: 0,
            running: false,
            rom_loaded: false,
            invert_audio_samples: false,
            frame_finished: Box::new(|_| ()),
        }
    }

    /// Load the given cartridge.
    /// `reset` indicates if the system should be reset before loading.
    pub fn load_cart(&mut self, cart: Cartridge, config: &GGOptions, reset: bool) {
        if reset {
            let old_self = mem::replace(self, Self::new());
            self.debugger = old_self.debugger.clone();
            self.mmu.debugger = old_self.debugger;
            self.frame_finished = old_self.frame_finished;
        }
        self.mmu.load_cart(cart, config);
        self.config = config.clone();
        self.running = true;
        self.rom_loaded = true;
    }

    /// Create a system with a cart already loaded.
    pub fn with_cart(rom: Vec<u8>) -> Self {
        let mut gg = Self::new();
        gg.load_cart(Cartridge::from_rom(rom), &GGOptions::default(), false);
        gg
    }
}

/// Configuration used when initializing the system.
#[derive(Clone, Serialize, Deserialize)]
pub struct GGOptions {
    /// How to handle CGB mode.
    pub mode: CgbMode,
    /// If save states should be compressed.
    pub compress_savestates: bool,
    /// If CGB colours should be corrected.
    pub cgb_colour_correction: bool,
    /// Audio volume multiplier
    pub volume: f32,
}

impl Default for GGOptions {
    fn default() -> Self {
        Self {
            mode: CgbMode::Prefer,
            compress_savestates: false,
            cgb_colour_correction: false,
            volume: 0.5,
        }
    }
}

/// How to handle CGB mode depending on cart compatibility.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CgbMode {
    /// Always run in CGB mode, even when the cart does not support it.
    /// If it does not, it is run in DMG compatibility mode, just like on a
    /// real CGB.
    Always,
    /// If the cart has CGB support, run it as CGB; if not, don't.
    Prefer,
    /// Never run the cart in CGB mode unless it requires it.
    Never,
}

fn frame_finished() -> Box<dyn Fn(&GameGirl) + Send> {
    Box::new(|_| ())
}
