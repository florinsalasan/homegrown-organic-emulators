// 7  bit  0
// ---- ----
// VPHB SINN
// |||| ||||
// |||| ||++- Base nametable address
// |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
// |||| |+--- VRAM address increment per CPU read/write of PPUDATA
// |||| |     (0: add 1, going across; 1: add 32, going down)
// |||| +---- Sprite pattern table address for 8x8 sprites
// ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
// |||+------ Background pattern table address (0: $0000; 1: $1000)
// ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
// |+-------- PPU master/slave select
// |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
// +--------- Generate an NMI at the start of the
//            vertical blanking interval (0: off; 1: on)

#[derive(Debug)]
pub struct ControlRegister  {
    value: u8,
}

const NAMETABLE1: u8 = 0b0000_0001;
const NAMETABLE2: u8 = 0b0000_0010;
const VRAM_ADDR_INCREMENT: u8 = 0b0000_0100;
const SPRITE_PATTERN_ADDR: u8 = 0b0000_1000; // not used on nes, still an instruction that clears it
const BACKGROUND_PATTERN_ADDR: u8 = 0b0001_0000;
const SPRITE_SIZE: u8 = 0b0010_0000; // Doesn't represent any flag
const MASTER_SLAVE_SELECT: u8 = 0b0100_0000;
const GENERATE_NMI: u8 = 0b1000_0000;

impl ControlRegister {
    pub fn new() -> Self {
        ControlRegister {
            value: 0,
        }
    }

    pub fn vram_addr_increment(&self) -> u8 {
        if self.value & VRAM_ADDR_INCREMENT != VRAM_ADDR_INCREMENT {
            1
        } else {
            32
        }
    }

    pub fn update(&mut self, data: u8) {
        self.value = data;
    }
}
