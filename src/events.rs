use sdl2::{event::Event, keyboard::{Keycode, Scancode}, EventPump, Sdl};

pub struct EventDriver {
    pub events: EventPump,
}

impl EventDriver {
    pub fn new(sdl_context: &Sdl) -> Self {
        let events = sdl_context.event_pump().unwrap();
        Self { events }
    }

    pub fn get_key(&self, vx: u8) -> Option<Scancode> {
        match vx {
            0x1 => Some(Scancode::Num1),
            0x2 => Some(Scancode::Num2),
            0x3 => Some(Scancode::Num3),
            0xc => Some(Scancode::Num4),
            0x4 => Some(Scancode::Q),
            0x5 => Some(Scancode::W),
            0x6 => Some(Scancode::E),
            0xd => Some(Scancode::R),
            0x7 => Some(Scancode::A),
            0x8 => Some(Scancode::S),
            0x9 => Some(Scancode::D),
            0xe => Some(Scancode::F),
            0xa => Some(Scancode::Z),
            0x0 => Some(Scancode::X),
            0xb => Some(Scancode::C),
            0xf => Some(Scancode::V), 
            _ => None,
        }
    }

    pub fn pool(&mut self) -> Result<[bool; 16], ()> {
        let events = self.events.poll_iter();

        for event in events {
            if let Event::Quit { .. } = event {
                return Err(());
            }
        }

        let keys: Vec<Keycode> = self
            .events
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        let mut chip8_keyboard = [false; 16];

        for key in keys {
            let index = match key {
                Keycode::Num1 => Some(0x1),
                Keycode::Num2 => Some(0x2),
                Keycode::Num3 => Some(0x3),
                Keycode::Num4 => Some(0xc),
                Keycode::Q => Some(0x4),
                Keycode::W => Some(0x5),
                Keycode::E => Some(0x6),
                Keycode::R => Some(0xd),
                Keycode::A => Some(0x7),
                Keycode::S => Some(0x8),
                Keycode::D => Some(0x9),
                Keycode::F => Some(0xe),
                Keycode::Z => Some(0xa),
                Keycode::X => Some(0x0),
                Keycode::C => Some(0xb),
                Keycode::V => Some(0xf),
                _ => None,
            };

            if let Some(i) = index {
                chip8_keyboard[i] = true;
            }
        }

        Ok(chip8_keyboard)
    }
}
