mod cpu;
mod display;
mod events;
mod memory;

use cpu::Cpu;
use display::Display;
use events::EventDriver;
use memory::Memory;

use sdl2::{event::Event, keyboard::Keycode, Sdl};

use std::{env, thread, time::Duration};

const SCALE_FACTOR: u32 = 20;
const WIDTH: u32 = 1280; // 64 * SCALE_FACTOR
const HEIGHT: u32 = 640; // 32 * SCALE_FACTOR

struct Chip8 {
    display: Display,
    cpu: Cpu,
    memory: Memory,
    event_pump: EventDriver,
}

impl Chip8 {
    fn new(sdl_context: &Sdl) -> Self {
        Self {
            display: Display::new(sdl_context, WIDTH, HEIGHT),
            cpu: Cpu::new(),
            memory: Memory::new(),
            event_pump: EventDriver::new(sdl_context),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Error: No input file");
        std::process::exit(1);
    }

    let sdl_context = sdl2::init().unwrap();
    let mut chip8 = Chip8::new(&sdl_context);

    'running: loop {
        chip8.display.canvas.clear();
        for event in chip8.event_pump.events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        chip8.display.canvas.present();
        thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
