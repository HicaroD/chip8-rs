mod display;
mod events;

use display::Display;
use sdl2::{
    event::Event,
    keyboard::Keycode,
};
use std::{env, thread, time::Duration};

const SCALE_FACTOR: u32 = 20;
const WIDTH: u32 = 1280; // 64 * SCALE_FACTOR
const HEIGHT: u32 = 640; // 32 * SCALE_FACTOR

struct Chip8 {}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Error: No input file");
        std::process::exit(1);
    }

    let sdl_context = sdl2::init().unwrap();
    let mut window = Display::new(&sdl_context, WIDTH, HEIGHT);

    let mut event_pump = events::EventDriver::new(&sdl_context);

    'running: loop {
        window.canvas.clear();
        for event in event_pump.events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        window.canvas.present();
        thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
