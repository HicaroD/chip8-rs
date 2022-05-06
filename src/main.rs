mod cpu;
mod display;
mod events;
mod memory;

use cpu::Cpu;
use display::Display;
use events::EventDriver;
use memory::Memory;

use sdl2::{Sdl, rect, pixels};
use rand::{thread_rng, Rng};

use std::{env, io};

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

    fn draw(&mut self, x: i32, y: i32, color: u8) {
        let get_color = |color: u8| -> pixels::Color {
            if color == 1 {
                pixels::Color::WHITE
            } else { 
                pixels::Color::BLACK
            }
        };

        self.display.canvas.set_draw_color(get_color(color));
        let point = rect::Rect::new(x, y, SCALE_FACTOR, SCALE_FACTOR);
        let _ = self.display.canvas.fill_rect(point);
        self.display.canvas.present();
    }

    fn execute_opcode(&mut self, opcode: u16) {
        let nibbles = (
            (opcode & 0xF000) >> 12_u8,
            (opcode & 0x0F00) >> 8_u8,
            (opcode & 0x00F0) >> 4_u8,
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
                self.cpu.v[x] = kk;
                self.next_instruction();
            }

            // 7xkk - ADD Vx, byte
            (0x7, _, _, _) => {
                println!("OPCODE: 7xkk");
                self.cpu.v[x] += kk;
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
                self.cpu.v[0x0F] = if result > 0xFF { 1 } else { 0 };
                self.cpu.v[x] = result as u8;
                self.next_instruction();
            }

            // 8xy5 - SUB Vx, Vy
            (0x8, _, _, 0x5) => {
                println!("OPCODE: 8xy5");
                self.cpu.v[0x0F] = if vx > vy { 1 } else { 0 };
                self.cpu.v[x] -= self.cpu.v[y];
                self.next_instruction();
            }

            // 8xy6 - SHR Vx {, Vy}
            (0x8, _, _, 0x6) => {
                println!("OPCODE: 8xy6");
                self.cpu.v[0x0F] = self.cpu.v[x] & 1;
                self.cpu.v[x] >>= 1;
                self.next_instruction();
            }

            // 8xy7 - SUBN Vx, Vy
            (0x8, _, _, 0x7) => {
                println!("OPCODE: 8xy7");
                self.cpu.v[x] = if vy > vx { 1 } else { 0 };
                self.cpu.v[x] = self.cpu.v[y] - self.cpu.v[x];
                self.next_instruction();
            }

            // 8xyE - SHL Vx {, Vy}
            (0x8, _, _, 0xE) => {
                println!("OPCODE: 8xyE");
                self.cpu.v[0x0F] = self.cpu.v[x] & 0b10000000;
                self.cpu.v[x] <<= 1;
                self.next_instruction();
            }

            // 9xy0 - SNE Vx, Vy
            (0x9, _, _, 0) => {
                println!("OPCODE: 9xy0");
                self.skip_next_instruction_if(vx != vy);
            }

            // Annn - LD I, addr
            (0xA, _, _, _) => {
                println!("OPCODE: Annn");
                self.cpu.i = nnn as usize;
                self.next_instruction();
            }

            // Bnnn - JP V0, addr
            (0xB, _, _, _) => {
                println!("OPCODE: Bnnn");
                self.cpu.pc = (nnn + self.cpu.v[0] as u16) as usize;
            }

            // Cxkk - RND Vx, byte
            (0xC, _, _, _) => {
                println!("OPCODE: Cxkk");
                let mut rng = thread_rng();
                let random_byte = rng.gen_range(0..255);
                self.cpu.v[x] = kk & random_byte;
                self.next_instruction();
            }

            // Dxyn - DRW Vx, Vy, nibble
            (0xD, _, _, _) => {
                println!("OPCODE: Dxyn");
                // TODO: Too much type casting
                for byte in 0..n {
                    let y_axis = (self.cpu.v[y] as usize + byte) % (HEIGHT as usize);
                    for bit in 0..8 {
                        let x_axis = (self.cpu.v[x] as usize + bit) % (WIDTH as usize);
                        let color = (self.memory.ram[self.cpu.i + byte] >> (7 - bit)) & 1;
                        self.draw(x_axis as i32, y_axis as i32, color);
                    }
                }
                self.next_instruction();
            }

            // TODO: Aprender sobre key press em SDL2
            // Ex9E - SKP Vx
            (0xE, _, 0x9, 0xE) => {}

            // TODO: Aprender sobre key press em SDL2
            // ExA1 - SKNP Vx
            (0xE, _, 0xA, 0x1) => {}

            // Fx07 - LD Vx, DT
            (0xF, _, 0, 0x7) => {
                println!("OPCODE: Fx07");
                self.cpu.v[x] = self.cpu.delay_timer;
                self.next_instruction();
            }

            // TODO: Aprender sobre key press em SDL2
            // Fx0A - LD Vx, K
            (0xF, _, 0, 0xA) => {}

            // Fx15 - LD DT, Vx
            (0xF, _, 0x1, 0x5) => {
                println!("OPCODE: Fx15");
                self.cpu.delay_timer = self.cpu.v[x];
                self.next_instruction();
            }

            // Fx18 - LD ST, Vx
            (0xF, _, 0x1, 0x8) => {
                println!("OPCODE: Fx18");
                self.cpu.sound_timer = self.cpu.v[x];
                self.next_instruction();
            }

            // Fx1E - ADD I, Vx
            (0xF, _, 0x1, 0xE) => {
                println!("OPCODE: Fx1E");
                self.cpu.i += self.cpu.v[x] as usize;
                self.next_instruction();
            }

            // Fx29 - LD F, Vx
            (0xF, _, 0x2, 0x9) => {
                println!("OPCODE: Fx29");
                self.cpu.i = self.cpu.v[x] as usize;
                self.next_instruction();
            }

            // Fx33 - LD B, Vx
            (0xF, _, 0x3, 0x3) => {
                println!("OPCODE: Fx33");
                self.memory.ram[self.cpu.i] = self.cpu.v[x] / 100;
                self.memory.ram[self.cpu.i+1] = (self.cpu.v[x] % 100) / 10;
                self.memory.ram[self.cpu.i+2] = (self.cpu.v[x] % 10) / 10;
                self.next_instruction();
            }

            // Fx55 - LD [I], Vx
            (0xF, _, 0x5, 0x5) => {
                println!("OPCODE: Fx55");
                for i in 0..=x {
                    self.memory.ram[i+self.cpu.i] = self.cpu.v[i];
                }
                self.next_instruction();
            }

            // Fx65 - LD Vx, [I]
            (0xF, _, 0x6, 0x5) => {
                println!("OPCODE: Fx65");
                for i in 0..=x {
                    self.cpu.v[i] = self.memory.ram[i+self.cpu.i];
                }
                self.next_instruction();
            }

            _ => {
                println!("Invalid opcode: {}", opcode);
            }
        }
    }

    fn execute_cycle(&mut self) {
        let opcode = self.fetch_opcode();
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

    if let Err(err) = chip8.memory.load_rom(rom_path) {
        eprintln!("Unable to load ROM into memory");
    }

    if let Err(err) = chip8.memory.load_fontset() {
        eprintln!("Load fontset into memory");
    }

    loop {
        chip8.display.canvas.clear();
        chip8.event.pool().unwrap();
        chip8.execute_cycle();
        chip8.display.canvas.present();
    }
}
