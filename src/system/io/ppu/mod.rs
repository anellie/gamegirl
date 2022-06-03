use crate::numutil::NumExt;
use crate::system::cpu::Interrupt;
use crate::system::io::addr::*;
use crate::system::io::ppu::cgb::Cgb;
use crate::system::io::Mmu;
use crate::system::GameGirl;
use crate::Colour;
use serde::Deserialize;
use serde::Serialize;
use serde_big_array::BigArray;

mod cgb;
mod dmg;

// LCDC
const BG_EN: u16 = 0;
const OBJ_EN: u16 = 1;
const BIG_OBJS: u16 = 2;
const BG_MAP: u16 = 3;
const ALT_BG_TILE: u16 = 4;
const WIN_EN: u16 = 5;
const WIN_MAP: u16 = 6;
pub(super) const DISP_EN: u16 = 7;

// OAM sprites 'option' byte
const DMG_PAL: u16 = 4;
const X_FLIP: u16 = 5;
const Y_FLIP: u16 = 6;
const PRIORITY: u16 = 7;
const CGB_BANK: u16 = 3;

/// PPU of the system, with differing ways of function depending on
/// DMG/CGB mode.
#[derive(Deserialize, Serialize)]
pub struct Ppu {
    mode: Mode,
    mode_clock: u16,
    #[serde(with = "BigArray")]
    bg_occupied_pixels: [bool; 160],
    window_line: u8,
    kind: PpuKind,

    #[serde(with = "BigArray")]
    pixels: [Colour; 160 * 144],
    /// The last frame finished by the PPU, ready for display.
    #[serde(skip)]
    #[serde(default)]
    pub last_frame: Option<Vec<Colour>>,
}

impl Ppu {
    pub(super) fn step(gg: &mut GameGirl, t_cycles: usize) {
        if !gg.lcdc(DISP_EN) {
            return;
        }
        let mode = {
            let ppu = gg.ppu();
            ppu.mode_clock += t_cycles as u16;
            if ppu.mode_clock < ppu.mode.cycles() {
                return;
            }
            ppu.mode_clock -= ppu.mode.cycles();
            ppu.mode
        };

        let next_mode = match mode {
            Mode::OAMScan => Mode::Upload,

            Mode::Upload => {
                Self::render_line(gg);
                gg.mmu.ppu.bg_occupied_pixels = [false; 160];
                gg.mmu.hdma.ppu_in_hblank = true;
                Self::stat_interrupt(gg, 3);
                Mode::HBlank
            }

            Mode::HBlank => {
                gg.mmu[LY] += 1;
                Self::stat_interrupt(gg, 5);
                Self::lyc_interrupt(gg);
                if gg.mmu[LY] == 144 {
                    Self::stat_interrupt(gg, 4);
                    gg.request_interrupt(Interrupt::VBlank);
                    gg.ppu().last_frame = Some(gg.mmu.ppu.pixels.to_vec());
                    Mode::VBlank
                } else {
                    Mode::OAMScan
                }
            }

            Mode::VBlank => {
                gg.mmu[LY] += 1;
                Self::lyc_interrupt(gg);
                if gg.mmu[LY] > 153 {
                    gg.mmu[LY] = 0;
                    gg.mmu.ppu.window_line = 0;
                    Self::stat_interrupt(gg, 5);
                    Mode::OAMScan
                } else {
                    Mode::VBlank
                }
            }
        };

        gg.mmu[STAT] =
            gg.mmu[STAT].set_bit(2, gg.mmu[LYC] == gg.mmu[LY]).u8() & 0xFC | next_mode.ordinal();
        gg.ppu().mode = next_mode;
    }

    fn stat_interrupt(gg: &mut GameGirl, bit: u16) {
        if gg.mmu[STAT].is_bit(bit) {
            gg.request_interrupt(Interrupt::Stat);
        }
    }

    fn lyc_interrupt(gg: &mut GameGirl) {
        if gg.mmu[LYC] == gg.mmu[LY] {
            Self::stat_interrupt(gg, 6);
        }
    }

