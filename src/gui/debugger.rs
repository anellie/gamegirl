use crate::system::cpu::DReg;
use crate::system::GameGirl;
use eframe::egui::Ui;

pub fn registers(gg: &GameGirl, ui: &mut Ui) {
    for reg in [DReg::AF, DReg::BC, DReg::DE, DReg::HL] {
        ui.label(format!("{:?} = {:04X}", reg, gg.cpu.dreg(reg)));
    }
}
