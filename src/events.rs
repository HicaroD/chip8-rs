use sdl2::{EventPump, Sdl};

pub struct EventDriver {
    pub events: EventPump,
}

impl EventDriver {
    pub fn new(sdl_context: &Sdl) -> Self {
        let events = sdl_context.event_pump().unwrap();
        Self { events }
    }
}
