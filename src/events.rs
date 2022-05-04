use sdl2::{
    Sdl,
    EventPump,
};

pub struct EventDriver {
    pub events: EventPump,
}

impl EventDriver {
    pub fn new(sdl_context: &Sdl) -> Self {
        let events = sdl_context.event_pump().unwrap();
        Self { events } 
    }
}
