// Unless otherwise noted, this file is released and thus subject to the
// terms of the Mozilla Public License Version 2.0 (MPL2). Also, it is
// "Incompatible With Secondary Licenses", as defined by the MPL2.
// If a copy of the MPL2 was not distributed with this file, you can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::ptr;

use serde::{Deserialize, Serialize};

use super::{Nds7, Nds9};
use crate::{
    components::{
        arm::interface::RwType,
        memory::{MemoryMappedSystem, MemoryMapper},
    },
    nds::Nds,
    numutil::{hword, word, NumExt, U16Ext, U32Ext},
};

pub const KB: usize = 1024;
pub const MB: usize = KB * KB;
pub const BIOS7: &[u8] = include_bytes!("bios7.bin");
pub const BIOS9: &[u8] = include_bytes!("bios9.bin");

/// Memory struct containing the NDS's memory regions along with page tables
/// and other auxiliary cached information relating to memory.
#[derive(Deserialize, Serialize)]
pub struct Memory {
    #[serde(with = "serde_arrays")]
    psram: [u8; 4 * MB],
    #[serde(with = "serde_arrays")]
    wram: [u8; 32 * KB],
    #[serde(with = "serde_arrays")]
    pub mmio7: [u16; 0x1010 / 2],
    #[serde(with = "serde_arrays")]
    pub mmio9: [u16; 0x520 / 2],

    #[serde(with = "serde_arrays")]
    wram7: [u8; 64 * KB],
    #[serde(with = "serde_arrays")]
    inst_tcm: [u8; 32 * KB],
    #[serde(with = "serde_arrays")]
    data_tcm: [u8; 16 * KB],

    mapper7: MemoryMapper<8192>,
    mapper9: MemoryMapper<8192>,

    wait_word7: [u16; 32],
    wait_other7: [u16; 32],
    wait_word9: [u16; 32],
    wait_other9: [u16; 32],
}

impl Nds {
    /// Initialize page tables and wait times.
    pub fn init_memory(&mut self) {
        MemoryMapper::init_pages(&mut self.nds7());
        MemoryMapper::init_pages(&mut self.nds9());
    }
}

impl Nds7 {
    pub fn get_slow<T: RwType>(&self, addr: u32) -> T {
        match addr {
            0x400_0000..=0x400_1010 if T::WIDTH == 1 && addr.is_bit(0) => {
                T::from_u8(self.get_mmio(addr).high())
            }
            0x400_0000..=0x400_1010 if T::WIDTH == 1 => T::from_u8(self.get_mmio(addr).low()),
            0x400_0000..=0x400_1010 if T::WIDTH == 2 => T::from_u16(self.get_mmio(addr)),
            0x400_0000..=0x400_1010 if T::WIDTH == 4 => {
                T::from_u32(word(self.get_mmio(addr), self.get_mmio(addr + 2)))
            }

            _ => T::from_u8(0),
        }
    }

    pub fn set_slow<T: RwType>(&mut self, addr: u32, value: T) {
        match addr {
            0x400_0000..=0x400_1010 if T::WIDTH == 1 && addr.is_bit(0) => {
                self.set_mmio(addr, hword(self.get_mmio(addr).low(), value.u8()))
            }
            0x400_0000..=0x400_1010 if T::WIDTH == 1 => {
                self.set_mmio(addr, hword(value.u8(), self.get_mmio(addr).high()))
            }
            0x400_0000..=0x400_1010 if T::WIDTH == 2 => self.set_mmio(addr, value.u16()),
            0x400_0000..=0x400_1010 if T::WIDTH == 4 => {
                self.set_mmio(addr, value.u16());
                self.set_mmio(addr, value.u32().high());
            }

            _ => (),
        }
    }

    fn get_mmio(&self, addr: u32) -> u16 {
        let addr = addr & !1;
        self[addr]
    }

    fn set_mmio(&mut self, addr: u32, value: u16) {
        let addr = addr & !1;
        self[addr] = value;
    }
}

impl Nds9 {
    pub fn get_slow<T: RwType>(&self, addr: u32) -> T {
        match addr {
            0x400_0000..=0x400_0520 if T::WIDTH == 1 && addr.is_bit(0) => {
                T::from_u8(self.get_mmio(addr).high())
            }
            0x400_0000..=0x400_0520 if T::WIDTH == 1 => T::from_u8(self.get_mmio(addr).low()),
            0x400_0000..=0x400_0520 if T::WIDTH == 2 => T::from_u16(self.get_mmio(addr)),
            0x400_0000..=0x400_0520 if T::WIDTH == 4 => {
                T::from_u32(word(self.get_mmio(addr), self.get_mmio(addr + 2)))
            }

            0xFFFF_0000..=0xFFFF_FFFF => unsafe {
                let ptr = BIOS9.as_ptr().add(addr.us() % BIOS9.len());
                ptr.cast::<T>().read()
            },

            _ => T::from_u8(0),
        }
    }

    pub fn set_slow<T: RwType>(&mut self, addr: u32, value: T) {
        match addr {
            0x400_0000..=0x400_0520 if T::WIDTH == 1 && addr.is_bit(0) => {
                self.set_mmio(addr, hword(self.get_mmio(addr).low(), value.u8()))
            }
            0x400_0000..=0x400_0520 if T::WIDTH == 1 => {
                self.set_mmio(addr, hword(value.u8(), self.get_mmio(addr).high()))
            }
            0x400_0000..=0x400_0520 if T::WIDTH == 2 => self.set_mmio(addr, value.u16()),
            0x400_0000..=0x400_0520 if T::WIDTH == 4 => {
                self.set_mmio(addr, value.u16());
                self.set_mmio(addr, value.u32().high());
            }

            _ => (),
        }
    }

