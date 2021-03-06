// Unless otherwise noted, this file is released and thus subject to the
// terms of the Mozilla Public License Version 2.0 (MPL2). Also, it is
// "Incompatible With Secondary Licenses", as defined by the MPL2.
// If a copy of the MPL2 was not distributed with this file, you can
// obtain one at https://mozilla.org/MPL/2.0/.

use crate::{
    components::{
        arm::{
            interface::{ArmSystem, RwType},
            Access, Cpu, Exception,
        },
        memory::MemoryMapper,
    },
    nds::{Nds, Nds7, Nds9},
    numutil::NumExt,
};

pub const NDS9_CLOCK: u32 = 67_027_964;

impl ArmSystem for Nds7 {
    const IS_V5: bool = false;
    const IE_ADDR: u32 = 0;
    const IF_ADDR: u32 = 0;
    const IME_ADDR: u32 = 0;

    fn cpur(&self) -> &Cpu<Self> {
        &self.cpu7
    }

    fn cpu(&mut self) -> &mut Cpu<Self> {
        &mut self.cpu7
    }

    fn advance_clock(&mut self) {}

    fn add_sn_cycles(&mut self, cycles: u16) {
        self.time_7 += cycles.u32() << 1;
    }

    fn add_i_cycles(&mut self, cycles: u16) {
        self.time_7 += cycles.u32() << 1;
    }

    fn exception_happened(&mut self, _kind: Exception) {}

    fn pipeline_stalled(&mut self) {}

    fn get<T: RwType>(&mut self, addr: u32) -> T {
        MemoryMapper::get(self, addr, T::WIDTH - 1, Self::get_slow)
    }

    fn set<T: RwType>(&mut self, addr: u32, value: T) {
        MemoryMapper::set(self, addr, value, Self::set_slow);
    }

    fn wait_time<T: RwType>(&mut self, _addr: u32, _access: Access) -> u16 {
        1
    }

    fn check_debugger(&mut self) -> bool {
        true
    }

    fn can_cache_at(_addr: u32) -> bool {
        false
    }
}

impl ArmSystem for Nds9 {
    const IS_V5: bool = true;
    const IE_ADDR: u32 = 0;
    const IF_ADDR: u32 = 0;
    const IME_ADDR: u32 = 0;

    fn cpur(&self) -> &Cpu<Self> {
        &self.cpu9
    }

    fn cpu(&mut self) -> &mut Cpu<Self> {
        &mut self.cpu9
    }

    fn advance_clock(&mut self) {
        Nds::advance_clock(self);
    }

    fn add_sn_cycles(&mut self, cycles: u16) {
        self.scheduler.advance(cycles.u32());
    }

    fn add_i_cycles(&mut self, cycles: u16) {
        self.scheduler.advance(cycles.u32());
    }

    fn exception_happened(&mut self, _kind: Exception) {}

    fn pipeline_stalled(&mut self) {}

    fn get<T: RwType>(&mut self, addr: u32) -> T {
        MemoryMapper::get(self, addr, T::WIDTH - 1, Self::get_slow)
    }

    fn set<T: RwType>(&mut self, addr: u32, value: T) {
        MemoryMapper::set(self, addr, value, Self::set_slow);
    }

    fn wait_time<T: RwType>(&mut self, _addr: u32, _access: Access) -> u16 {
        1
    }

    fn check_debugger(&mut self) -> bool {
        true
    }

    fn can_cache_at(_addr: u32) -> bool {
        false
    }
}
