use std::mem;

use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};

use crate::{
    gga::{
        addr::VCOUNT,
        cartridge::SaveType,
        cpu::{Cpu, Interrupt},
        Access, GameGirlAdv,
    },
    numutil::{word, NumExt},
};

const SRC_MASK: [u32; 4] = [0x7FF_FFFF, 0xFFF_FFFF, 0xFFF_FFFF, 0xFFF_FFFF];
const DST_MASK: [u32; 4] = [0x7FF_FFFF, 0x7FF_FFFF, 0x7FF_FFFF, 0xFFF_FFFF];

/// GGA's 4 DMA channels.
#[derive(Default, Deserialize, Serialize)]
pub struct Dmas {
    /// Internal source registers
    src: [u32; 4],
    /// Internal destination registers
    dst: [u32; 4],
    /// Internal cache shared between DMAs
    cache: u32,
    /// Currently running DMA, or 99
    pub(super) running: u16,
    /// DMAs waiting to run after current.
    queued: ArrayVec<(u16, DmaReason), 3>,
}

impl Dmas {
    /// Update all DMAs to see if they need ticking.
    pub fn update_all(gg: &mut GameGirlAdv, reason: DmaReason) {
        for idx in 0..4 {
            let base = Self::base_addr(idx);
            Self::step_dma(gg, idx, base, gg[base + 0xA], reason);
        }
    }

    /// Update a given DMA after it's control register was written.
    pub fn ctrl_write(gg: &mut GameGirlAdv, idx: u16, new_ctrl: u16) {
        let base = Self::base_addr(idx);
        let old_ctrl = gg[base + 0xA];
        if !old_ctrl.is_bit(15) && new_ctrl.is_bit(15) {
            // Reload SRC/DST
            let src = word(gg[base], gg[base + 2]);
            let dst = word(gg[base + 4], gg[base + 6]);
            gg.dma.src[idx.us()] = src & SRC_MASK[idx.us()];
            gg.dma.dst[idx.us()] = dst & DST_MASK[idx.us()];
        }

        gg[base + 0xA] = new_ctrl & if idx == 3 { 0xFFE0 } else { 0xF7E0 };
        Self::step_dma(gg, idx, base, new_ctrl, DmaReason::CtrlWrite);
    }

    /// Try to perform a FIFO transfer, if the DMA is otherwise configured for
    /// it.
    pub fn try_fifo_transfer(gg: &mut GameGirlAdv, idx: u16) {
        let base = Self::base_addr(idx);
        Self::step_dma(gg, idx, base, gg[base + 0xA], DmaReason::Fifo);
    }

    /// Get the destination register for a DMA; this is not the internal one.
    pub fn get_dest(gg: &mut GameGirlAdv, idx: u16) -> u32 {
        let base = Self::base_addr(idx);
        word(gg[base + 4], gg[base + 6])
    }

    /// Step a DMA and perform a transfer if possible.
    fn step_dma(gg: &mut GameGirlAdv, idx: u16, base: u32, ctrl: u16, reason: DmaReason) {
        let fifo = reason == DmaReason::Fifo;
        let vid_capture = idx == 3
            && (2..162).contains(&gg[VCOUNT])
            && reason == DmaReason::HBlank
            && ctrl.bits(12, 2) == 3;
        let on = ctrl.is_bit(15)
            && match ctrl.bits(12, 2) {
                0 => reason == DmaReason::CtrlWrite,
                1 => reason == DmaReason::VBlank,
                2 => reason == DmaReason::HBlank && gg[VCOUNT] < 160,
                _ => fifo || vid_capture,
            };
        if !on {
            return;
        }
        if gg.dma.running <= idx {
            gg.dma.queued.push((idx, reason));
            return;
        }

        let prev_dma = mem::replace(&mut gg.dma.running, idx);

        let count = gg[base + 8];
        let count = match count {
            _ if fifo => 4,
            0 if idx == 3 => 0x1_0000,
            0 => 0x4000,
            _ => count.u32(),
        };
        let src_mod = Self::get_step(ctrl.bits(7, 2));

        let dst_raw = ctrl.bits(5, 2);
        let dst_mod = match dst_raw {
            _ if fifo => 0,
            3 => {
                // Reload DST + Increment
                let dst = word(gg[base + 4], gg[base + 6]);
                gg.dma.dst[idx.us()] = dst & DST_MASK[idx.us()];
                2
            }
            _ => Self::get_step(dst_raw),
        };

        let word_transfer = ctrl.is_bit(10);
        if fifo || word_transfer {
            Self::perform_transfer::<true>(gg, idx.us(), count, src_mod * 2, dst_mod * 2);
        } else if idx == 3 {
            // Maybe alert EEPROM, if the cart has one
            if let SaveType::Eeprom(eeprom) = &mut gg.cart.save_type {
                eeprom.dma3_started(gg.dma.dst[3], count);
            }
            Self::perform_transfer::<false>(gg, 3, count, src_mod, dst_mod);
        } else {
            Self::perform_transfer::<false>(gg, idx.us(), count, src_mod, dst_mod);
        }

        if !ctrl.is_bit(9) || ctrl.bits(12, 2) == 0 || (vid_capture && gg[VCOUNT] == 161) {
            // Disable if reload is not enabled, it's an immediate transfer, or end of video
            // capture
            gg[base + 0xA] = ctrl.set_bit(15, false)
        }
        if ctrl.is_bit(14) {
            // Fire interrupt if configured
            Cpu::request_interrupt_idx(gg, Interrupt::Dma0 as u16 + idx)
        }

        gg.dma.running = prev_dma;
        if let Some((dma, reason)) = gg.dma.queued.pop() {
            let base = Self::base_addr(dma);
            Self::step_dma(gg, dma, base, gg[base + 0xA], reason);
        }
    }

