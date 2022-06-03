use crate::numutil::NumExt;
use crate::system::io::addr::*;
use crate::system::io::apu::Apu;
use crate::system::io::cartridge::Cartridge;
use crate::system::io::dma::{Dma, Hdma};
use crate::system::io::joypad::Joypad;
use crate::system::io::ppu::Ppu;
use crate::system::io::timer::Timer;
use crate::system::{CgbMode, GGOptions, GameGirl};
use serde::{Deserialize, Serialize};
use std::{
    ops::{Index, IndexMut},
    sync::Arc,
};

use super::debugger::Debugger;

pub(super) mod addr;
pub(crate) mod apu;
pub mod cartridge;
mod dma;
pub mod joypad;
mod ppu;
mod timer;

/// The MMU of the GG, containing all IO devices along
/// with big arrays holding internal memory.
///
/// IO registers can be directly read by IO devices by indexing the MMU,
/// the various addresses are defined in the `addr` submodule.
#[derive(Deserialize, Serialize)]
pub struct Mmu {
    #[serde(with = "serde_arrays")]
    vram: [u8; 2 * 8192],
    vram_bank: u8,
    #[serde(with = "serde_arrays")]
    wram: [u8; 4 * 8192],
    wram_bank: u8,
    #[serde(with = "serde_arrays")]
    oam: [u8; 160],
    #[serde(with = "serde_arrays")]
    high: [u8; 256],

    #[serde(skip)]
    #[serde(default)]
    pub(super) bootrom: Option<Vec<u8>>,
    cgb: bool,
    #[serde(skip)]
    #[serde(default)]
    pub(super) debugger: Arc<Debugger>,

    pub cart: Cartridge,
    timer: Timer,
    pub ppu: Ppu,
    joypad: Joypad,
    dma: Dma,
    pub(super) apu: Apu,
    hdma: Hdma,
}

impl Mmu {
    /// Step the system forward by the given amount of cycles.
    /// The given T cycles should already be adjusted for CGB 2x speed mode.
    pub(super) fn step(gg: &mut GameGirl, m_cycles: usize, t_cycles: usize) {
        let t_cpu = m_cycles * 4;
        Hdma::step(gg);
        Timer::step(gg, t_cpu);
        Ppu::step(gg, t_cycles);
        Dma::step(gg, t_cpu);
        Apu::step(&mut gg.mmu, m_cycles);
    }

    pub fn read(&self, addr: u16) -> u8 {
        let a = addr.us();
        match addr {
            0x0000..0x0100 if self.bootrom.is_some() => self.bootrom.as_ref().unwrap()[a],
            0x0200..0x0900 if self.bootrom.is_some() && self.cgb => {
                self.bootrom.as_ref().unwrap()[a - 0x0100]
            }
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.cart.read(addr),

            0x8000..=0x9FFF => self.vram[(a & 0x1FFF) + (self.vram_bank.us() * 0x2000)],
            0xC000..=0xCFFF => self.wram[(a & 0x0FFF)],
            0xD000..=0xDFFF => self.wram[(a & 0x0FFF) + (self.wram_bank.us() * 0x1000)],
            0xE000..=0xFDFF => self.wram[a & 0x1FFF],
            0xFE00..=0xFE9F => self.oam[a & 0xFF],

            _ => self.read_high(addr & 0x00FF),
        }
    }

    pub(super) fn read_signed(&self, addr: u16) -> i8 {
        self.read(addr) as i8
    }

