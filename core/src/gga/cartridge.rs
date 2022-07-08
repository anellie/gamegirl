// Unless otherwise noted, this file is released and thus subject to the
// terms of the Mozilla Public License Version 2.0 (MPL2). Also, it is
// "Incompatible With Secondary Licenses", as defined by the MPL2.
// If a copy of the MPL2 was not distributed with this file, you can
// obtain one at https://mozilla.org/MPL/2.0/.

use std::{cell::RefCell, iter};

use serde::{Deserialize, Serialize};
use FlashCmdStage::*;
use SaveType::*;

use crate::{gga::memory::KB, numutil::NumExt, storage::GameSave};

// Both Macronix.
const FLASH64_ID: [u8; 2] = [0xC2, 0x1C];
const FLASH128_ID: [u8; 2] = [0xC2, 0x09];

#[derive(Default, Deserialize, Serialize)]
pub struct Cartridge {
    #[serde(skip)]
    #[serde(default)]
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,
    pub save_type: SaveType,
}

impl Cartridge {
    pub fn read_ram_byte(&self, addr: usize) -> u8 {
        match &self.save_type {
            Flash64(state) if state.mode == FlashMode::Id => FLASH64_ID[addr & 1],
            Flash128 { state, .. } if state.mode == FlashMode::Id => FLASH128_ID[addr & 1],

            Flash128 { bank: 1, .. } => self.ram[addr | 0x10000],
            Flash64(_) | Flash128 { .. } => self.ram[addr],
            Sram => self.ram[addr & 0x7FFF],

            _ => 0xFF,
        }
    }

    pub fn read_ram_hword(&self) -> u16 {
        match &self.save_type {
            Eeprom(eeprom) => eeprom.read(),
            _ => 0,
        }
    }

    pub fn write_ram_byte(&mut self, addr: usize, value: u8) {
        match &mut self.save_type {
            Flash64(state) => state.write(addr, value, &mut self.ram, None),
            Flash128 { state, bank } => state.write(addr, value, &mut self.ram, Some(bank)),
            Sram => self.ram[addr & 0x7FFF] = value,
            _ => (),
        }
    }

    pub fn write_ram_hword(&mut self, value: u16) {
        if let Eeprom(eeprom) = &mut self.save_type {
            eeprom.write(value, &mut self.ram);
        }
    }

    pub fn is_eeprom_at(&self, addr: u32) -> bool {
        matches!(self.save_type, SaveType::Eeprom(_))
            && (self.rom.len() <= 16 * (KB * KB) || addr >= 0x0DFF_FF00)
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        self.rom = rom;
        self.save_type = self.detect_save();

        let ff_iter = iter::repeat(0xFF);
        let len = self.ram.len();
        match self.save_type {
            Nothing => {}
            Eeprom(_) => self.ram.extend(ff_iter.take((8 * KB) - len)),
            Sram => self.ram.extend(ff_iter.take((32 * KB) - len)),
            Flash64(_) => self.ram.extend(ff_iter.take((64 * KB) - len)),
            Flash128 { .. } => self.ram.extend(ff_iter.take((128 * KB) - len)),
        }
    }

    pub fn make_save(&self) -> Option<GameSave> {
        match self.save_type {
            Nothing => None,
            _ => Some(GameSave {
                ram: self.ram.clone(),
                rtc: None,
                title: self.title(),
            }),
        }
    }

    pub fn load_save(&mut self, save: GameSave) {
        self.ram = save.ram;
    }

    pub fn title(&self) -> String {
        self.read_string(0x0A0, 12)
    }

    pub fn game_code(&self) -> String {
        self.read_string(0x0AC, 4)
    }

    fn detect_save(&self) -> SaveType {
        // This is not efficient
        let save_types: [(SaveType, &str); 5] = [
            (
                Flash128 {
                    state: FlashState::new(),
                    bank: 0,
                },
                "FLASH1M_V",
            ),
            (Flash64(FlashState::new()), "FLASH_V"),
            (Flash64(FlashState::new()), "FLASH512_V"),
            (Sram, "SRAM_V"),
            (Eeprom(Eeprom::new()), "EEPROM_V"),
        ];
        let self_str = String::from_utf8_lossy(&self.rom);
        for (ty, str) in save_types {
            if self_str.contains(str) {
                return ty;
            }
        }
        Nothing
    }

