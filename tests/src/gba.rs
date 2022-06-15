use std::ops::ControlFlow::{Break, Continue};

use crate::Status;

pub fn exec() {
    crate::run_dir("gba-tests", |gg| {
        let gg = gg.as_gga();
        if gg.cpu.low[0] == 0x04000000 {
            if gg.cpu.low[7] != 0 {
                Break(Status::FailAt(gg.cpu.low[7].to_string()))
            } else if gg.cpu.reg(12) != 0 {
                Break(Status::FailAt(gg.cpu.reg(12).to_string()))
            } else {
                Break(Status::Success)
            }
        } else {
            Continue(())
        }
    })
}
