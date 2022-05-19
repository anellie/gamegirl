use std::ops::Range;

pub const BOOTROM_DISABLE: u16 = 0xFF50;
pub const INVALID_READ: u8 = 0xFF;

// LCD
pub const LCDC: u16 = 0xFF40;
pub const LY: u16 = 0xFF44;
pub const LYC: u16 = 0xFF45;
pub const STAT: u16 = 0xFF41;
pub const SCY: u16 = 0xFF42;
pub const SCX: u16 = 0xFF43;
pub const WY: u16 = 0xFF4A;
pub const WX: u16 = 0xFF4B;
pub const BGP: u16 = 0xFF47;
pub const OBP0: u16 = 0xFF48;
pub const OBP1: u16 = 0xFF49;
pub const BCPS: u16 = 0xFF68;
pub const BCPD: u16 = 0xFF69;
pub const OCPS: u16 = 0xFF6A;
pub const OCPD: u16 = 0xFF6B;

// Interrupts
pub const IF: u16 = 0xFF0F;
pub const IE: u16 = 0xFFFF;

// Serial
pub const SB: u16 = 0xFF01;
pub const SC: u16 = 0xFF02;

// Timer
pub const DIV: u16 = 0xFF04;
pub const TIMA: u16 = 0xFF05;
pub const TMA: u16 = 0xFF06;
pub const TAC: u16 = 0xFF07;

// Joypad
pub const JOYP: u16 = 0xFF00;

// (H)DMA
pub const DMA: u16 = 0xFF46;
pub const HDMA_SRC_HIGH: u16 = 0xFF51;
pub const HDMA_SRC_LOW: u16 = 0xFF52;
pub const HDMA_DEST_HIGH: u16 = 0xFF53;
pub const HDMA_DEST_LOW: u16 = 0xFF54;
pub const HDMA_START: u16 = 0xFF55;

// Sound
pub const NR10: u16 = 0xFF10;
pub const NR11: u16 = 0xFF11;
pub const NR12: u16 = 0xFF12;
pub const NR13: u16 = 0xFF13;
pub const NR14: u16 = 0xFF14;
pub const NR21: u16 = 0xFF16;
pub const NR22: u16 = 0xFF17;
pub const NR23: u16 = 0xFF18;
pub const NR24: u16 = 0xFF19;
pub const NR30: u16 = 0xFF1A;
pub const NR31: u16 = 0xFF1B;
pub const NR32: u16 = 0xFF1C;
pub const NR33: u16 = 0xFF1D;
pub const NR34: u16 = 0xFF1E;
pub const NR41: u16 = 0xFF20;
pub const NR42: u16 = 0xFF21;
pub const NR43: u16 = 0xFF22;
pub const NR44: u16 = 0xFF23;
pub const NR50: u16 = 0xFF24;
pub const NR51: u16 = 0xFF25;
pub const NR52: u16 = 0xFF26;

pub const WAVE_SAMPLES: Range<u16> = 0xFF30..0xFF3F;

// CGB
pub const VRAM_SELECT: u16 = 0xFF4F;
pub const WRAM_SELECT: u16 = 0xFF70;
pub const HDMA: Range<u16> = HDMA_SRC_HIGH..HDMA_START;
pub const KEY1: u16 = 0xFF4D;

