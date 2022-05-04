pub struct Cpu {
    pub pc: u16,
    pub sp: u8,
    pub v: Vec<u8>,
    pub i: u16,
    pub delay_timer: u8,
    pub sound_timer: u8,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            pc: 0x200,
            sp: 0,
            v: vec![0; 16],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
        }
    }
}
