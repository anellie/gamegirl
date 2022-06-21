use crate::{
    gga::{
        addr::*,
        cpu::Cpu,
        dma::Dmas,
        timer::Timers,
        Access::{self, *},
        GameGirlAdv,
    },
    numutil::{hword, word, NumExt, U16Ext, U32Ext},
};
use serde::{Deserialize, Serialize};
use std::{
    mem,
    ops::{Index, IndexMut},
    ptr,
};

pub const KB: usize = 1024;
pub const PAGE_SIZE: usize = 0x8000; // 32KiB
pub const BIOS: &[u8] = include_bytes!("bios.bin");

#[derive(Deserialize, Serialize)]
pub struct Memory {
    #[serde(with = "serde_arrays")]
    pub ewram: [u8; 256 * KB],
    #[serde(with = "serde_arrays")]
    pub iwram: [u8; 32 * KB],
    #[serde(with = "serde_arrays")]
    pub mmio: [u16; KB / 2],

    open_bus: [u8; 4],
    #[serde(skip)]
    #[serde(default = "serde_pages")]
    pages: [*mut u8; 8192],
}

impl GameGirlAdv {
    /// Read a byte from the bus. Also enforces timing.
    pub(super) fn read_byte(&mut self, addr: u32, kind: Access) -> u8 {
        self.add_wait_cycles(self.wait_time::<1>(addr, kind));
        self.get_byte(addr)
    }

    /// Read a half-word from the bus (LE). Also enforces timing.
    /// Also handles unaligned reads, which is why ret is u32.
    pub(super) fn read_hword(&mut self, addr: u32, kind: Access) -> u32 {
        self.add_wait_cycles(self.wait_time::<2>(addr, kind));
        if addr.is_bit(0) {
            // Unaligned
            let val = self.get_hword(addr - 1);
            Cpu::ror_s0(val.u32(), 8)
        } else {
            // Aligned
            self.get_hword(addr).u32()
        }
    }

    /// Read a half-word from the bus (LE). Also enforces timing.
    /// If address is unaligned, do LDRSH behavior.
    pub(super) fn read_hword_ldrsh(&mut self, addr: u32, kind: Access) -> u32 {
        self.add_wait_cycles(self.wait_time::<2>(addr, kind));
        if addr.is_bit(0) {
            // Unaligned
            let val = self.get_byte(addr);
            val as i8 as i16 as u32
        } else {
            // Aligned
            self.get_hword(addr).u32()
        }
    }

    /// Read a word from the bus (LE). Also enforces timing.
    pub(super) fn read_word(&mut self, addr: u32, kind: Access) -> u32 {
        let addr = addr & !3; // Forcibly align
        self.add_wait_cycles(self.wait_time::<4>(addr, kind));
        self.get_word(addr)
    }