    fn get_mmio(&self, addr: u32) -> u16 {
        let addr = addr & !1;
        self[addr]
    }

    fn set_mmio(&mut self, addr: u32, value: u16) {
        let addr = addr & !1;
        self[addr] = value;
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            psram: [0; 4 * MB],
            wram: [0; 32 * KB],
            mmio7: [0; 0x1010 / 2],
            mmio9: [0; 0x520 / 2],

            wram7: [0; 64 * KB],
            inst_tcm: [0; 32 * KB],
            data_tcm: [0; 16 * KB],

            mapper7: MemoryMapper::default(),
            mapper9: MemoryMapper::default(),
            wait_word7: [0; 32],
            wait_other7: [0; 32],
            wait_word9: [0; 32],
            wait_other9: [0; 32],
        }
    }
}

unsafe impl Send for Memory {}

impl MemoryMappedSystem<8192> for Nds7 {
    type Usize = u32;
    const ADDR_MASK: &'static [usize] = &[
        0x3FFF, // ARM7 BIOS
        0,      // Unmapped
        0x7FFF, // PSRAM
        0x7FFF, // WRAM/WRAM7
        0,      // MMIO
        0,      // Unmapped
        0x7FFF, // VRAM
        0,      // Unmapped
        0,      // Unmapped
        0,      // Unmapped
        0,      // Unmapped
        0,      // Unmapped
        0,      // Unmapped
        0,      // Unmapped
        0,      // Unmapped
        0,      // Unmapped
    ];
    const PAGE_POW: usize = 15;
    const MASK_POW: usize = 24;

    fn get_mapper(&self) -> &MemoryMapper<8192> {
        &self.memory.mapper7
    }

    fn get_mapper_mut(&mut self) -> &mut MemoryMapper<8192> {
        &mut self.memory.mapper7
    }

    unsafe fn get_page<const R: bool>(&self, a: usize) -> *mut u8 {
        unsafe fn offs(reg: &[u8], offs: usize) -> *mut u8 {
            let ptr = reg.as_ptr() as *mut u8;
            ptr.add(offs % reg.len())
        }

        match a {
            0x0000_0000..=0x00FF_FFFF if R => offs(BIOS7, a),
            0x0200_0000..=0x02FF_FFFF => offs(&self.memory.psram, a - 0x200_0000),
            // TODO not quite right...
            0x0300_0000..=0x037F_FFFF => offs(&self.memory.wram, a - 0x300_0000),
            0x0380_0000..=0x03FF_FFFF => offs(&self.memory.wram7, a - 0x380_0000),
            0x0600_0000..=0x06FF_FFFF if false => todo!(),

            _ => ptr::null::<u8>() as *mut u8,
        }
    }
}

impl MemoryMappedSystem<8192> for Nds9 {
    type Usize = u32;
    const ADDR_MASK: &'static [usize] = &[
        0x7FFF, // Instruction TCM
        0,      // Unmapped
        0x7FFF, // PSRAM
        0x7FFF, // WRAM
        0,      // MMIO
        0x7FFF, // Palette
        0x7FFF, // VRAM
        0x7FFF, // OAM
        0,      // Unmapped
        0,      // Unmapped
        0,      // Unmapped
        0,      // Unmapped
        0,      // Unmapped
        0,      // Unmapped
        0,      // Unmapped
        0,      // Unmapped
    ];
    const PAGE_POW: usize = 15;
    const MASK_POW: usize = 24;

    fn get_mapper(&self) -> &MemoryMapper<8192> {
        &self.memory.mapper9
    }

    fn get_mapper_mut(&mut self) -> &mut MemoryMapper<8192> {
        &mut self.memory.mapper9
    }

    unsafe fn get_page<const R: bool>(&self, a: usize) -> *mut u8 {
        unsafe fn offs(reg: &[u8], offs: usize) -> *mut u8 {
            let ptr = reg.as_ptr() as *mut u8;
            ptr.add(offs % reg.len())
        }

        match a {
            0x0000_0000..=0x01FF_FFFF if R => offs(&self.memory.inst_tcm, a),
            0x0200_0000..=0x02FF_FFFF => offs(&self.memory.psram, a - 0x200_0000),
            0x0300_0000..=0x03FF_FFFF => offs(&self.memory.wram, a - 0x300_0000),

            0x0500_0000..=0x05FF_FFFF if (a & 0x1FFF) < 0x1000 => {
                offs(&self.ppu_nomut::<0>().palette, a - 0x500_0000)
            }
            0x0500_0000..=0x05FF_FFFF => offs(&self.ppu_nomut::<1>().palette, a - 0x501_0000),
            0x0600_0000..=0x061F_FFFF => offs(&self.ppu_nomut::<0>().vram, a - 0x600_0000),
            0x0620_0000..=0x063F_FFFF => offs(&self.ppu_nomut::<1>().vram, a - 0x620_0000),
            // TODO not quite right...
            0x0640_0000..=0x065F_FFFF => offs(&self.ppu_nomut::<0>().vram, a - 0x640_0000),
            0x0660_0000..=0x067F_FFFF => offs(&self.ppu_nomut::<1>().vram, a - 0x660_0000),
            0x0700_0000..=0x07FF_FFFF if (a & 0x1FFF) < 0x1000 => {
                offs(&self.ppu_nomut::<0>().oam, a - 0x700_0000)
            }
            0x0700_0000..=0x07FF_FFFF => offs(&self.ppu_nomut::<1>().oam, a - 0x701_0000),

            0x0600_0000..=0x06FF_FFFF if false => todo!(),

            _ => ptr::null::<u8>() as *mut u8,
        }
    }
}
