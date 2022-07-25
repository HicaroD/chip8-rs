mod cpu;
mod display;
mod events;
mod memory;

use cpu::Cpu;
use display::Display;
use events::EventDriver;
use memory::Memory;

use rand::{thread_rng, Rng};
use sdl2::{pixels, rect, Sdl};

use std::{env, io};

const SCALE_FACTOR: u32 = 20;
const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;
const DISPLAY_WIDTH: u32 = 64 * SCALE_FACTOR;
const DISPLAY_HEIGHT: u32 = 32 * SCALE_FACTOR;

enum ProgramCounter {
    Next,
    Skip,
    Jump(u16),
}

struct Chip8 {
    display: Display,
    screen: [[u8; WIDTH as usize]; HEIGHT as usize],
    event: EventDriver,
    cpu: Cpu,
    memory: Memory,
    keyboard: [bool; 16],
    screen_changed: bool,
}

impl Chip8 {
    fn new(sdl_context: &Sdl) -> Self {
        Self {
            display: Display::new(sdl_context, DISPLAY_WIDTH, DISPLAY_HEIGHT),
            screen: [[0; WIDTH as usize]; HEIGHT as usize],
            cpu: Cpu::new(),
            memory: Memory::new(),
            event: EventDriver::new(sdl_context),
            keyboard: [false; 16],
            screen_changed: false,
        }
    }