    /// Perform a transfer.
    fn perform_transfer<const WORD: bool>(
        gg: &mut GameGirlAdv,
        idx: usize,
        count: u32,
        src_mod: i32,
        dst_mod: i32,
    ) {
        if gg.dma.dst[idx] < 0x200_0000 {
            return;
        }

        let mut kind = Access::NonSeq;
        if gg.dma.src[idx] >= 0x200_0000 {
            // First, align SRC/DST
            let align = if WORD { 3 } else { 1 };
            gg.dma.src[idx] &= !align;
            gg.dma.dst[idx] &= !align;

            for _ in 0..count {
                if WORD {
                    let value = gg.read_word(gg.dma.src[idx], kind);
                    gg.write_word(gg.dma.dst[idx], value, kind);
                } else {
                    let value = gg.read_hword(gg.dma.src[idx], kind).u16();
                    gg.write_hword(gg.dma.dst[idx], value, kind);
                }
                gg.dma.src[idx] = gg.dma.src[idx].wrapping_add_signed(src_mod);
                gg.dma.dst[idx] = gg.dma.dst[idx].wrapping_add_signed(dst_mod);
                // Only first is NonSeq
                kind = Access::Seq;
                gg.advance_clock();
            }

            // Put last value into cache
            if WORD {
                gg.dma.cache = gg.get_word(gg.dma.src[idx].wrapping_add_signed(-src_mod));
            } else {
                let value = gg.get_hword(gg.dma.src[idx].wrapping_add_signed(-src_mod));
                gg.dma.cache = word(value, value);
            }
        } else {
            for _ in 0..count {
                if WORD {
                    gg.write_word(gg.dma.dst[idx], gg.dma.cache, kind);
                } else if gg.dma.dst[idx].is_bit(1) {
                    gg.write_hword(gg.dma.dst[idx], (gg.dma.cache >> 16).u16(), kind);
                } else {
                    gg.write_hword(gg.dma.dst[idx], gg.dma.cache.u16(), kind);
                }
                gg.dma.src[idx] = gg.dma.src[idx].wrapping_add_signed(src_mod);
                gg.dma.dst[idx] = gg.dma.dst[idx].wrapping_add_signed(dst_mod);
                // Only first is NonSeq
                kind = Access::Seq;
                gg.advance_clock();
            }
        }
        gg.add_i_cycles(2);
    }

    /// Get the step with which to change SRC/DST registers after every write.
    /// Multiplied by 2 for word-sized DMAs.
    /// Inc+Reload handled separately.
    fn get_step(bits: u16) -> i32 {
        match bits {
            0 => 2,
            1 => -2,
            _ => 0,
        }
    }

    /// Get the base address for a DMA (First register: SRC low)
    fn base_addr(idx: u16) -> u32 {
        0xB0 + (idx.u32() * 0xC)
    }
}

/// Reason for why a DMA transfer attempt was initiated.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum DmaReason {
    /// The control register was written.
    CtrlWrite,
    /// The PPU entered HBlank.
    HBlank,
    /// The PPU entered VBlank.
    VBlank,
    /// A FIFO sound channel is requesting new samples.
    Fifo,
}