    /// Read a word from the bus (LE). Also enforces timing.
    /// If address is unaligned, do LDR/SWP behavior.
    pub(super) fn read_word_ldrswp(&mut self, addr: u32, kind: Access) -> u32 {
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
    pub(super) fn get_byte(&self, addr: u32) -> u8 {
        self.get(addr, |this, addr| match addr {
            0x0400_0000..=0x04FF_FFFF if addr.is_bit(0) => this.get_mmio(addr).high(),
            0x0400_0000..=0x04FF_FFFF => this.get_mmio(addr).low(),
            _ => 0,
        })
    }

    /// Read a half-word from the bus (LE). Does no timing-related things;
    /// simply fetches the value.
    pub(super) fn get_hword(&self, addr: u32) -> u16 {
        self.get(addr, |this, addr| match addr {
            0x0400_0000..=0x04FF_FFFF => this.get_mmio(addr),
            _ => 0,
        })
    }

    /// Read a word from the bus (LE). Does no timing-related things; simply
    /// fetches the value. Also does not handle unaligned reads (yet)
    pub fn get_word(&self, addr: u32) -> u32 {
        self.get(addr, |this, addr| match addr {
            0x0400_0000..=0x04FF_FFFF => {
                word(this.get_mmio(addr), this.get_mmio(addr.wrapping_add(2)))
            }
            _ => 0,
        })
    }

    #[inline]
    fn get<T>(&self, addr: u32, slow: fn(&GameGirlAdv, u32) -> T) -> T {
        let ptr = self.page(addr);
        if !ptr.is_null() {
            unsafe { mem::transmute::<_, *const T>(ptr).read() }
        } else {
            slow(self, addr)
        }
    }

    fn get_mmio(&self, addr: u32) -> u16 {
        let a = addr & 0x3FE;
        match a {
            // Timers
            TM0CNT_L => self.timers.counters[0],
            TM1CNT_L => self.timers.counters[1],
            TM2CNT_L => self.timers.counters[2],
            TM3CNT_L => self.timers.counters[3],

            _ => self[a],
        }
    }

    /// Write a byte to the bus. Handles timing.
    pub(super) fn write_byte(&mut self, addr: u32, value: u8, kind: Access) {
        self.add_wait_cycles(self.wait_time::<1>(addr, kind));
        self.set_byte(addr, value)
    }

    /// Write a half-word from the bus (LE). Handles timing.
    pub(super) fn write_hword(&mut self, addr: u32, value: u16, kind: Access) {
        self.add_wait_cycles(self.wait_time::<2>(addr, kind));
        self.set_hword(addr, value)
    }

    /// Write a word from the bus (LE). Handles timing.
    pub(super) fn write_word(&mut self, addr: u32, value: u32, kind: Access) {
        self.add_wait_cycles(self.wait_time::<4>(addr, kind));
        self.set_word(addr, value)
    }

    /// Write a byte to the bus. Does no timing-related things; simply sets the
    /// value.
    pub(super) fn set_byte(&mut self, addr: u32, value: u8) {
        let a = addr.us();
        match a {
            0x0200_0000..=0x02FF_FFFF => self.memory.ewram[a & 0x3FFFF] = value,
            0x0300_0000..=0x03FF_FFFF => self.memory.iwram[a & 0x7FFF] = value,

            0x0400_0000..=0x04FF_FFFF if addr.is_bit(0) => {
                self.set_hword(addr, self.get_hword(addr).set_high(value))
            }
            0x0400_0000..=0x04FF_FFFF => self.set_hword(addr, self.get_hword(addr).set_low(value)),

            // VRAM weirdness
            0x0500_0000..=0x07FF_FFFF => self.set_hword(addr, hword(value, value)),
            _ => (),
        }
    }

    /// Write a half-word from the bus (LE). Does no timing-related things;
    /// simply sets the value.
    pub(super) fn set_hword(&mut self, addr: u32, value: u16) {
        let addr = addr & !1; // Forcibly align: All write instructions do this
        let a = addr.us();
        match a {
            0x0400_0000..=0x04FF_FFFF => self.set_mmio(addr, value),

            0x0500_0000..=0x05FF_FFFF => {
                self.ppu.palette[a & 0x3FF] = value.low();
                self.ppu.palette[(a & 0x3FF) + 1] = value.high();
            }
            0x0600_0000..=0x0601_7FFF => {
                self.ppu.vram[a & 0x17FFF] = value.low();
                self.ppu.vram[(a & 0x17FFF) + 1] = value.high();
            }
            0x0700_0000..=0x07FF_FFFF => {
                self.ppu.oam[a & 0x3FF] = value.low();
                self.ppu.oam[(a & 0x3FF) + 1] = value.high();
            }

            // VRAM mirror weirdness
            0x0601_8000..=0x0601_FFFF => {
                self.ppu.vram[0x1_0000 + a & 0x7FFF] = value.low();
                self.ppu.vram[(0x1_0000 + a & 0x7FFF) + 1] = value.high();
            }
            0x0602_0000..=0x06FF_FFFF => self.set_hword(addr & 0x0601_FFFF, value),

            _ => {
                self.set_byte(addr, value.low());
                self.set_byte(addr.wrapping_add(1), value.high());
            }
        }
    }

    fn set_mmio(&mut self, addr: u32, value: u16) {
        let a = addr & 0x3FF;
        match a {
            // General
            IME => self[IME] = value & 1,
            IF => self[IF] &= !value,

            // PPU
            DISPSTAT => self[DISPSTAT] = (self[DISPSTAT] & 0b111) | (value & !0b11000111),

            // Timers
            TM0CNT_H => Timers::hi_write(self, 0, value),
            TM1CNT_H => Timers::hi_write(self, 1, value),
            TM2CNT_H => Timers::hi_write(self, 2, value),
            TM3CNT_H => Timers::hi_write(self, 3, value),

            // DMAs
            0xBA => {
                self[a] = value;
                Dmas::update_idx(self, 0, value);
            }
            0xC6 => {
                self[a] = value;
                Dmas::update_idx(self, 1, value);
            }
            0xD2 => {
                self[a] = value;
                Dmas::update_idx(self, 2, value);
            }
            0xDE => {
                self[a] = value;
                Dmas::update_idx(self, 3, value);
            }

            // RO registers
            VCOUNT | KEYINPUT => (),

            _ => self[a] = value,
        }
    }

    /// Write a word from the bus (LE). Does no timing-related things; simply
    /// sets the value.
    pub(super) fn set_word(&mut self, addr: u32, value: u32) {
        let addr = addr & !3; // Forcibly align: All write instructions do this
        self.set_hword(addr, value.low());
        self.set_hword(addr.wrapping_add(2), value.high());
    }

    fn page(&self, addr: u32) -> *const u8 {
        let addr = addr.us();
        unsafe { self.memory.pages[(addr >> 15) & 8191].add(addr & 0x7FFF) }
    }

    const WS_NONSEQ: [u16; 4] = [4, 3, 2, 8];

    fn wait_time<const W: u32>(&self, addr: u32, ty: Access) -> u16 {
        match (addr, W, ty) {
            (0x0200_0000..=0x02FF_FFFF, 4, _) => 6,
            (0x0200_0000..=0x02FF_FFFF, _, _) => 3,
            (0x0500_0000..=0x06FF_FFFF, 4, _) => 2,

            (0x0800_0000..=0x09FF_FFFF, _, Seq) => 2 - self[WAITCNT].bit(4),
            (0x0800_0000..=0x09FF_FFFF, 4, NonSeq) => {
                Self::WS_NONSEQ[self[WAITCNT].bits(2, 2).us()] + (2 - self[WAITCNT].bit(4))
            }
            (0x0800_0000..=0x09FF_FFFF, _, NonSeq) => {
                Self::WS_NONSEQ[self[WAITCNT].bits(2, 2).us()]
            }

            (0x0A00_0000..=0x0BFF_FFFF, _, Seq) => 4 - (self[WAITCNT].bit(7) * 3),
            (0x0A00_0000..=0x0BFF_FFFF, 4, NonSeq) => {
                Self::WS_NONSEQ[self[WAITCNT].bits(5, 2).us()] + (4 - (self[WAITCNT].bit(7) * 3))
            }
            (0x0A00_0000..=0x0BFF_FFFF, _, NonSeq) => {
                Self::WS_NONSEQ[self[WAITCNT].bits(5, 2).us()]
            }

            (0x0C00_0000..=0x0DFF_FFFF, _, Seq) => 8 - (self[WAITCNT].bit(10) * 7),
            (0x0C00_0000..=0x0DFF_FFFF, 4, NonSeq) => {
                Self::WS_NONSEQ[self[WAITCNT].bits(8, 2).us()] + (8 - (self[WAITCNT].bit(10) * 7))
            }
            (0x0C00_0000..=0x0DFF_FFFF, _, NonSeq) => {
                Self::WS_NONSEQ[self[WAITCNT].bits(8, 2).us()]
            }

            _ => 1,
        }
    }

    pub fn init_memory(&mut self) {
        for i in 0..self.memory.pages.len() {
            self.memory.pages[i] = unsafe { self.get_page(i) };
        }
    }

    unsafe fn get_page(&self, page: usize) -> *mut u8 {
        unsafe fn offs(reg: &[u8], offs: usize) -> *mut u8 {
            let ptr = reg.as_ptr() as *mut u8;
            ptr.add(offs & (reg.len() - 1))
        }

        let a = page * PAGE_SIZE;
        match a {
            0x0000_0000..=0x0000_3FFF => offs(BIOS, a),
            0x0200_0000..=0x02FF_FFFF => offs(&self.memory.ewram, a - 0x200_0000),
            0x0300_0000..=0x03FF_FFFF => offs(&self.memory.iwram, a - 0x300_0000),
            0x0500_0000..=0x05FF_FFFF => offs(&self.ppu.palette, a - 0x500_0000),
            0x0600_0000..=0x0601_7FFF => offs(&self.ppu.vram, a - 0x600_0000),
            0x0700_0000..=0x07FF_FFFF => offs(&self.ppu.oam, a - 0x700_0000),
            0x0800_0000..=0x0DFF_FFFF => offs(&self.cart.rom, a - 0x800_0000),
            // VRAM mirror weirdness
            0x0601_8000..=0x0601_FFFF => offs(&self.ppu.vram, 0x1_0000 + (a - 0x600_0000)),
            0x0602_0000..=0x06FF_FFFF => self.get_page(a & 0x0601_FFFF),
            _ => ptr::null::<u8>() as *mut u8,
        }
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            ewram: [0; 256 * KB],
            iwram: [0; 32 * KB],
            mmio: [0; KB / 2],
            open_bus: [0; 4],
            pages: serde_pages(),
        }
    }
}

unsafe impl Send for Memory {}

impl Index<u32> for GameGirlAdv {
    type Output = u16;

    fn index(&self, addr: u32) -> &Self::Output {
        assert!(addr < 0x3FF);
        assert_eq!(addr & 1, 0);
        &self.memory.mmio[(addr >> 1).us()]
    }
}

impl IndexMut<u32> for GameGirlAdv {
    fn index_mut(&mut self, addr: u32) -> &mut Self::Output {
        assert!(addr < 0x3FF);
        assert_eq!(addr & 1, 0);
        &mut self.memory.mmio[(addr >> 1).us()]
    }
}

fn serde_pages() -> [*mut u8; 8192] {
    [ptr::null::<u8>() as *mut u8; 8192]
}
