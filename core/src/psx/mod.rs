// Unless otherwise noted, this file is released and thus subject to the
// terms of the Mozilla Public License Version 2.0 (MPL2). Also, it is
// "Incompatible With Secondary Licenses", as defined by the MPL2.
// If a copy of the MPL2 was not distributed with this file, you can
// obtain one at https://mozilla.org/MPL/2.0/.

#![allow(unused)]
#![allow(clippy::unused_self)]

use std::mem;

use serde::{Deserialize, Serialize};

use crate::{
    common,
    common::{EmulateOptions, SystemConfig},
    components::{debugger::Debugger, scheduler::Scheduler},
    psx::{apu::Apu, cpu::Cpu, gpu::Gpu, memory::Memory, scheduling::PsxEvent},
    Colour,
};

mod apu;
mod cpu;
mod gpu;
mod memory;
mod scheduling;

pub type PsxDebugger = Debugger<u32>;

/// System state representing entire console.
#[derive(Default, Deserialize, Serialize)]
pub struct PlayStation {
    cpu: Cpu,
    ppu: Gpu,
    apu: Apu,
    memory: Memory,

    #[serde(skip)]
    #[serde(default)]
    pub debugger: PsxDebugger,
    scheduler: Scheduler<PsxEvent>,

    pub options: EmulateOptions,
    pub config: SystemConfig,
    ticking: bool,
}

impl PlayStation {
    common_functions!(todo!(), todo!());

    /// Advance the system by a single CPU instruction.
    pub fn advance(&mut self) {
        Cpu::execute_next(self);
    }

    /// Advance the scheduler, which controls everything except the CPU.
    fn advance_clock(&mut self, cycles: u32) {
        self.scheduler.advance(cycles);
        while let Some(event) = self.scheduler.get_next_pending() {
            event.kind.dispatch(self, event.late_by);
        }
    }

    /// Restore state after a savestate load. `old_self` should be the
    /// system state before the state was loaded.
    pub fn restore_from(&mut self, old_self: Self) {
        self.options = old_self.options;
        self.config = old_self.config;
        self.debugger = old_self.debugger;
    }

    pub fn skip_bootrom(&mut self) {
        todo!()
    }
}
