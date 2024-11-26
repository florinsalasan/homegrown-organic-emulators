// 7  bit  0
// ---- ----
// VSOx xxxx
// |||| ||||
// |||+------ PPU open bus or 2C05 PPU identifier
// ||+------- Sprite overflow flag
// |+-------- Sprite 0 hit flag
// +--------- Vblank flag, cleared on read. Labelled as unreliable on nesdev


#[derive(Debug)]
pub struct StatusRegister {
    value: u8,
}

const PPU_OPEN_BUS: u8 = 0b0001_0000;
const SPRITE_OVERFLOW: u8 = 0b0010_0000; // Doesn't represent any flag
const SPRITE_0_HIT: u8 = 0b0100_0000;
const VBLANK: u8 = 0b1000_0000;


impl StatusRegister {

    pub fn new() -> Self {
        StatusRegister {
            value: 0,
        }
    }

    pub fn update(&mut self, data: u8) {
        self.value = data;
    }

    pub fn get(&self) -> u8 {
        self.value
    }

    pub fn snapshot(&self) -> u8 {
        self.value
    }

    pub fn set_vblank_status(&mut self, flagged: bool) {
        if flagged {
            self.value = self.value | VBLANK;
        } else {
            self.value = self.value & !VBLANK;
        }
    }
}
