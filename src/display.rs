use sdl2::{render::Canvas, video::Window, Sdl};

pub struct Display {
    pub canvas: Canvas<Window>,
}

impl Display {
    pub fn new(sdl_context: &Sdl, width: u32, height: u32) -> Self {
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("CHIP8", width, height)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.clear();
        canvas.present();
        Self { canvas }
    }
}