    fn read_string(&self, base: usize, max: usize) -> String {
        let mut buf = String::new();
        for idx in 0..max {
            let ch = self.rom[base + idx] as char;
            if ch == '\0' {
                break;
            }
            buf.push(ch);
        }
        buf
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SaveType {
    Nothing,
    Eeprom(Eeprom),
    Sram,
    Flash64(FlashState),
    Flash128 { state: FlashState, bank: u8 },
}

impl Default for SaveType {
    fn default() -> Self {
        Nothing
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Eeprom {
    size: EepromSize,
    command: EepromCmd,

    recv_buffer: u128,
    recv_count: u32,

    send_buffer: u128,
    send_count: RefCell<u32>,
}

impl Eeprom {
    pub fn read(&self) -> u16 {
        let mut count = self.send_count.borrow_mut();
        if *count == 0 {
            1
        } else {
            *count -= 1;
            (self.send_buffer >> *count) as u16 & 1
        }
    }

    pub fn write(&mut self, value: u16, ram: &mut [u8]) {
        let bit = value & 1;
        self.recv_buffer = (self.recv_buffer << 1) | bit as u128;
        self.recv_count += 1;

        if self.recv_count == 2 {
            self.command = match self.recv_buffer & 3 {
                0b11 => EepromCmd::Read,
                0b10 => EepromCmd::Write,
                _ => {
                    self.reset_rx();
                    EepromCmd::Nothing
                }
            };
        } else if self.recv_count == self.cmd_size(self.command) {
            self.recv_buffer >>= 1; // Shift out the 0 bit
            match self.command {
                EepromCmd::Nothing => panic!("invalid command!"),

                EepromCmd::Read => {
                    let recv_buffer = self.recv_buffer as u32;
                    let addr = recv_buffer.bits(0, self.size as u32) & 0x3FF;
                    let idx = addr.us() << 3; // Addressing is in 64bits, not 8bits
                    for byte in ram.iter_mut().skip(idx).take(8) {
                        self.send_buffer <<= 8;
                        self.send_buffer |= *byte as u128;
                    }
                    *self.send_count.borrow_mut() = 68; // 4 dummy bits
                }

                EepromCmd::Write => {
                    let mut data = self.recv_buffer as u64;
                    let addr = ((self.recv_buffer >> 64) as u32).bits(0, self.size as u32) & 0x3FF;
                    let idx = addr.us() << 3; // Addressing is in 64bits, not 8bits
                    for byte in ram.iter_mut().skip(idx).take(8).rev() {
                        *byte = data as u8;
                        data >>= 8;
                    }
                    // We want to send 1's, which indicate the operation is done.
                    self.send_buffer = u128::MAX;
                    *self.send_count.borrow_mut() = 128;
                }
            }
            self.reset_rx();
        }
    }

    pub fn dma3_started(&mut self, dst: u32, cnt: u32) {
        // Try to detect EEPROM size
        if self.size == EepromSize::Unknown && (0xD00_0000..=0xDFF_FFFF).contains(&dst) {
            self.size = match cnt {
                9 | 73 => EepromSize::E512,
                17 | 81 => EepromSize::E8k,
                _ => EepromSize::Unknown,
            }
        }
        // Reset state, just in case
        self.reset_rx();
    }

    fn reset_rx(&mut self) {
        self.recv_count = 0;
        self.recv_buffer = 0;
    }

    fn cmd_size(&self, cmd: EepromCmd) -> u32 {
        2 + self.size as u32 + cmd as u32
    }

    const fn new() -> Self {
        Self {
            size: EepromSize::Unknown,
            command: EepromCmd::Nothing,
            recv_buffer: 0,
            recv_count: 0,
            send_buffer: 0,
            send_count: RefCell::new(0),
        }
    }
}

/// Eeprom size. Integer equals address size.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum EepromSize {
    Unknown = 99,
    E512 = 6,
    E8k = 14,
}

/// Eeprom commands. Size equals length of the command, minus
/// the 2 bits of the command itself and the address.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum EepromCmd {
    Nothing = 0,
    Read = 1,
    Write = 65,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct FlashState {
    command_stage: Option<FlashCmdStage>,
    mode: FlashMode,
}

impl FlashState {
    fn write(&mut self, addr: usize, value: u8, ram: &mut [u8], bank: Option<&mut u8>) {
        match (addr, value, self.command_stage) {
            (0x0, _, _) if self.mode == FlashMode::BankSelect => {
                self.mode = FlashMode::Regular;
                *bank.unwrap() = value & 1;
            }

            (_, _, _) if self.mode == FlashMode::Write => {
                self.mode = FlashMode::Regular;
                if bank.cloned() == Some(1) {
                    ram[addr | 0x10000] = value;
                } else {
                    ram[addr] = value;
                }
            }

            (0x5555, 0xAA, None) => self.command_stage = Some(FirstWritten),
            (0x2AAA, 0x55, Some(FirstWritten)) => self.command_stage = Some(SecondWritten),

            // Erase 4K sector
            (_, 0x30, Some(SecondWritten)) => {
                if self.mode == FlashMode::Erase {
                    let addr = if bank.cloned() == Some(1) {
                        (addr & 0xF000) | 0x10000
                    } else {
                        addr & 0xF000
                    };
                    for mem in ram.iter_mut().skip(addr).take(0x1000) {
                        *mem = 0xFF;
                    }
                }
                self.mode = FlashMode::Regular;
                self.command_stage = None;
            }

            (0x5555, _, Some(SecondWritten)) => {
                match value {
                    // Enter Erase mode
                    0x80 => self.mode = FlashMode::Erase,
                    // Erase entire chip
                    0x10 => {
                        if self.mode == FlashMode::Erase {
                            for mem in ram {
                                *mem = 0xFF;
                            }
                        }
                        self.mode = FlashMode::Regular;
                    }

                    // Enter write mode
                    0xA0 => self.mode = FlashMode::Write,
                    // Enter bank select, if banked chip
                    0xB0 if bank.is_some() => self.mode = FlashMode::BankSelect,

                    // Enter ID mode
                    0x90 => self.mode = FlashMode::Id,
                    // Exit ID mode
                    0xF0 => self.mode = FlashMode::Regular,

                    _ => (),
                }
                self.command_stage = None;
            }

            _ => (),
        }
    }

    const fn new() -> Self {
        // Why is Default not const...
        Self {
            command_stage: None,
            mode: FlashMode::Regular,
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum FlashCmdStage {
    FirstWritten,
    SecondWritten,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum FlashMode {
    Regular,
    Write,
    Id,
    Erase,
    BankSelect,
}