    fn fetch_opcode(&self) -> u16 {
        (self.memory.ram[self.cpu.pc as usize] as u16) << 8
            | (self.memory.ram[self.cpu.pc as usize + 1] as u16)
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

    fn draw(&mut self) {
        let get_color = |color: u8| -> pixels::Color {
            if color == 1 {
                pixels::Color::WHITE
            } else {
                pixels::Color::BLACK
            }
        };

        for (y, row) in self.screen.iter().enumerate() {
            for (x, color) in row.iter().enumerate() {
                self.display.canvas.set_draw_color(get_color(*color));
                let y_axis = (y * SCALE_FACTOR as usize).try_into().unwrap();
                let x_axis = (x * SCALE_FACTOR as usize).try_into().unwrap();

                let point = rect::Rect::new(x_axis, y_axis, SCALE_FACTOR, SCALE_FACTOR);
                let _ = self
                    .display
                    .canvas
                    .draw_rect(point)
                    .expect("This should work!");
            }
        }
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

        let program_counter = match nibbles {
            // 00E0 - CLS
            (0x0, 0x0, 0xE, 0x0) => {
                println!("OPCODE: 00E0");
                for y in 0..HEIGHT {
                    for x in 0..WIDTH {
                        self.screen[y as usize][x as usize] = 0;
                    }
                }
                self.screen_changed = true;
                ProgramCounter::Next
            }

            // 00EE - RET
            (0x0, 0x0, 0xE, 0xE) => {
                println!("OPCODE: 00EE");
                self.cpu.pc = self.memory.stack[self.cpu.sp as usize];
                self.cpu.sp -= 1;
                ProgramCounter::Next
            }

            // 1nnn - JP addr
            (0x1, _, _, _) => {
                println!("OPCODE: 1nnn");
                ProgramCounter::Jump(nnn)
            }

            // 2nnn - CALL addr
            (0x2, _, _, _) => {
                println!("OPCODE: 2nnn");
                self.cpu.sp += 1;
                self.memory.stack[self.cpu.sp as usize] = self.cpu.pc as u16;
                ProgramCounter::Jump(nnn)
            }

            // 3xkk - SE Vx, byte
            (0x3, _, _, _) => {
                println!("OPCODE: 3xkk");
                if vx == kk {
                    ProgramCounter::Skip
                } else {
                    ProgramCounter::Next
                }
            }

            // 4xkk - SNE Vx, byte
            (0x4, _, _, _) => {
                println!("OPCODE: 4xkk");
                if vx != kk {
                    ProgramCounter::Skip
                } else {
                    ProgramCounter::Next
                }
            }

            // 5xkk - SE Vx, Vy
            (0x5, _, _, _) => {
                println!("OPCODE: 5xkk");
                if vx == vy {
                    ProgramCounter::Skip
                } else {
                    ProgramCounter::Next
                }
            }

            // 6xkk - LD Vx, byte
            (0x6, _, _, _) => {
                println!("OPCODE: 5xkk");
                self.cpu.v[x] = kk;
                ProgramCounter::Next
            }

            // 7xkk - ADD Vx, byte
            (0x7, _, _, _) => {
                println!("OPCODE: 7xkk");
                let result = vx as u16 + kk as u16;
                self.cpu.v[x] = result as u8;
                ProgramCounter::Next
            }

            // 8xy0 - LD Vx, Vy
            (0x8, _, _, 0) => {
                println!("OPCODE: 8xy0");
                self.cpu.v[x] = self.cpu.v[y];
                ProgramCounter::Next
            }

            // 8xy1 - OR Vx, Vy
            (0x8, _, _, 0x1) => {
                println!("OPCODE: 8xy1");
                self.cpu.v[x] |= self.cpu.v[y];
                ProgramCounter::Next
            }

            // 8xy2 - AND Vx, Vy
            (0x8, _, _, 0x2) => {
                println!("OPCODE: 8xy2");
                self.cpu.v[x] &= self.cpu.v[y];
                ProgramCounter::Next
            }

            // 8xy3 - XOR Vx, Vy
            (0x8, _, _, 0x3) => {
                println!("OPCODE: 8xy3");
                self.cpu.v[x] ^= self.cpu.v[y];
                ProgramCounter::Next
            }

            // 8xy4 - ADD Vx, Vy
            (0x8, _, _, 0x4) => {
                println!("OPCODE: 8xy4");
                let result = vx as u16 + vy as u16;
                self.cpu.v[0x0F] = if result > 0xFF { 1 } else { 0 };
                self.cpu.v[x] = result as u8;
                ProgramCounter::Next
            }

            // 8xy5 - SUB Vx, Vy
            (0x8, _, _, 0x5) => {
                println!("OPCODE: 8xy5");
                self.cpu.v[0x0F] = if vx > vy { 1 } else { 0 };
                self.cpu.v[x] -= self.cpu.v[y];
                ProgramCounter::Next
            }

            // 8xy6 - SHR Vx {, Vy}
            (0x8, _, _, 0x6) => {
                println!("OPCODE: 8xy6");
                self.cpu.v[0x0F] = self.cpu.v[x] & 1;
                self.cpu.v[x] >>= 1;
                ProgramCounter::Next
            }

            // 8xy7 - SUBN Vx, Vy
            (0x8, _, _, 0x7) => {
                println!("OPCODE: 8xy7");
                self.cpu.v[x] = if vy > vx { 1 } else { 0 };
                self.cpu.v[x] = self.cpu.v[y] - self.cpu.v[x];
                ProgramCounter::Next
            }

            // 8xyE - SHL Vx {, Vy}
            (0x8, _, _, 0xE) => {
                println!("OPCODE: 8xyE");
                self.cpu.v[0x0F] = self.cpu.v[x] & 0b10000000;
                self.cpu.v[x] <<= 1;
                ProgramCounter::Next
            }

            // 9xy0 - SNE Vx, Vy
            (0x9, _, _, 0) => {
                println!("OPCODE: 9xy0");
                if vx != vy {
                    ProgramCounter::Skip
                } else {
                    ProgramCounter::Next
                }
            }

            // Annn - LD I, addr
            (0xA, _, _, _) => {
                println!("OPCODE: Annn");
                self.cpu.i = nnn;
                ProgramCounter::Next
            }

            // Bnnn - JP V0, addr
            (0xB, _, _, _) => {
                println!("OPCODE: Bnnn");
                ProgramCounter::Jump(nnn + self.cpu.v[0] as u16)
            }

            // Cxkk - RND Vx, byte
            (0xC, _, _, _) => {
                println!("OPCODE: Cxkk");
                let mut rng = thread_rng();
                let random_byte = rng.gen_range(0..255);
                self.cpu.v[x] = kk & random_byte;
                ProgramCounter::Next
            }

            // Dxyn - DRW Vx, Vy, nibble
            (0xD, _, _, _) => {
                println!("OPCODE: Dxyn");
                for byte in 0..=n {
                    let y_axis = (self.cpu.v[y] as usize + byte) % HEIGHT as usize;
                    for bit in 0..8 {
                        let x_axis = (self.cpu.v[x] as usize + bit) % WIDTH as usize;
                        let color = (self.memory.ram[self.cpu.i as usize + byte] >> (7 - bit)) & 1;
                        self.cpu.v[0xf] |= color & self.screen[y_axis][x_axis];
                        self.screen[y_axis][x_axis] ^= color;
                    }
                }
                self.screen_changed = true;
                ProgramCounter::Next
            }

            // Ex9E - SKP Vx
            (0xE, _, 0x9, 0xE) => {
                println!("OPCODE: Ex9E");
                let key = self.event.get_key(vx);
                let mut pc = ProgramCounter::Next;

                if let Some(selected_key) = key {
                    if self.event.events.keyboard_state().is_scancode_pressed(selected_key) {
                        pc = ProgramCounter::Skip;
                    }
                }
                pc
            }

            // ExA1 - SKNP Vx
            (0xE, _, 0xA, 0x1) => {
                println!("OPCODE: ExA1");
                let key = self.event.get_key(vx);
                let mut pc = ProgramCounter::Next;

                if let Some(selected_key) = key {
                    if !self.event.events.keyboard_state().is_scancode_pressed(selected_key) {
                        pc = ProgramCounter::Skip
                    } 
                }
                pc
            }

            // Fx07 - LD Vx, DT
            (0xF, _, 0, 0x7) => {
                println!("OPCODE: Fx07");
                self.cpu.v[x] = self.cpu.delay_timer;
                ProgramCounter::Next
            }

            // Fx0A - LD Vx, K
            (0xF, _, 0, 0xA) => {
                println!("OPCODE: Fx0A");

                'keypress_waiting: loop {
                    println!("WAITING FOR KEY PRESS");
                    self.keyboard = self.event.pool().unwrap();

                    for (i, key) in self.keyboard.iter().enumerate() {
                        if *key {
                            self.cpu.v[x] = i as u8;
                            break 'keypress_waiting;
                        }
                    }
                }
                ProgramCounter::Next
            }

            // Fx15 - LD DT, Vx
            (0xF, _, 0x1, 0x5) => {
                println!("OPCODE: Fx15");
                self.cpu.delay_timer = self.cpu.v[x];
                ProgramCounter::Next
            }

            // Fx18 - LD ST, Vx
            (0xF, _, 0x1, 0x8) => {
                println!("OPCODE: Fx18");
                self.cpu.sound_timer = self.cpu.v[x];
                ProgramCounter::Next
            }

            // Fx1E - ADD I, Vx
            (0xF, _, 0x1, 0xE) => {
                println!("OPCODE: Fx1E");
                let val: u16 = self.cpu.v[x].into();
                self.cpu.i += val;
                ProgramCounter::Next
            }

            // Fx29 - LD F, Vx
            (0xF, _, 0x2, 0x9) => {
                println!("OPCODE: Fx29");
                let val: u16 = self.cpu.v[x].into();
                self.cpu.i = val;
                ProgramCounter::Next
            }

            // Fx33 - LD B, Vx
            (0xF, _, 0x3, 0x3) => {
                println!("OPCODE: Fx33");
                self.memory.ram[self.cpu.i as usize] = self.cpu.v[x] / 100;
                self.memory.ram[self.cpu.i as usize + 1] = (self.cpu.v[x] % 100) / 10;
                self.memory.ram[self.cpu.i as usize + 2] = (self.cpu.v[x] % 10) / 10;
                ProgramCounter::Next
            }

            // Fx55 - LD [I], Vx
            (0xF, _, 0x5, 0x5) => {
                println!("OPCODE: Fx55");
                for i in 0..=x {
                    self.memory.ram[i + self.cpu.i as usize] = self.cpu.v[i];
                }
                ProgramCounter::Next
            }

            // Fx65 - LD Vx, [I]
            (0xF, _, 0x6, 0x5) => {
                println!("OPCODE: Fx65");
                for i in 0..=x {
                    self.cpu.v[i] = self.memory.ram[i + self.cpu.i as usize];
                }
                ProgramCounter::Next
            }

            _ => {
                println!("(ERROR) Invalid opcode: {:#06X}", opcode);
                std::process::exit(1);
            }
        };

        match program_counter {
            ProgramCounter::Next => self.cpu.pc += 2,
            ProgramCounter::Skip => self.cpu.pc += 4,
            ProgramCounter::Jump(instruction) => self.cpu.pc = instruction as u16,
        }
    }

    fn tick(&mut self) {
        self.screen_changed = false;

        let opcode = self.fetch_opcode();
        println!("CURRENT OPCODE: {:#06X}", opcode);
        self.execute_opcode(opcode);

        if self.screen_changed {
            self.draw();
        } 
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

    if let Err(err) = chip8.memory.load_fontset() {
        eprintln!("Load fontset into memory");
    }

    if let Err(err) = chip8.memory.load_rom(rom_path) {
        eprintln!("Unable to load ROM into memory");
    }

    loop {
        chip8.display.canvas.clear();
        chip8.event.pool().unwrap();
        chip8.tick();
    }
}
