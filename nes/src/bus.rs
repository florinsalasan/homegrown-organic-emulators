use crate::cartridge::Rom;
use crate::cpu::Memory;
use crate::ppu::NesPPU;
use crate::ppu::PPU;

//  _______________ $10000  _______________
// | PRG-ROM       |       |               |
// | Upper Bank    |       |               |
// |_ _ _ _ _ _ _ _| $C000 | PRG-ROM       |
// | PRG-ROM       |       |               |
// | Lower Bank    |       |               |
// |_______________| $8000 |_______________|
// | SRAM          |       | SRAM          |
// |_______________| $6000 |_______________|
// | Expansion ROM |       | Expansion ROM |
// |_______________| $4020 |_______________|
// | I/O Registers |       |               |
// |_ _ _ _ _ _ _ _| $4000 |               |
// | Mirrors       |       | I/O Registers |
// | $2000-$2007   |       |               |
// |_ _ _ _ _ _ _ _| $2008 |               |
// | I/O Registers |       |               |
// |_______________| $2000 |_______________|
// | Mirrors       |       |               |
// | $0000-$07FF   |       |               |
// |_ _ _ _ _ _ _ _| $0800 |               |
// | RAM           |       | RAM           |
// |_ _ _ _ _ _ _ _| $0200 |               |
// | Stack         |       |               |
// |_ _ _ _ _ _ _ _| $0100 |               |
// | Zero Page     |       |               |
// |_______________| $0000 |_______________|

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

pub struct Bus<'call> {
    cpu_vram: [u8; 2048],
    prg_rom: Vec<u8>,
    ppu: NesPPU,

    cycles: usize,
    gameloop_callback: Box<dyn FnMut(&NesPPU) + 'call>,
}

impl<'a> Bus<'a> {
    pub fn new<'call, F>(rom: Rom, gameloop_callback: F) -> Bus<'call>
    where
        F: FnMut(&NesPPU) + 'call,
    {
        let ppu = NesPPU::new(rom.chr_rom, rom.screen_mirroring);

        Bus {
            cpu_vram: [0; 2048],
            prg_rom: rom.prg_rom,
            ppu,
            cycles: 0,
            gameloop_callback: Box::from(gameloop_callback),
        }
    }

    fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr -= 0x8000;
        if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            // mirror if needed
            addr = addr % 0x4000;
        }
        self.prg_rom[addr as usize]
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize;

        let new_frame = self.ppu.tick(cycles * 3);

        if new_frame {
            (self.gameloop_callback)(&self.ppu)
        }

    }

    pub fn poll_nmi_status(&mut self) -> Option<u8> {
        self.ppu.nmi_interrupt.take()
    }
}

impl Memory for Bus<'_> {
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize]
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                // panic!("Attempt to read from write-only PPU address {:x}", addr);
                0
            }
            0x2002 => self.ppu.read_status(),
            0x2004 => self.ppu.read_oam_data(),
            0x2007 => self.ppu.read_data(),

            0x4000..=0x4015 => {
                // Future APU reads
                0
            }

            0x4016 => {
                // Future Joypad1 value
                0
            }

            0x4017 => {
                // Future Joypad2 value
                0
            }

            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                let _mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_read(_mirror_down_addr)
            }
            0x8000..=0xFFFF => self.read_prg_rom(addr),
            _ => {
                println!("Ignoring memory read access at {:04x}\n", addr);
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b11111111111;
                self.cpu_vram[mirror_down_addr as usize] = data;
            }
            0x2000 => {
                self.ppu.write_to_ctrl(data);
            }
            0x2001 => {
                self.ppu.write_to_mask(data);
            }
            0x2002 => {
                panic!("Attempt to write to PPU status register");
            }
            0x2003 => {
                self.ppu.write_to_oam_addr(data);
            }
            0x2004 => {
                self.ppu.write_to_oam_data(data);
            }
            0x2005 => {
                self.ppu.write_to_scroll(data);
            }
            0x2006 => {
                self.ppu.write_to_ppu_addr(data);
            }
            0x2007 => {
                self.ppu.write_to_data(data);
            }
            
            0x4000..=0x4013 | 0x4015 => {
                // APU access
            }

            0x4016 => {
                // Future Joypad1 value
            }

            0x4017 => {
                // Future Joypad2 value
            }

            0x4014 => {
                let mut buffer: [u8; 256] = [0; 256];
                let hi: u16 = (data as u16) << 8;
                for i in 0..256u16 {
                    buffer[i as usize] = self.mem_read(hi + i);
                }
                self.ppu.write_oam_dma(&buffer);
            }

            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                let _mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_write(_mirror_down_addr, data);
            }
            0x8000..=0xFFFF => {
                print!(
                    "Attempting to write to Cartridge ROM space fix this!! Addr: {:x}",
                    addr
                )
            }
            _ => {
                print!("Ignoring memory write access at {:x}\n", addr);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cartridge::test;

    #[test]
    fn test_mem_read_write_to_ram() {
        let mut bus = Bus::new(test::test_rom(), |ppu: &NesPPU| {}); 
        bus.mem_write(0x01, 0x55);
        assert_eq!(bus.mem_read(0x01), 0x55);
    }
}
