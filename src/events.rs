use sdl2::{EventPump, Sdl, event::Event};

pub struct EventDriver {
    pub events: EventPump,
}

impl EventDriver {
    pub fn new(sdl_context: &Sdl) -> Self {
        let events = sdl_context.event_pump().unwrap();
        Self { events }
    }

    pub fn pool(&mut self) -> Result<(), ()> {
        let events = self.events.poll_iter();
        for event in events {
            if let Event::Quit {..} = event {
                return Err(());
            }
        }
        Ok(())
    }
}
