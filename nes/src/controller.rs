pub struct ControllerButton {
    value: u8,
}

impl ControllerButton {
    pub fn new() -> Self {
        ControllerButton {
            value:  0,
        }
    }
}

const RIGHT: u8       = 0b10000000;
const LEFT: u8        = 0b01000000;
const DOWN: u8        = 0b00100000;
const UP: u8          = 0b00010000;
const START: u8       = 0b00001000;
const SELECT: u8      = 0b00000100;
const BUTTON_B: u8    = 0b00000010;
const BUTTON_A: u8    = 0b00000001;

pub struct Controller {
    strobe: bool,
    button_idx: u8,
    button_status: ControllerButton,
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            strobe: false,
            button_idx: 0,
            button_status: ControllerButton::new(),
        }
    }

    pub fn write(&mut self, data: u8) {
        self.strobe = data & 1 == 1;
        if self.strobe {
            self.button_idx = 0
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.button_idx > 7 {
            return 1;
        }
        let response = (self.button_status.value & (1 << self.button_idx)) >> self.button_idx;
        if !self.strobe && self.button_idx <= 7 {
            self.button_idx += 1;
        }
        response
    }
}
