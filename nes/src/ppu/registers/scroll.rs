// 1st write
// 7  bit  0
// ---- ----
// XXXX XXXX
// |||| ||||
// ++++-++++- X scroll bits 7-0 (bit 8 in PPUCTRL bit 0)
// 
// 2nd write
// 7  bit  0
// ---- ----
// YYYY YYYY
// |||| ||||
// ++++-++++- Y scroll bits 7-0 (bit 8 in PPUCTRL bit 1)

#[derive(Debug)]
pub struct ScrollRegister  {
    scroll_x: u8,
    scroll_y: u8,
    pub latch: bool,
}

impl ScrollRegister {
    pub fn new() -> Self {
        ScrollRegister {
            scroll_x: 0,
            scroll_y: 0,
            latch: false,
        }
    }

    pub fn reset_latch(&mut self) {
        self.latch = false;
    }

    pub fn write(&mut self, data: u8) {
        if !self.latch {
            self.scroll_x = data;
        } else {
            self.scroll_y = data;
        }
        self.latch = !self.latch;
    }
}
