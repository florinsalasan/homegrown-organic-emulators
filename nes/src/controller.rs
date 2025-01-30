/*
const BUTTON_A: u8 = 0b0000_0001;
const BUTTON_B: u8 = 0b0000_0010;
const SELECT: u8 = 0b0000_0100;
const START: u8 = 0b0000_1000;
const UP: u8 = 0b0001_0000;
const DOWN: u8 = 0b0010_0000;
const LEFT: u8 = 0b0100_0000;
const RIGHT: u8 = 0b1000_0000;
*/

#[derive(Debug, Clone, Copy)]
pub enum ControllerButtons {
    BUTTON_A = 0b0000_0001,
    BUTTON_B = 0b0000_0010,
    SELECT = 0b0000_0100,
    START = 0b0000_1000,
    UP = 0b0001_0000,
    DOWN = 0b0010_0000,
    LEFT = 0b0100_0000,
    RIGHT = 0b1000_0000,
}

#[derive(Debug, Clone, Copy)]
pub struct Controller {
    strobe: bool,
    button_idx: u8,
    pub button_status: u8,
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            strobe: false,
            button_idx: 0,
            button_status: 0,
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
        let response = (self.button_status & (1 << self.button_idx)) >> self.button_idx;
        if !self.strobe && self.button_idx <= 7 {
            self.button_idx += 1;
        }
        response
    }

    pub fn set_button_pressed_status(&mut self, init_button: ControllerButtons, pressed: bool) {
        let button = init_button as u8;
        if pressed {
            self.button_status = self.button_status | button;
        } else {
            self.button_status = self.button_status & !button;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_strobe_mode() {
        let mut joypad = Controller::new();
        joypad.write(1);
        joypad.set_button_pressed_status(ControllerButtons::BUTTON_A, true);
        for _x in 0..10 {
            assert_eq!(joypad.read(), 1);
        }
    }

    #[test]
    fn test_strobe_mode_on_off() {
        let mut joypad = Controller::new();

        joypad.write(0);
        joypad.set_button_pressed_status(ControllerButtons::RIGHT, true);
        joypad.set_button_pressed_status(ControllerButtons::LEFT, true);
        joypad.set_button_pressed_status(ControllerButtons::SELECT, true);
        joypad.set_button_pressed_status(ControllerButtons::BUTTON_B, true);

        for _ in 0..=1 {
            assert_eq!(joypad.read(), 0);
            assert_eq!(joypad.read(), 1);
            assert_eq!(joypad.read(), 1);
            assert_eq!(joypad.read(), 0);
            assert_eq!(joypad.read(), 0);
            assert_eq!(joypad.read(), 0);
            assert_eq!(joypad.read(), 1);
            assert_eq!(joypad.read(), 1);

            for _x in 0..10 {
                assert_eq!(joypad.read(), 1);
            }
            joypad.write(1);
            joypad.write(0);
        }
    }
}
