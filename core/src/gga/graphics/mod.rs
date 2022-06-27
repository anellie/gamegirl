//! For this PPU implementation, I took a lot of reference from DenSinH's GBAC-.
//! It is not an outright copy, but I want to thank them for their code
//! that helped me understand the PPU's more complex behavior.
//! The code is under the MIT license at https://github.com/DenSinH/GBAC-.

mod bitmap;
mod objects;
mod palette;
mod render;
mod tile;

use serde::{Deserialize, Serialize};

use super::memory::KB;
use crate::{
    common::BorrowedSystem,
    gga::{
        addr::{DISPCNT, DISPSTAT, VCOUNT},
        cpu::{Cpu, Interrupt},
        dma::Dmas,
        scheduling::{AdvEvent, PpuEvent},
        GameGirlAdv,
    },
    numutil::NumExt,
    Colour,
};

// DISPCNT
const FRAME_SELECT: u16 = 4;
const OAM_HBLANK_FREE: u16 = 5;
const OBJ_MAPPING_1D: u16 = 6;
const FORCED_BLANK: u16 = 7;
const BG0_EN: u16 = 8;
const BG2_EN: u16 = 10;
const OBJ_EN: u16 = 12;
const WIN0_EN: u16 = 13;
const WIN1_EN: u16 = 14;
const WIN_OBJS: u16 = 15;

// DISPSTAT
pub const VBLANK: u16 = 0;
pub const HBLANK: u16 = 1;
const LYC_MATCH: u16 = 2;
const VBLANK_IRQ: u16 = 3;
const HBLANK_IRQ: u16 = 4;
const LYC_IRQ: u16 = 5;

type Layer = [Colour; 240];

#[derive(Deserialize, Serialize)]
pub struct Ppu {
    #[serde(with = "serde_arrays")]
    pub palette: [u8; KB],
    #[serde(with = "serde_arrays")]
    pub vram: [u8; 96 * KB],
    #[serde(with = "serde_arrays")]
    pub oam: [u8; KB],

    #[serde(skip)]
    #[serde(default = "serde_colour_arr")]
    pixels: [Colour; 240 * 160],
    #[serde(skip)]
    #[serde(default = "serde_layer_arr")]
    bg_layers: [Layer; 4],
    #[serde(skip)]
    #[serde(default = "serde_layer_arr")]
    obj_layers: [Layer; 4],

    #[serde(skip)]
    #[serde(default)]
    pub last_frame: Option<Vec<Colour>>,
}

impl Ppu {
    pub fn handle_event(gg: &mut GameGirlAdv, event: PpuEvent, late_by: u32) {
        let (next_event, cycles) = match event {
            PpuEvent::HblankStart => {
                Self::render_line(gg);

                Self::maybe_interrupt(gg, Interrupt::HBlank, HBLANK_IRQ);
                gg[DISPSTAT] = gg[DISPSTAT].set_bit(HBLANK, true);
                Dmas::update(gg);

                (PpuEvent::HblankEnd, 272u32)
            }

            PpuEvent::HblankEnd => {
                gg[VCOUNT] += 1;

                gg[DISPSTAT] = gg[DISPSTAT].set_bit(LYC_MATCH, false);
                if gg[DISPSTAT].is_bit(LYC_IRQ) && gg[VCOUNT] == gg[DISPSTAT].bits(8, 8) {
                    gg[DISPSTAT] = gg[DISPSTAT].set_bit(LYC_MATCH, true);
                    Self::maybe_interrupt(gg, Interrupt::VCounter, LYC_IRQ);
                }

                if gg[VCOUNT] == 160 {
                    gg[DISPSTAT] = gg[DISPSTAT].set_bit(VBLANK, true);
                    Self::maybe_interrupt(gg, Interrupt::VBlank, VBLANK_IRQ);
                    Dmas::update(gg);
                    gg.ppu.last_frame = Some(gg.ppu.pixels.to_vec());
                } else if gg[VCOUNT] > 227 {
                    gg[VCOUNT] = 0;
                    gg[DISPSTAT] = gg[DISPSTAT].set_bit(VBLANK, false);
                    (gg.options.frame_finished)(BorrowedSystem::GGA(gg));
                }

                gg[DISPSTAT] = gg[DISPSTAT].set_bit(HBLANK, false);
                (PpuEvent::HblankStart, 960)
            }
        };
        gg.scheduler.schedule(
            AdvEvent::PpuEvent(next_event),
            cycles.saturating_sub(late_by),
        );
    }

    fn maybe_interrupt(gg: &mut GameGirlAdv, int: Interrupt, bit: u16) {
        if gg[DISPSTAT].is_bit(bit) {
            Cpu::request_interrupt(gg, int);
        }
    }

    fn render_line(gg: &mut GameGirlAdv) {
        let line = gg[VCOUNT];
        if line >= 160 {
            return;
        }
        if gg[DISPCNT].is_bit(FORCED_BLANK) {
            let start = line.us() * 240;
            for pixel in 0..240 {
                gg.ppu.pixels[start + pixel] = [31, 31, 31, 255];
            }
            return;
        }

        match gg[DISPCNT] & 7 {
            0 => Self::render_mode0(gg, line),
            1 => Self::render_mode1(gg, line),
            2 => Self::render_mode2(gg, line),
            3 => Self::render_mode3(gg, line),
            4 => Self::render_mode4(gg, line),
            5 => Self::render_mode5(gg, line),
            _ => println!("Invalid mode {}", gg[DISPCNT] & 7),
        }

        Self::finish_line(gg, line);
    }

    fn finish_line(gg: &mut GameGirlAdv, line: u16) {
        let start = line.us() * 240;
        let mut backdrop = gg.ppu.idx_to_palette::<false>(0); // BG0 is backdrop
        Self::adjust_pixel(&mut backdrop);

        'pixels: for (x, pixel) in gg.ppu.pixels[start..].iter_mut().take(240).enumerate() {
            for prio in 0..4 {
                if gg.ppu.obj_layers[prio][x] != EMPTY {
                    *pixel = gg.ppu.obj_layers[prio][x];
                    Self::adjust_pixel(pixel);
                    continue 'pixels;
                }
                if gg.ppu.bg_layers[prio][x] != EMPTY {
                    *pixel = gg.ppu.bg_layers[prio][x];
                    Self::adjust_pixel(pixel);
                    continue 'pixels;
                }
            }

            // No matching pixel found...
            *pixel = backdrop;
        }

        // Clear last line buffers
        gg.ppu.bg_layers = serde_layer_arr();
        gg.ppu.obj_layers = serde_layer_arr();
    }

    #[inline]
    fn adjust_pixel(pixel: &mut Colour) {
        for col in pixel.iter_mut().take(3) {
            *col = (*col << 3) | (*col >> 2);
        }
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            palette: [0; KB],
            vram: [0; 96 * KB],
            oam: [0; KB],
            pixels: serde_colour_arr(),
            bg_layers: serde_layer_arr(),
            obj_layers: serde_layer_arr(),
            last_frame: None,
        }
    }
}

fn serde_colour_arr() -> [Colour; 240 * 160] {
    [[0, 0, 0, 255]; 240 * 160]
}
fn serde_layer_arr() -> [Layer; 4] {
    [[EMPTY; 240]; 4]
}

const EMPTY: Colour = [0, 0, 0, 0];
