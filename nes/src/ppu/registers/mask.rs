// 7  bit  0
// ---- ----
// BGRs bMmG
// |||| ||||
// |||| |||+- Greyscale (0: normal color, 1: greyscale)
// |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
// |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
// |||| +---- 1: Enable background rendering
// |||+------ 1: Enable sprite rendering
// ||+------- Emphasize red (green on PAL/Dendy)
// |+-------- Emphasize green (red on PAL/Dendy)
// +--------- Emphasize blue#[derive(Debug)]


#[derive(Debug)]
pub struct MaskRegister {
    value: u8,
}

const GREYSCALE: u8 = 0b0000_0001;
const BACKGROUND_LEFT_BOOL: u8 = 0b0000_0010;
const SPRITE_LEFT_BOOL: u8 = 0b0000_0100;
const BACKGROUND_RENDERING: u8 = 0b0000_1000; // not used on nes, still an instruction that clears it
const SPRITE_RENDERING: u8 = 0b0001_0000;
const EMPHASIZE_RED: u8 = 0b0010_0000; // Doesn't represent any flag
const EMPHASIZE_GREEN: u8 = 0b0100_0000;
const EMPHASIZE_BLUE: u8 = 0b1000_0000;

pub enum Color {
    Red,
    Green,
    Blue,
}

impl MaskRegister {

    pub fn new() -> Self {
        MaskRegister {
            value: 0,
        }
    }

    pub fn update(&mut self, data: u8) {
        self.value = data;
    }

    pub fn is_grayscale(&self) -> bool {
        self.value & GREYSCALE == GREYSCALE
    }

    pub fn leftmost_8pixels_background(&self) -> bool {
        self.value & BACKGROUND_LEFT_BOOL == BACKGROUND_LEFT_BOOL
    }

    pub fn leftmost_8pixels_sprite(&self) -> bool {
        self.value & SPRITE_LEFT_BOOL == SPRITE_LEFT_BOOL
    }

    pub fn show_background(&self) -> bool {
        self.value & BACKGROUND_RENDERING == BACKGROUND_RENDERING
    }

    pub fn show_sprites(&self) -> bool {
        self.value & SPRITE_RENDERING == SPRITE_RENDERING
    }

    pub fn emphasize(&self) -> Vec<Color> {
        let mut result = Vec::<Color>::new();
        if self.value & EMPHASIZE_RED == EMPHASIZE_RED {
            result.push(Color::Red);
        }
        if self.value & EMPHASIZE_BLUE == EMPHASIZE_BLUE {
            result.push(Color::Blue);
        }
        if self.value & EMPHASIZE_GREEN == EMPHASIZE_GREEN {
            result.push(Color::Green);
        }

        result

    }
}