/// DMG BOOT ROM, Bootix made by Hacktix: https://github.com/Hacktix/Bootix
/// Thank you, Hacktix! This is Version 1.2.
pub const BOOTIX_ROM: [i8; 256] = [
    49, -2, -1, 33, -1, -97, -81, 50, -53, 124, 32, -6, 14, 17, 33, 38, -1, 62, -128, 50, -30, 12,
    62, -13, 50, -30, 12, 62, 119, 50, -30, 17, 4, 1, 33, 16, -128, 26, -51, -72, 0, 26, -53, 55,
    -51, -72, 0, 19, 123, -2, 52, 32, -16, 17, -52, 0, 6, 8, 26, 19, 34, 35, 5, 32, -7, 33, 4,
    -103, 1, 12, 1, -51, -79, 0, 62, 25, 119, 33, 36, -103, 14, 12, -51, -79, 0, 62, -111, -32, 64,
    6, 16, 17, -44, 0, 120, -32, 67, 5, 123, -2, -40, 40, 4, 26, -32, 71, 19, 14, 28, -51, -89, 0,
    -81, -112, -32, 67, 5, 14, 28, -51, -89, 0, -81, -80, 32, -32, -32, 67, 62, -125, -51, -97, 0,
    14, 39, -51, -89, 0, 62, -63, -51, -97, 0, 17, -118, 1, -16, 68, -2, -112, 32, -6, 27, 122,
    -77, 32, -11, 24, 73, 14, 19, -30, 12, 62, -121, -30, -55, -16, 68, -2, -112, 32, -6, 13, 32,
    -9, -55, 120, 34, 4, 13, 32, -6, -55, 71, 14, 4, -81, -59, -53, 16, 23, -63, -53, 16, 23, 13,
    32, -11, 34, 35, 34, 35, -55, 60, 66, -71, -91, -71, -91, 66, 60, 0, 84, -88, -4, 66, 79, 79,
    84, 73, 88, 46, 68, 77, 71, 32, 118, 49, 46, 50, 0, 62, -1, -58, 1, 11, 30, -40, 33, 77, 1, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 62, 1, -32, 80,
];

