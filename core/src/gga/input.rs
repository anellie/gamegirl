// Unless otherwise noted, this file is released and thus subject to the
// terms of the Mozilla Public License Version 2.0 (MPL2). Also, it is
// "Incompatible With Secondary Licenses", as defined by the MPL2.
// If a copy of the MPL2 was not distributed with this file, you can
// obtain one at https://mozilla.org/MPL/2.0/.

//! Input handler.
//! Luckily, GGA input is dead simple compared to even GG.

use crate::{
    common::Button,
    components::arm::{Cpu, Interrupt},
    gga::{
        addr::{KEYCNT, KEYINPUT},
        GameGirlAdv,
    },
    numutil::NumExt,
};

impl GameGirlAdv {
    pub fn set_button(&mut self, btn: Button, state: bool) {
        self[KEYINPUT] = self[KEYINPUT].set_bit(btn as u16, !state);
        self.check_keycnt();
    }

    pub fn check_keycnt(&mut self) {
        let input = self[KEYINPUT];
        let cnt = self[KEYCNT];
        if cnt.is_bit(14) {
            let cond = cnt.bits(0, 10);
            let fire = if !cnt.is_bit(15) {
                cond & input != 0
            } else {
                cond & input == cond
            };
            if fire {
                Cpu::request_interrupt(self, Interrupt::Joypad);
            }
        }
    }
}
