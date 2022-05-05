mod cpu;
mod display;
mod events;
mod memory;

use cpu::Cpu;
use display::Display;
use events::EventDriver;
use memory::Memory;

use sdl2::{event::Event, keyboard::Keycode, Sdl};

use std::{env, io, thread, time::Duration};

const SCALE_FACTOR: u32 = 20;
const WIDTH: u32 = 1280; // 64 * SCALE_FACTOR
const HEIGHT: u32 = 640; // 32 * SCALE_FACTOR

struct Chip8 {
    display: Display,
    cpu: Cpu,
    memory: Memory,
    event: EventDriver,
}

impl Chip8 {
    fn new(sdl_context: &Sdl) -> Self {
        Self {
            display: Display::new(sdl_context, WIDTH, HEIGHT),
            cpu: Cpu::new(),
            memory: Memory::new(),
            event: EventDriver::new(sdl_context),
        }
    }

    fn fetch_opcode(&self) -> u16 {
        (self.memory.ram[self.cpu.pc] as u16) << 8 | (self.memory.ram[self.cpu.pc + 1] as u16)
    }

    fn next_instruction(&mut self) {
        self.cpu.pc += 2;
    }

    fn skip_next_instruction_if(&mut self, condition: bool) {
        if condition {
            self.cpu.pc += 4;
        } else {
            self.cpu.pc += 2;
        }
    }

    fn execute_opcode(&mut self, opcode: u16) {
        let nibbles = (
            (opcode & 0xF000) >> 12 as u8,
            (opcode & 0x0F00) >> 8 as u8,
            (opcode & 0x00F0) >> 4 as u8,
            (opcode & 0x000F) as u8,
        );

        let x = nibbles.1 as usize; 
        let y = nibbles.2 as usize;
        let n = nibbles.3 as usize;
        let kk = (opcode & 0x00FF) as u8;
        let nnn = opcode & 0x0FFF;

        let vx = self.cpu.v[x as usize];
        let vy = self.cpu.v[y as usize];

        match nibbles {

            // 0nnn - SYS addr
            (0x0, _, _, _) => {
                println!("OPCODE: 0nnn");
                self.next_instruction();
            }

            // 00E0 - CLS
            (0x0, 0x0, 0xE, 0x0) => {
                println!("OPCODE: 00E0");
                self.display.canvas.clear();
                self.next_instruction();
            }

            // 00EE - RET
            (0x0, 0x0, 0xE, 0xE) => {
                println!("OPCODE: 00EE");
                self.cpu.pc = self.memory.stack[self.cpu.sp] as usize;
                self.cpu.sp -= 1;
                self.next_instruction();
            }

            // 1nnn - JP addr
            (0x1, _, _, _) => {
                println!("OPCODE: 1nnn");
                self.cpu.pc = nnn as usize;
            }
            
            // 2nnn - CALL addr
            (0x2, _, _, _) => {
                println!("OPCODE: 2nnn");
                self.cpu.sp += 1;
                self.memory.stack[self.cpu.sp] = self.cpu.pc as u16;
                self.cpu.pc = nnn as usize;
            }

            // 3xkk - SE Vx, byte
            (0x3, _, _, _) => {
                println!("OPCODE: 3xkk");
                self.skip_next_instruction_if(vx == kk);
            }

            // 4xkk - SNE Vx, byte
            (0x4, _, _, _) => {
                println!("OPCODE: 4xkk");
                self.skip_next_instruction_if(vx != kk);
            }

            // 5xkk - SE Vx, Vy
            (0x5, _, _, _) => {
                println!("OPCODE: 5xkk");
                self.skip_next_instruction_if(vx == vy);
            }

            // 6xkk - LD Vx, byte
            (0x6, _, _, _) => {
                println!("OPCODE: 5xkk");
                self.cpu.v[x as usize] = kk;
                self.next_instruction();
            }

            // 7xkk - ADD Vx, byte
            (0x7, _, _, _) => {
                println!("OPCODE: 7xkk");
                self.cpu.v[x as usize] += kk;
                self.next_instruction();
            }

            // 8xy0 - LD Vx, Vy
            (0x8, _, _, 0) => {
                println!("OPCODE: 8xy0");
                self.cpu.v[x] = self.cpu.v[y];
                self.next_instruction();
            }

            // 8xy1 - OR Vx, Vy
            (0x8, _, _, 0x1) => {
                println!("OPCODE: 8xy1");
                self.cpu.v[x] |= self.cpu.v[y];
                self.next_instruction();
            }

            // 8xy2 - AND Vx, Vy
            (0x8, _, _, 0x2) => {
                println!("OPCODE: 8xy2");
                self.cpu.v[x] &= self.cpu.v[y];
                self.next_instruction();
            }

            // 8xy3 - XOR Vx, Vy
            (0x8, _, _, 0x3) => {
                println!("OPCODE: 8xy3");
                self.cpu.v[x] ^= self.cpu.v[y];
                self.next_instruction();
            }

            // 8xy4 - ADD Vx, Vy
            (0x8, _, _, 0x4) => {
                println!("OPCODE: 8xy4");
                let result = vx + vy;
                self.cpu.v[x] = result as u8;
                self.cpu.v[0xF] = if result > 0xFF { 1 } else { 0 };
                self.next_instruction();
            }

            // 8xy5 - SUB Vx, Vy
            (0x8, _, _, 0x5) => {
                println!("OPCODE: 8xy5");
                let result = vx - vy;
                self.cpu.v[x] = result;
                self.cpu.v[0xF] = if vx > vy { 1 } else { 0 };
                self.next_instruction();
            }

            // 8xy6 - SHR Vx {, Vy}
            (0x8, _, _, 0x6) {
                println!("OPCODE: 8xy6");
                self.cpu.v[0xF] = self.cpu.v[x] & 1;
                self.cpu.v[x] >>= 1;
                self.next_instruction();
            }

            _ => {
                println!("Invalid opcode: {}", opcode);
            }
        }
    }

    fn execute_cycle(&mut self) {
        let mut opcode = self.fetch_opcode();
        println!("CURRENT OPCODE: {:#06X}", opcode);
        self.execute_opcode(opcode);
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Error: No input file");
        std::process::exit(1);
    }

    let sdl_context = sdl2::init().unwrap();
    let mut chip8 = Chip8::new(&sdl_context);

    let rom_path = &args[1];
    chip8.memory.load_rom(rom_path);
    chip8.memory.load_fontset();

    'running: loop {
        chip8.display.canvas.clear();
        let events = chip8.event.events.poll_iter();
        for event in events {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        chip8.execute_cycle();
        chip8.display.canvas.present();
    }
    Ok(())
}