/// CGB BOOT ROM, the original one. Taken from coffee-gb sources.
pub const CGB_BOOTROM: [u8; 2048] = [
    0x31, 0xfe, 0xff, 0x3e, 0x02, 0xc3, 0x7c, 0x00, 0xd3, 0x00, 0x98, 0xa0, 0x12, 0xd3, 0x00, 0x80,
    0x00, 0x40, 0x1e, 0x53, 0xd0, 0x00, 0x1f, 0x42, 0x1c, 0x00, 0x14, 0x2a, 0x4d, 0x19, 0x8c, 0x7e,
    0x00, 0x7c, 0x31, 0x6e, 0x4a, 0x45, 0x52, 0x4a, 0x00, 0x00, 0xff, 0x53, 0x1f, 0x7c, 0xff, 0x03,
    0x1f, 0x00, 0xff, 0x1f, 0xa7, 0x00, 0xef, 0x1b, 0x1f, 0x00, 0xef, 0x1b, 0x00, 0x7c, 0x00, 0x00,
    0xff, 0x03, 0xce, 0xed, 0x66, 0x66, 0xcc, 0x0d, 0x00, 0x0b, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0c,
    0x00, 0x0d, 0x00, 0x08, 0x11, 0x1f, 0x88, 0x89, 0x00, 0x0e, 0xdc, 0xcc, 0x6e, 0xe6, 0xdd, 0xdd,
    0xd9, 0x99, 0xbb, 0xbb, 0x67, 0x63, 0x6e, 0x0e, 0xec, 0xcc, 0xdd, 0xdc, 0x99, 0x9f, 0xbb, 0xb9,
    0x33, 0x3e, 0x3c, 0x42, 0xb9, 0xa5, 0xb9, 0xa5, 0x42, 0x3c, 0x58, 0x43, 0xe0, 0x70, 0x3e, 0xfc,
    0xe0, 0x47, 0xcd, 0x75, 0x02, 0xcd, 0x00, 0x02, 0x26, 0xd0, 0xcd, 0x03, 0x02, 0x21, 0x00, 0xfe,
    0x0e, 0xa0, 0xaf, 0x22, 0x0d, 0x20, 0xfc, 0x11, 0x04, 0x01, 0x21, 0x10, 0x80, 0x4c, 0x1a, 0xe2,
    0x0c, 0xcd, 0xc6, 0x03, 0xcd, 0xc7, 0x03, 0x13, 0x7b, 0xfe, 0x34, 0x20, 0xf1, 0x11, 0x72, 0x00,
    0x06, 0x08, 0x1a, 0x13, 0x22, 0x23, 0x05, 0x20, 0xf9, 0xcd, 0xf0, 0x03, 0x3e, 0x01, 0xe0, 0x4f,
    0x3e, 0x91, 0xe0, 0x40, 0x21, 0xb2, 0x98, 0x06, 0x4e, 0x0e, 0x44, 0xcd, 0x91, 0x02, 0xaf, 0xe0,
    0x4f, 0x0e, 0x80, 0x21, 0x42, 0x00, 0x06, 0x18, 0xf2, 0x0c, 0xbe, 0x20, 0xfe, 0x23, 0x05, 0x20,
    0xf7, 0x21, 0x34, 0x01, 0x06, 0x19, 0x78, 0x86, 0x2c, 0x05, 0x20, 0xfb, 0x86, 0x20, 0xfe, 0xcd,
    0x1c, 0x03, 0x18, 0x02, 0x00, 0x00, 0xcd, 0xd0, 0x05, 0xaf, 0xe0, 0x70, 0x3e, 0x11, 0xe0, 0x50,
    0x21, 0x00, 0x80, 0xaf, 0x22, 0xcb, 0x6c, 0x28, 0xfb, 0xc9, 0x2a, 0x12, 0x13, 0x0d, 0x20, 0xfa,
    0xc9, 0xe5, 0x21, 0x0f, 0xff, 0xcb, 0x86, 0xcb, 0x46, 0x28, 0xfc, 0xe1, 0xc9, 0x11, 0x00, 0xff,
    0x21, 0x03, 0xd0, 0x0e, 0x0f, 0x3e, 0x30, 0x12, 0x3e, 0x20, 0x12, 0x1a, 0x2f, 0xa1, 0xcb, 0x37,
    0x47, 0x3e, 0x10, 0x12, 0x1a, 0x2f, 0xa1, 0xb0, 0x4f, 0x7e, 0xa9, 0xe6, 0xf0, 0x47, 0x2a, 0xa9,
    0xa1, 0xb0, 0x32, 0x47, 0x79, 0x77, 0x3e, 0x30, 0x12, 0xc9, 0x3e, 0x80, 0xe0, 0x68, 0xe0, 0x6a,
    0x0e, 0x6b, 0x2a, 0xe2, 0x05, 0x20, 0xfb, 0x4a, 0x09, 0x43, 0x0e, 0x69, 0x2a, 0xe2, 0x05, 0x20,
    0xfb, 0xc9, 0xc5, 0xd5, 0xe5, 0x21, 0x00, 0xd8, 0x06, 0x01, 0x16, 0x3f, 0x1e, 0x40, 0xcd, 0x4a,
    0x02, 0xe1, 0xd1, 0xc1, 0xc9, 0x3e, 0x80, 0xe0, 0x26, 0xe0, 0x11, 0x3e, 0xf3, 0xe0, 0x12, 0xe0,
    0x25, 0x3e, 0x77, 0xe0, 0x24, 0x21, 0x30, 0xff, 0xaf, 0x0e, 0x10, 0x22, 0x2f, 0x0d, 0x20, 0xfb,
    0xc9, 0xcd, 0x11, 0x02, 0xcd, 0x62, 0x02, 0x79, 0xfe, 0x38, 0x20, 0x14, 0xe5, 0xaf, 0xe0, 0x4f,
    0x21, 0xa7, 0x99, 0x3e, 0x38, 0x22, 0x3c, 0xfe, 0x3f, 0x20, 0xfa, 0x3e, 0x01, 0xe0, 0x4f, 0xe1,
    0xc5, 0xe5, 0x21, 0x43, 0x01, 0xcb, 0x7e, 0xcc, 0x89, 0x05, 0xe1, 0xc1, 0xcd, 0x11, 0x02, 0x79,
    0xd6, 0x30, 0xd2, 0x06, 0x03, 0x79, 0xfe, 0x01, 0xca, 0x06, 0x03, 0x7d, 0xfe, 0xd1, 0x28, 0x21,
    0xc5, 0x06, 0x03, 0x0e, 0x01, 0x16, 0x03, 0x7e, 0xe6, 0xf8, 0xb1, 0x22, 0x15, 0x20, 0xf8, 0x0c,
    0x79, 0xfe, 0x06, 0x20, 0xf0, 0x11, 0x11, 0x00, 0x19, 0x05, 0x20, 0xe7, 0x11, 0xa1, 0xff, 0x19,
    0xc1, 0x04, 0x78, 0x1e, 0x83, 0xfe, 0x62, 0x28, 0x06, 0x1e, 0xc1, 0xfe, 0x64, 0x20, 0x07, 0x7b,
    0xe0, 0x13, 0x3e, 0x87, 0xe0, 0x14, 0xfa, 0x02, 0xd0, 0xfe, 0x00, 0x28, 0x0a, 0x3d, 0xea, 0x02,
    0xd0, 0x79, 0xfe, 0x01, 0xca, 0x91, 0x02, 0x0d, 0xc2, 0x91, 0x02, 0xc9, 0x0e, 0x26, 0xcd, 0x4a,
    0x03, 0xcd, 0x11, 0x02, 0xcd, 0x62, 0x02, 0x0d, 0x20, 0xf4, 0xcd, 0x11, 0x02, 0x3e, 0x01, 0xe0,
    0x4f, 0xcd, 0x3e, 0x03, 0xcd, 0x41, 0x03, 0xaf, 0xe0, 0x4f, 0xcd, 0x3e, 0x03, 0xc9, 0x21, 0x08,
    0x00, 0x11, 0x51, 0xff, 0x0e, 0x05, 0xcd, 0x0a, 0x02, 0xc9, 0xc5, 0xd5, 0xe5, 0x21, 0x40, 0xd8,
    0x0e, 0x20, 0x7e, 0xe6, 0x1f, 0xfe, 0x1f, 0x28, 0x01, 0x3c, 0x57, 0x2a, 0x07, 0x07, 0x07, 0xe6,
    0x07, 0x47, 0x3a, 0x07, 0x07, 0x07, 0xe6, 0x18, 0xb0, 0xfe, 0x1f, 0x28, 0x01, 0x3c, 0x0f, 0x0f,
    0x0f, 0x47, 0xe6, 0xe0, 0xb2, 0x22, 0x78, 0xe6, 0x03, 0x5f, 0x7e, 0x0f, 0x0f, 0xe6, 0x1f, 0xfe,
    0x1f, 0x28, 0x01, 0x3c, 0x07, 0x07, 0xb3, 0x22, 0x0d, 0x20, 0xc7, 0xe1, 0xd1, 0xc1, 0xc9, 0x0e,
    0x00, 0x1a, 0xe6, 0xf0, 0xcb, 0x49, 0x28, 0x02, 0xcb, 0x37, 0x47, 0x23, 0x7e, 0xb0, 0x22, 0x1a,
    0xe6, 0x0f, 0xcb, 0x49, 0x20, 0x02, 0xcb, 0x37, 0x47, 0x23, 0x7e, 0xb0, 0x22, 0x13, 0xcb, 0x41,
    0x28, 0x0d, 0xd5, 0x11, 0xf8, 0xff, 0xcb, 0x49, 0x28, 0x03, 0x11, 0x08, 0x00, 0x19, 0xd1, 0x0c,
    0x79, 0xfe, 0x18, 0x20, 0xcc, 0xc9, 0x47, 0xd5, 0x16, 0x04, 0x58, 0xcb, 0x10, 0x17, 0xcb, 0x13,
    0x17, 0x15, 0x20, 0xf6, 0xd1, 0x22, 0x23, 0x22, 0x23, 0xc9, 0x3e, 0x19, 0xea, 0x10, 0x99, 0x21,
    0x2f, 0x99, 0x0e, 0x0c, 0x3d, 0x28, 0x08, 0x32, 0x0d, 0x20, 0xf9, 0x2e, 0x0f, 0x18, 0xf3, 0xc9,
    0x3e, 0x01, 0xe0, 0x4f, 0xcd, 0x00, 0x02, 0x11, 0x07, 0x06, 0x21, 0x80, 0x80, 0x0e, 0xc0, 0x1a,
    0x22, 0x23, 0x22, 0x23, 0x13, 0x0d, 0x20, 0xf7, 0x11, 0x04, 0x01, 0xcd, 0x8f, 0x03, 0x01, 0xa8,
    0xff, 0x09, 0xcd, 0x8f, 0x03, 0x01, 0xf8, 0xff, 0x09, 0x11, 0x72, 0x00, 0x0e, 0x08, 0x23, 0x1a,
    0x22, 0x13, 0x0d, 0x20, 0xf9, 0x21, 0xc2, 0x98, 0x06, 0x08, 0x3e, 0x08, 0x0e, 0x10, 0x22, 0x0d,
    0x20, 0xfc, 0x11, 0x10, 0x00, 0x19, 0x05, 0x20, 0xf3, 0xaf, 0xe0, 0x4f, 0x21, 0xc2, 0x98, 0x3e,
    0x08, 0x22, 0x3c, 0xfe, 0x18, 0x20, 0x02, 0x2e, 0xe2, 0xfe, 0x28, 0x20, 0x03, 0x21, 0x02, 0x99,
    0xfe, 0x38, 0x20, 0xed, 0x21, 0xd8, 0x08, 0x11, 0x40, 0xd8, 0x06, 0x08, 0x3e, 0xff, 0x12, 0x13,
    0x12, 0x13, 0x0e, 0x02, 0xcd, 0x0a, 0x02, 0x3e, 0x00, 0x12, 0x13, 0x12, 0x13, 0x13, 0x13, 0x05,
    0x20, 0xea, 0xcd, 0x62, 0x02, 0x21, 0x4b, 0x01, 0x7e, 0xfe, 0x33, 0x20, 0x0b, 0x2e, 0x44, 0x1e,
    0x30, 0x2a, 0xbb, 0x20, 0x49, 0x1c, 0x18, 0x04, 0x2e, 0x4b, 0x1e, 0x01, 0x2a, 0xbb, 0x20, 0x3e,
    0x2e, 0x34, 0x01, 0x10, 0x00, 0x2a, 0x80, 0x47, 0x0d, 0x20, 0xfa, 0xea, 0x00, 0xd0, 0x21, 0xc7,
    0x06, 0x0e, 0x00, 0x2a, 0xb8, 0x28, 0x08, 0x0c, 0x79, 0xfe, 0x4f, 0x20, 0xf6, 0x18, 0x1f, 0x79,
    0xd6, 0x41, 0x38, 0x1c, 0x21, 0x16, 0x07, 0x16, 0x00, 0x5f, 0x19, 0xfa, 0x37, 0x01, 0x57, 0x7e,
    0xba, 0x28, 0x0d, 0x11, 0x0e, 0x00, 0x19, 0x79, 0x83, 0x4f, 0xd6, 0x5e, 0x38, 0xed, 0x0e, 0x00,
    0x21, 0x33, 0x07, 0x06, 0x00, 0x09, 0x7e, 0xe6, 0x1f, 0xea, 0x08, 0xd0, 0x7e, 0xe6, 0xe0, 0x07,
    0x07, 0x07, 0xea, 0x0b, 0xd0, 0xcd, 0xe9, 0x04, 0xc9, 0x11, 0x91, 0x07, 0x21, 0x00, 0xd9, 0xfa,
    0x0b, 0xd0, 0x47, 0x0e, 0x1e, 0xcb, 0x40, 0x20, 0x02, 0x13, 0x13, 0x1a, 0x22, 0x20, 0x02, 0x1b,
    0x1b, 0xcb, 0x48, 0x20, 0x02, 0x13, 0x13, 0x1a, 0x22, 0x13, 0x13, 0x20, 0x02, 0x1b, 0x1b, 0xcb,
    0x50, 0x28, 0x05, 0x1b, 0x2b, 0x1a, 0x22, 0x13, 0x1a, 0x22, 0x13, 0x0d, 0x20, 0xd7, 0x21, 0x00,
    0xd9, 0x11, 0x00, 0xda, 0xcd, 0x64, 0x05, 0xc9, 0x21, 0x12, 0x00, 0xfa, 0x05, 0xd0, 0x07, 0x07,
    0x06, 0x00, 0x4f, 0x09, 0x11, 0x40, 0xd8, 0x06, 0x08, 0xe5, 0x0e, 0x02, 0xcd, 0x0a, 0x02, 0x13,
    0x13, 0x13, 0x13, 0x13, 0x13, 0xe1, 0x05, 0x20, 0xf0, 0x11, 0x42, 0xd8, 0x0e, 0x02, 0xcd, 0x0a,
    0x02, 0x11, 0x4a, 0xd8, 0x0e, 0x02, 0xcd, 0x0a, 0x02, 0x2b, 0x2b, 0x11, 0x44, 0xd8, 0x0e, 0x02,
    0xcd, 0x0a, 0x02, 0xc9, 0x0e, 0x60, 0x2a, 0xe5, 0xc5, 0x21, 0xe8, 0x07, 0x06, 0x00, 0x4f, 0x09,
    0x0e, 0x08, 0xcd, 0x0a, 0x02, 0xc1, 0xe1, 0x0d, 0x20, 0xec, 0xc9, 0xfa, 0x08, 0xd0, 0x11, 0x18,
    0x00, 0x3c, 0x3d, 0x28, 0x03, 0x19, 0x20, 0xfa, 0xc9, 0xcd, 0x1d, 0x02, 0x78, 0xe6, 0xff, 0x28,
    0x0f, 0x21, 0xe4, 0x08, 0x06, 0x00, 0x2a, 0xb9, 0x28, 0x08, 0x04, 0x78, 0xfe, 0x0c, 0x20, 0xf6,
    0x18, 0x2d, 0x78, 0xea, 0x05, 0xd0, 0x3e, 0x1e, 0xea, 0x02, 0xd0, 0x11, 0x0b, 0x00, 0x19, 0x56,
    0x7a, 0xe6, 0x1f, 0x5f, 0x21, 0x08, 0xd0, 0x3a, 0x22, 0x7b, 0x77, 0x7a, 0xe6, 0xe0, 0x07, 0x07,
    0x07, 0x5f, 0x21, 0x0b, 0xd0, 0x3a, 0x22, 0x7b, 0x77, 0xcd, 0xe9, 0x04, 0xcd, 0x28, 0x05, 0xc9,
    0xcd, 0x11, 0x02, 0xfa, 0x43, 0x01, 0xcb, 0x7f, 0x28, 0x04, 0xe0, 0x4c, 0x18, 0x28, 0x3e, 0x04,
    0xe0, 0x4c, 0x3e, 0x01, 0xe0, 0x6c, 0x21, 0x00, 0xda, 0xcd, 0x7b, 0x05, 0x06, 0x10, 0x16, 0x00,
    0x1e, 0x08, 0xcd, 0x4a, 0x02, 0x21, 0x7a, 0x00, 0xfa, 0x00, 0xd0, 0x47, 0x0e, 0x02, 0x2a, 0xb8,
    0xcc, 0xda, 0x03, 0x0d, 0x20, 0xf8, 0xc9, 0x01, 0x0f, 0x3f, 0x7e, 0xff, 0xff, 0xc0, 0x00, 0xc0,
    0xf0, 0xf1, 0x03, 0x7c, 0xfc, 0xfe, 0xfe, 0x03, 0x07, 0x07, 0x0f, 0xe0, 0xe0, 0xf0, 0xf0, 0x1e,
    0x3e, 0x7e, 0xfe, 0x0f, 0x0f, 0x1f, 0x1f, 0xff, 0xff, 0x00, 0x00, 0x01, 0x01, 0x01, 0x03, 0xff,
    0xff, 0xe1, 0xe0, 0xc0, 0xf0, 0xf9, 0xfb, 0x1f, 0x7f, 0xf8, 0xe0, 0xf3, 0xfd, 0x3e, 0x1e, 0xe0,
    0xf0, 0xf9, 0x7f, 0x3e, 0x7c, 0xf8, 0xe0, 0xf8, 0xf0, 0xf0, 0xf8, 0x00, 0x00, 0x7f, 0x7f, 0x07,
    0x0f, 0x9f, 0xbf, 0x9e, 0x1f, 0xff, 0xff, 0x0f, 0x1e, 0x3e, 0x3c, 0xf1, 0xfb, 0x7f, 0x7f, 0xfe,
    0xde, 0xdf, 0x9f, 0x1f, 0x3f, 0x3e, 0x3c, 0xf8, 0xf8, 0x00, 0x00, 0x03, 0x03, 0x07, 0x07, 0xff,
    0xff, 0xc1, 0xc0, 0xf3, 0xe7, 0xf7, 0xf3, 0xc0, 0xc0, 0xc0, 0xc0, 0x1f, 0x1f, 0x1e, 0x3e, 0x3f,
    0x1f, 0x3e, 0x3e, 0x80, 0x00, 0x00, 0x00, 0x7c, 0x1f, 0x07, 0x00, 0x0f, 0xff, 0xfe, 0x00, 0x7c,
    0xf8, 0xf0, 0x00, 0x1f, 0x0f, 0x0f, 0x00, 0x7c, 0xf8, 0xf8, 0x00, 0x3f, 0x3e, 0x1c, 0x00, 0x0f,
    0x0f, 0x0f, 0x00, 0x7c, 0xff, 0xff, 0x00, 0x00, 0xf8, 0xf8, 0x00, 0x07, 0x0f, 0x0f, 0x00, 0x81,
    0xff, 0xff, 0x00, 0xf3, 0xe1, 0x80, 0x00, 0xe0, 0xff, 0x7f, 0x00, 0xfc, 0xf0, 0xc0, 0x00, 0x3e,
    0x7c, 0x7c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x88, 0x16, 0x36, 0xd1, 0xdb, 0xf2, 0x3c, 0x8c,
    0x92, 0x3d, 0x5c, 0x58, 0xc9, 0x3e, 0x70, 0x1d, 0x59, 0x69, 0x19, 0x35, 0xa8, 0x14, 0xaa, 0x75,
    0x95, 0x99, 0x34, 0x6f, 0x15, 0xff, 0x97, 0x4b, 0x90, 0x17, 0x10, 0x39, 0xf7, 0xf6, 0xa2, 0x49,
    0x4e, 0x43, 0x68, 0xe0, 0x8b, 0xf0, 0xce, 0x0c, 0x29, 0xe8, 0xb7, 0x86, 0x9a, 0x52, 0x01, 0x9d,
    0x71, 0x9c, 0xbd, 0x5d, 0x6d, 0x67, 0x3f, 0x6b, 0xb3, 0x46, 0x28, 0xa5, 0xc6, 0xd3, 0x27, 0x61,
    0x18, 0x66, 0x6a, 0xbf, 0x0d, 0xf4, 0x42, 0x45, 0x46, 0x41, 0x41, 0x52, 0x42, 0x45, 0x4b, 0x45,
    0x4b, 0x20, 0x52, 0x2d, 0x55, 0x52, 0x41, 0x52, 0x20, 0x49, 0x4e, 0x41, 0x49, 0x4c, 0x49, 0x43,
    0x45, 0x20, 0x52, 0x7c, 0x08, 0x12, 0xa3, 0xa2, 0x07, 0x87, 0x4b, 0x20, 0x12, 0x65, 0xa8, 0x16,
    0xa9, 0x86, 0xb1, 0x68, 0xa0, 0x87, 0x66, 0x12, 0xa1, 0x30, 0x3c, 0x12, 0x85, 0x12, 0x64, 0x1b,
    0x07, 0x06, 0x6f, 0x6e, 0x6e, 0xae, 0xaf, 0x6f, 0xb2, 0xaf, 0xb2, 0xa8, 0xab, 0x6f, 0xaf, 0x86,
    0xae, 0xa2, 0xa2, 0x12, 0xaf, 0x13, 0x12, 0xa1, 0x6e, 0xaf, 0xaf, 0xad, 0x06, 0x4c, 0x6e, 0xaf,
    0xaf, 0x12, 0x7c, 0xac, 0xa8, 0x6a, 0x6e, 0x13, 0xa0, 0x2d, 0xa8, 0x2b, 0xac, 0x64, 0xac, 0x6d,
    0x87, 0xbc, 0x60, 0xb4, 0x13, 0x72, 0x7c, 0xb5, 0xae, 0xae, 0x7c, 0x7c, 0x65, 0xa2, 0x6c, 0x64,
    0x85, 0x80, 0xb0, 0x40, 0x88, 0x20, 0x68, 0xde, 0x00, 0x70, 0xde, 0x20, 0x78, 0x20, 0x20, 0x38,
    0x20, 0xb0, 0x90, 0x20, 0xb0, 0xa0, 0xe0, 0xb0, 0xc0, 0x98, 0xb6, 0x48, 0x80, 0xe0, 0x50, 0x1e,
    0x1e, 0x58, 0x20, 0xb8, 0xe0, 0x88, 0xb0, 0x10, 0x20, 0x00, 0x10, 0x20, 0xe0, 0x18, 0xe0, 0x18,
    0x00, 0x18, 0xe0, 0x20, 0xa8, 0xe0, 0x20, 0x18, 0xe0, 0x00, 0x20, 0x18, 0xd8, 0xc8, 0x18, 0xe0,
    0x00, 0xe0, 0x40, 0x28, 0x28, 0x28, 0x18, 0xe0, 0x60, 0x20, 0x18, 0xe0, 0x00, 0x00, 0x08, 0xe0,
    0x18, 0x30, 0xd0, 0xd0, 0xd0, 0x20, 0xe0, 0xe8, 0xff, 0x7f, 0xbf, 0x32, 0xd0, 0x00, 0x00, 0x00,
    0x9f, 0x63, 0x79, 0x42, 0xb0, 0x15, 0xcb, 0x04, 0xff, 0x7f, 0x31, 0x6e, 0x4a, 0x45, 0x00, 0x00,
    0xff, 0x7f, 0xef, 0x1b, 0x00, 0x02, 0x00, 0x00, 0xff, 0x7f, 0x1f, 0x42, 0xf2, 0x1c, 0x00, 0x00,
    0xff, 0x7f, 0x94, 0x52, 0x4a, 0x29, 0x00, 0x00, 0xff, 0x7f, 0xff, 0x03, 0x2f, 0x01, 0x00, 0x00,
    0xff, 0x7f, 0xef, 0x03, 0xd6, 0x01, 0x00, 0x00, 0xff, 0x7f, 0xb5, 0x42, 0xc8, 0x3d, 0x00, 0x00,
    0x74, 0x7e, 0xff, 0x03, 0x80, 0x01, 0x00, 0x00, 0xff, 0x67, 0xac, 0x77, 0x13, 0x1a, 0x6b, 0x2d,
    0xd6, 0x7e, 0xff, 0x4b, 0x75, 0x21, 0x00, 0x00, 0xff, 0x53, 0x5f, 0x4a, 0x52, 0x7e, 0x00, 0x00,
    0xff, 0x4f, 0xd2, 0x7e, 0x4c, 0x3a, 0xe0, 0x1c, 0xed, 0x03, 0xff, 0x7f, 0x5f, 0x25, 0x00, 0x00,
    0x6a, 0x03, 0x1f, 0x02, 0xff, 0x03, 0xff, 0x7f, 0xff, 0x7f, 0xdf, 0x01, 0x12, 0x01, 0x00, 0x00,
    0x1f, 0x23, 0x5f, 0x03, 0xf2, 0x00, 0x09, 0x00, 0xff, 0x7f, 0xea, 0x03, 0x1f, 0x01, 0x00, 0x00,
    0x9f, 0x29, 0x1a, 0x00, 0x0c, 0x00, 0x00, 0x00, 0xff, 0x7f, 0x7f, 0x02, 0x1f, 0x00, 0x00, 0x00,
    0xff, 0x7f, 0xe0, 0x03, 0x06, 0x02, 0x20, 0x01, 0xff, 0x7f, 0xeb, 0x7e, 0x1f, 0x00, 0x00, 0x7c,
    0xff, 0x7f, 0xff, 0x3f, 0x00, 0x7e, 0x1f, 0x00, 0xff, 0x7f, 0xff, 0x03, 0x1f, 0x00, 0x00, 0x00,
    0xff, 0x03, 0x1f, 0x00, 0x0c, 0x00, 0x00, 0x00, 0xff, 0x7f, 0x3f, 0x03, 0x93, 0x01, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x42, 0x7f, 0x03, 0xff, 0x7f, 0xff, 0x7f, 0x8c, 0x7e, 0x00, 0x7c, 0x00, 0x00,
    0xff, 0x7f, 0xef, 0x1b, 0x80, 0x61, 0x00, 0x00, 0xff, 0x7f, 0x00, 0x7c, 0xe0, 0x03, 0x1f, 0x7c,
    0x1f, 0x00, 0xff, 0x03, 0x40, 0x41, 0x42, 0x20, 0x21, 0x22, 0x80, 0x81, 0x82, 0x10, 0x11, 0x12,
    0x12, 0xb0, 0x79, 0xb8, 0xad, 0x16, 0x17, 0x07, 0xba, 0x05, 0x7c, 0x13, 0x00, 0x00, 0x00, 0x00,
];
