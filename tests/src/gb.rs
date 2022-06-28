use core::ggc::cpu::{DReg::*, Reg::A};
use std::ops::ControlFlow::{Break, Continue};

use crate::Status;

pub fn blargg() {
    crate::run_dir::<true, true>("blargg", |gg| {
        let gg = gg.as_ggc();
        let serial = &gg.debugger.serial_output;
        if serial.contains("Failed") {
            Break(Status::FailAt(serial.lines().last().unwrap().to_string()))
        } else {
            Continue(())
        }
    })
}

pub fn blargg_sound() {
    crate::run_dir::<true, true>("blargg_sound", |gg| {
        let gg = gg.as_ggc();
        if gg.get8(0xA000) == 0 {
            Break(Status::Success)
        } else {
            Continue(())
        }
    })
}

pub fn mooneye(subdir: &str) {
    crate::run_dir::<false, false>(&format!("mooneye/{subdir}"), |gg| {
        let gg = gg.as_ggc();
        if gg.cpu.reg(A) == 0
            && gg.cpu.dreg(BC) == 0x0305
            && gg.cpu.dreg(DE) == 0x080D
            && gg.cpu.dreg(HL) == 0x1522
        {
            Break(Status::Success)
        } else {
            Continue(())
        }
    })
}

pub fn acid2() {
    crate::run_dir::<true, true>("acid2", |_| Continue(()));
}