    fn read_high(&self, addr: u16) -> u8 {
        match addr {
            JOYP => self.joypad.read(self.high[JOYP as usize]),
            DIV | TAC => self.timer.read(addr),

            LY if !self[LCDC].is_bit(7) => 0,
            BCPS..=OCPD => self.ppu.read_high(addr),

            NR10..=WAV_END => self.apu.read_register(HIGH_START + addr),
            0x76 if self.cgb => self.apu.read_pcm12(),
            0x77 if self.cgb => self.apu.read_pcm34(),

            HDMA_START if self.cgb => self.hdma.transfer_left as u8,
            HDMA_SRC_HIGH..=HDMA_DEST_LOW => 0xFF,

            _ => self[addr],
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        self.debugger.write_occurred(addr);
        let a = addr.us();
        match addr {
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.cart.write(addr, value),
            0x8000..=0x9FFF => self.vram[(a & 0x1FFF) + (self.vram_bank.us() * 0x2000)] = value,
            0xC000..=0xCFFF => self.wram[(a & 0x0FFF)] = value,
            0xD000..=0xDFFF => self.wram[(a & 0x0FFF) + (self.wram_bank.us() * 0x1000)] = value,
            0xFE00..=0xFE9F => self.oam[a & 0xFF] = value,
            0xFF00..=0xFFFF => self.write_high(addr & 0x00FF, value),
            _ => (),
        }
    }

    fn write_high(&mut self, addr: u16, value: u8) {
        match addr {
            VRAM_SELECT if self.cgb => {
                self.vram_bank = value & 1;
                self[VRAM_SELECT] = value | 0xFE;
            }
            WRAM_SELECT if self.cgb => {
                self.wram_bank = u8::max(1, value & 7);
                self[WRAM_SELECT] = value | 0xF8;
            }

            IF => self[IF] = value | 0xE0,
            IE => self[IE] = value | 0xE0,
            BOOTROM_DISABLE => self.bootrom = None,

            DIV | TAC => self.timer.write(addr, value),
            LCDC => {
                self[LCDC] = value;
                if !value.is_bit(7) {
                    self[STAT] &= 0xF8;
                }
            }
            DMA => {
                self[addr] = value;
                self.dma.start();
            }
            BCPS..=OPRI => self.ppu.write_high(addr, value),
            HDMA_START => Hdma::write_start(self, value),
            KEY1 => self[KEY1] = (value & 1) | self[KEY1] & 0x80,
            NR10..=WAV_END => self.apu.write_register(HIGH_START + addr, value),

            0x01 => self
                .debugger
                .serial_output
                .lock()
                .unwrap()
                .push(value as char),

            LY | SC => (),
            _ => self[addr] = value,
        }
    }

    pub fn read16(&self, addr: u16) -> u16 {
        let low = self.read(addr);
        let high = self.read(addr.wrapping_add(1));
        (high.u16() << 8) | low.u16()
    }

    pub fn write16(&mut self, addr: u16, value: u16) {
        self.write(addr, value.u8());
        self.write(addr.wrapping_add(1), (value >> 8).u8());
    }

    /// Reset the MMU and all IO devices except the cartridge.
    pub(super) fn reset(&mut self) -> Self {
        // TODO the clones are kinda eh
        let mut new = Self::new(self.debugger.clone());
        new.cgb = self.cgb;
        new.load_cart_inner(self.cart.clone());
        new
    }

    pub(super) fn new(debugger: Arc<Debugger>) -> Self {
        let mut mmu = Self {
            vram: [0; 16384],
            vram_bank: 0,
            wram: [0; 32768],
            wram_bank: 1,
            oam: [0; 160],
            high: [0xFF; 256],

            bootrom: None,
            cgb: false,
            debugger,

            timer: Timer::default(),
            ppu: Ppu::new(),
            joypad: Joypad::default(),
            dma: Dma::default(),
            apu: Apu::new(false),
            hdma: Hdma::default(),
            cart: Cartridge::dummy(),
        };
        mmu.init_high();
        mmu
    }

    pub(super) fn load_cart(&mut self, cart: Cartridge, conf: &GGOptions) {
        self.cgb = match conf.mode {
            CgbMode::Always => true,
            CgbMode::Prefer => cart.supports_cgb(),
            CgbMode::Never => cart.requires_cgb(),
        };
        self.load_cart_inner(cart);
    }

    fn load_cart_inner(&mut self, cart: Cartridge) {
        self.bootrom = Some(if self.cgb {
            CGB_BOOTROM.to_vec()
        } else {
            BOOTIX_ROM.to_vec()
        });
        self.ppu.switch_kind(self.cgb);
        self.apu = Apu::new(self.cgb);
        self.cart = cart;
    }

    fn init_high(&mut self) {
        self[LY] = 0;
        self[LCDC] = 0;
        self[STAT] = 0;
        self[SCY] = 0;
        self[SCX] = 0;
        self[WY] = 0;
        self[WX] = 0;
        self[BGP] = 0b1110_0100;
        self[OBP0] = 0b1110_0100;
        self[OBP1] = 0b1110_0100;

        self[SB] = 0;
        self[SC] = 0x7E;
        self[TIMA] = 0;
        self[TMA] = 0;
        self[KEY1] = 0;
    }
}

impl Index<u16> for Mmu {
    type Output = u8;
    fn index(&self, index: u16) -> &Self::Output {
        &self.high[index.us()]
    }
}

impl IndexMut<u16> for Mmu {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.high[index.us()]
    }
}