    fn render_line(gg: &mut GameGirl) {
        match &gg.mmu.ppu.kind {
            PpuKind::Dmg { .. } if gg.lcdc(BG_EN) => {
                Self::render_bg(gg);
                if gg.lcdc(WIN_EN) {
                    Self::render_window(gg);
                }
            }
            PpuKind::Dmg { .. } => Self::clear_line(gg),

            PpuKind::Cgb(cgb) => {
                // Emulate DMG behavior in DMG mode.
                if !cgb.dmg_used_x_obj_cords.is_some() || gg.lcdc(BG_EN) {
                    Self::render_bg(gg);
                    if gg.lcdc(WIN_EN) {
                        Self::render_window(gg);
                    }
                } else {
                    Self::clear_line(gg)
                }
            }
        }

        if gg.lcdc(OBJ_EN) {
            Self::render_objs(gg);
        }

        match &mut gg.ppu().kind {
            PpuKind::Cgb(Cgb {
                unavailable_pixels,
                dmg_used_x_obj_cords: Some(used_x_obj_coords),
                ..
            }) => {
                *unavailable_pixels = [false; 160];
                *used_x_obj_coords = [None; 10];
            }
            PpuKind::Dmg { used_x_obj_coords } => *used_x_obj_coords = [None; 10],
            PpuKind::Cgb(cgb) => cgb.unavailable_pixels = [false; 160],
        }
    }

    fn render_bg(gg: &mut GameGirl) {
        // Only render until the point where the window starts, should it be active
        let end_x =
            if gg.lcdc(WIN_EN) && (7u8..166u8).contains(&gg.mmu[WX]) && gg.mmu[WY] <= gg.mmu[LY] {
                gg.mmu[WX] - 7
            } else {
                160
            };
        Self::render_bg_or_window(
            gg,
            gg.mmu[SCX],
            0,
            end_x,
            gg.map_addr(BG_MAP),
            gg.mmu[SCY].wrapping_add(gg.mmu[LY]),
            true,
        )
    }

    fn render_window(gg: &mut GameGirl) {
        let wx = gg.mmu[WX] as i16 - 7;
        if !(0..=159).contains(&wx) || gg.mmu[WY] > gg.mmu[LY] {
            return;
        }

        Self::render_bg_or_window(
            gg,
            0,
            wx as u8,
            160,
            gg.map_addr(WIN_MAP),
            gg.mmu.ppu.window_line,
            false,
        );
        gg.ppu().window_line += 1;
    }

    fn render_bg_or_window(
        gg: &mut GameGirl,
        scroll_x: u8,
        start_x: u8,
        end_x: u8,
        map_addr: u16,
        map_line: u8,
        correct_tile_addr: bool,
    ) {
        let method = match gg.mmu.ppu.kind {
            PpuKind::Dmg { .. } => Self::dmg_render_bg_or_window,
            PpuKind::Cgb(_) => Self::cgb_render_bg_or_window,
        };
        method(
            gg,
            scroll_x,
            start_x,
            end_x,
            map_addr,
            map_line,
            correct_tile_addr,
        );
    }

    fn render_objs(gg: &mut GameGirl) {
        let mut count = 0;
        let sprite_offs = 8 + gg.lcdc(BIG_OBJS) as i16 * 8;
        let ly = gg.mmu[LY] as i16;

        for idx in 0..40 {
            let sprite = Sprite::from(&gg.mmu, idx);
            if sprite.y <= ly
                && ((sprite.y + sprite_offs) > ly)
                && gg.ppu().allow_obj(sprite.x as u8, count)
            {
                Self::render_obj(gg, ly, sprite);
                count += 1;
                if count == 10 {
                    break;
                }
            }
        }
    }

    fn render_obj(gg: &mut GameGirl, line: i16, sprite: Sprite) {
        // OBP0/OBP1 are right next to each other, make use of it
        let dmg_palette = gg.mmu[OBP0 + sprite.opt.bit(DMG_PAL)];
        let tile_y_op = (line - sprite.y) & 0x07;
        let tile_y = if sprite.opt.is_bit(Y_FLIP) {
            7 - tile_y_op
        } else {
            tile_y_op
        };

        let tile_num = match () {
            _ if gg.lcdc(BIG_OBJS) && (((line - sprite.y) <= 7) != sprite.opt.is_bit(Y_FLIP)) => {
                sprite.tile_num & 0xFE
            }
            _ if gg.lcdc(BIG_OBJS) => sprite.tile_num | 0x01,
            _ => sprite.tile_num,
        };

        let tile_data_addr = (tile_num.u16() * 0x10)
            + (tile_y as u16 * 2)
            + ((gg.mmu.cgb && sprite.opt.is_bit(CGB_BANK)) as u16) * 0x2000;
        let high = gg.mmu.vram[tile_data_addr.us() + 1];
        let low = gg.mmu.vram[tile_data_addr.us()];

        for tile_x in 0..8 {
            let colour_idx = if !sprite.opt.is_bit(X_FLIP) {
                (high.bit(7 - tile_x) << 1) + low.bit(7 - tile_x)
            } else {
                (high.bit(tile_x) << 1) + low.bit(tile_x)
            };
            let screen_x = sprite.x + tile_x as i16;
            if (0..160).contains(&screen_x)
                && colour_idx != 0
                && Self::is_pixel_free(gg, screen_x, !sprite.opt.is_bit(PRIORITY))
            {
                Self::draw_obj_pixel(
                    gg,
                    screen_x as u8,
                    line as u8,
                    colour_idx,
                    dmg_palette,
                    sprite.opt & 7,
                );
            }
        }
    }

