use std::{fs::File, io, io::Read};

const KEYBOARD: [[u8; 5]; 16] = [
    [0xF0, 0x90, 0x90, 0x90, 0xF0], // 0
    [0x20, 0x60, 0x20, 0x20, 0x70], // 1
    [0xF0, 0x10, 0xF0, 0x80, 0xF0], // 2
    [0xF0, 0x10, 0xF0, 0x10, 0xF0], // 3
    [0x90, 0x90, 0xF0, 0x10, 0x10], // 4
    [0xF0, 0x80, 0xF0, 0x10, 0xF0], // 5
    [0xF0, 0x80, 0xF0, 0x90, 0xF0], // 6
    [0xF0, 0x10, 0x20, 0x40, 0x40], // 7
    [0xF0, 0x90, 0xF0, 0x90, 0xF0], // 8
    [0xF0, 0x90, 0xF0, 0x10, 0xF0], // 9
    [0xF0, 0x90, 0xF0, 0x90, 0x90], // A
    [0xE0, 0x90, 0xE0, 0x90, 0xE0], // B
    [0xF0, 0x80, 0x80, 0x80, 0xF0], // C
    [0xE0, 0x90, 0x90, 0x90, 0xE0], // D
    [0xF0, 0x80, 0xF0, 0x80, 0xF0], // E
    [0xF0, 0x80, 0xF0, 0x80, 0x80], // F
];

const STARTING_ADDRESS_ROM: usize = 0x200;
const STARTING_ADDRESS_FONT: usize = 0x50;

pub struct Memory {
    pub ram: Vec<u8>,
    pub stack: Vec<u16>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            ram: vec![0; 4096],
            stack: vec![0; 16],
        }
    }

    pub fn load_rom(&mut self, rom_path: &str) -> io::Result<()> {
        let mut rom = File::open(rom_path)?;
        let mut buffer = Vec::new();
        rom.read_to_end(&mut buffer)?;
        self.ram[STARTING_ADDRESS_ROM..(buffer.len() + STARTING_ADDRESS_ROM)]
            .clone_from_slice(&buffer[..]);
        Ok(())
    }

    pub fn load_fontset(&mut self) -> io::Result<()> {
        let mut i = 0;
        for sprite in KEYBOARD.iter() {
            for tile in sprite.iter() {
                self.ram[STARTING_ADDRESS_FONT + i] = *tile;
                i += 1;
            }
        }
        Ok(())
    }
}