    fn draw_obj_pixel(
        gg: &mut GameGirl,
        x: u8,
        y: u8,
        colour_idx: u16,
        dmg_palette: u8,
        cgb_palette: u8,
    ) {
        let colour = match &mut gg.mmu.ppu.kind {
            PpuKind::Dmg { .. } => Self::get_colour(dmg_palette, colour_idx),
            PpuKind::Cgb(cgb) => {
                cgb.unavailable_pixels[x.us()] = colour_idx != 0;
                cgb.obj_palettes[((cgb_palette * 4) + colour_idx.u8()).us()].colour
            }
        };
        gg.ppu().set_pixel(x, y, colour);
    }

    fn set_pixel(&mut self, x: u8, y: u8, col: Colour) {
        let idx = x.us() + (y.us() * 160);
        self.pixels[idx] = col;
    }

    fn is_pixel_free(gg: &GameGirl, x: i16, prio: bool) -> bool {
        let base = prio || !gg.mmu.ppu.bg_occupied_pixels[x as usize];
        match &gg.mmu.ppu.kind {
            PpuKind::Dmg { .. } => base,
            // Make sure we ignore unavailable pixels in DMG compat mode
            PpuKind::Cgb(cgb) => {
                base && (cgb.dmg_used_x_obj_cords.is_some() || !cgb.unavailable_pixels[x as usize])
            }
        }
    }

    fn _bg_idx_tile_data_addr(gg: &GameGirl, window: bool, idx: u16) -> u16 {
        let addr = match () {
            _ if window && gg.lcdc(WIN_MAP) => 0x1C00,
            _ if window => 0x1800,
            _ if gg.lcdc(BG_MAP) => 0x1C00,
            _ => 0x1800,
        } + idx;
        Self::bg_tile_data_addr(gg, gg.mmu.vram[addr.us()])
    }

    fn bg_tile_data_addr(gg: &GameGirl, idx: u8) -> u16 {
        if gg.lcdc(ALT_BG_TILE) {
            idx.u16() * 0x10
        } else {
            (0x1000 + (idx as i8 as i16 * 0x10)) as u16
        }
    }

    pub(super) fn new() -> Self {
        Self {
            mode: Mode::OAMScan,
            mode_clock: 0,
            bg_occupied_pixels: [false; 160],
            window_line: 0,
            kind: PpuKind::Dmg {
                used_x_obj_coords: [None; 10],
            },
            pixels: [Colour::BLACK; 160 * 144],
            last_frame: None,
        }
    }

    pub(super) fn switch_kind(&mut self, cgb: bool) {
        self.kind = if cgb {
            PpuKind::Cgb(Cgb::default())
        } else {
            PpuKind::Dmg {
                used_x_obj_coords: [None; 10],
            }
        };
    }
}

/// The kind of PPU this is
#[derive(Deserialize, Serialize)]
pub enum PpuKind {
    Dmg { used_x_obj_coords: [Option<u8>; 10] },
    Cgb(Cgb),
}

/// Mode the PPU can be in.
#[derive(Copy, Clone, Deserialize, Serialize)]
enum Mode {
    HBlank = 204,
    VBlank = 456,
    OAMScan = 80,
    Upload = 172,
}

impl Mode {
    fn cycles(self) -> u16 {
        self as u16
    }

    fn ordinal(self) -> u8 {
        // ehhh
        match self {
            Mode::HBlank => 0,
            Mode::VBlank => 1,
            Mode::OAMScan => 2,
            Mode::Upload => 3,
        }
    }
}

/// Data for a single sprite in OAM.
#[derive(Debug)]
struct Sprite {
    x: i16,
    y: i16,
    tile_num: u8,
    opt: u8,
}

impl Sprite {
    fn from(mmu: &Mmu, idx: u8) -> Self {
        let base = idx.us() * 4;
        Self {
            x: mmu.oam[base + 1] as i16 - 8,
            y: mmu.oam[base] as i16 - 16,
            tile_num: mmu.oam[base + 2],
            opt: mmu.oam[base + 3],
        }
    }
}

impl GameGirl {
    #[inline]
    fn ppu(&mut self) -> &mut Ppu {
        &mut self.mmu.ppu
    }

    #[inline]
    fn lcdc(&self, bit: u16) -> bool {
        self.mmu[LCDC].is_bit(bit)
    }

    #[inline]
    fn map_addr(&self, bit: u16) -> u16 {
        if self.mmu[LCDC].is_bit(bit) {
            0x1C00
        } else {
            0x1800
        }
    }
}
