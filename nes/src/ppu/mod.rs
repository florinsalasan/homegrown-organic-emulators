use std::usize;

use crate::cartridge::Mirroring;
use registers::control::ControlRegister;
use registers::mask::MaskRegister;
use registers::status::StatusRegister;
use registers::scroll::ScrollRegister;
use registers::addr::AddrRegister;

pub mod registers;

#[derive(Debug)]
pub struct NesPPU {
    pub chr_rom: Vec<u8>,
    pub palette_table: [u8; 32],
    pub vram: [u8; 2048],
    pub oam_data: [u8; 256],
    pub oam_addr: u8,

    pub addr: AddrRegister,
    pub ctrl: ControlRegister,
    pub mask: MaskRegister,
    pub status: StatusRegister,
    pub scroll: ScrollRegister,

    internal_data_buf: u8,

    pub scanline: u16,
    cycles: usize,

    pub mirroring: Mirroring,

    pub nmi_interrupt: Option<u8>,
}

pub trait PPU {
    fn write_to_ctrl(&mut self, value: u8);
    fn write_to_mask(&mut self, value: u8);
    fn read_status(&mut self) -> u8;
    fn write_to_oam_addr(&mut self, value: u8);
    fn write_to_oam_data(&mut self, value: u8);
    fn read_oam_data(&self) -> u8;
    fn write_to_scroll(&mut self, value: u8);
    fn write_to_ppu_addr(&mut self, value: u8);
    fn write_to_data(&mut self, value: u8);
    fn read_data(&mut self) -> u8;
    fn write_oam_dma(&mut self, value: &[u8; 256]);
}

impl NesPPU {
    pub fn new_empty_rom() -> Self {
        NesPPU::new(vec![0; 2048], Mirroring::HORIZONTAL)
    }

    pub fn poll_nmi_interrupt(&mut self) -> Option<u8> {
        self.nmi_interrupt.take()
    }

    pub fn tick(&mut self, cycles: u8) -> bool {
        self.cycles += cycles as usize;
        if self.cycles >= 341 {

            if self.is_sprite_0_hit(self.cycles) {
                self.status.set_sprite_zero_hit(true);
            }

            self.cycles = self.cycles - 341;
            self.scanline += 1;

            if self.scanline == 241 {
                self.status.set_vblank_status(true);
                self.status.set_sprite_zero_hit(false);
                if self.ctrl.generate_vblank_nmi() {
                    self.nmi_interrupt = Some(1);
                }
            }

            if self.scanline >= 262 {
                self.scanline = 0;
                self.nmi_interrupt = None;
                self.status.set_sprite_zero_hit(false);
                self.status.reset_vblank_status();
                return true;
            }
        }
        return false;
    }

    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        NesPPU { 
            chr_rom, 
            mirroring,
            ctrl: ControlRegister::new(),
            mask: MaskRegister::new(),
            status: StatusRegister::new(),
            oam_addr: 0,
            scroll: ScrollRegister::new(),
            addr: AddrRegister::new(),
            palette_table: [0; 32], 
            vram: [0; 2048], 
            oam_data: [0; 64 * 4], 
            internal_data_buf: 0,
            scanline: 0,
            cycles: 0,
            nmi_interrupt: None,
        }
    }

    fn increment_vram_addr(&mut self) {
        self.addr.increment(self.ctrl.vram_addr_increment());
    }


    fn is_sprite_0_hit(&self, cycle: usize) -> bool {
        let y = self.oam_data[0] as usize;
        let x = self.oam_data[3] as usize;

        (y == self.scanline as usize) && x <= cycle && self.mask.show_sprites()
    }

    // Mirroring:
    //
    // Horizontal:
    //  [ A ] [ a ]
    //  [ B ] [ b ]
    //
    // Vertical:
    // [ A ] [ B ]
    // [ a ] [ b ]
    //
    pub fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0b10111111111111; // mirror down 0x3000-0x3EFF to 0x2000 - 0x2EFF
        let vram_index = mirrored_vram - 0x2000; // to vram vector
        let name_table = vram_index / 0x0400; // to the name table index

        match (&self.mirroring, name_table) {
            (Mirroring::VERTICAL, 2) | (Mirroring::VERTICAL, 3) => vram_index - 0x0800,
            (Mirroring::HORIZONTAL, 2) | (Mirroring::HORIZONTAL, 1)  => vram_index - 0x0400,
            // (Mirroring::HORIZONTAL, 1) => vram_index - 0x0400,
            (Mirroring::HORIZONTAL, 3) => vram_index - 0x0800,
            _ => vram_index,
        }
    }

}

impl PPU for NesPPU {

    fn write_to_ppu_addr(&mut self, value: u8) {
        self.addr.update(value);
    }

    fn write_to_ctrl(&mut self, value: u8) {
        let before_nmi_status = self.ctrl.generate_vblank_nmi();
        self.ctrl.update(value);
        if !before_nmi_status && self.ctrl.generate_vblank_nmi() && self.status.is_in_vblank() {
            self.nmi_interrupt = Some(1);
        }
    }

    fn write_to_data(&mut self, value: u8) {
        let addr = self.addr.get();
        match addr {
            0..=0x1FFF => println!("Attempted to write to chr rom space: {:04x}", addr),
            0x2000..=0x2FFF => {
                self.vram[self.mirror_vram_addr(addr) as usize] = value;
            }
            0x3000..=0x3EFF => unimplemented!("Addr {:04x} shouldn't be used :/", addr),
            0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => {
                let addr_mirror = addr - 0x0010;
                self.palette_table[(addr_mirror - 0x3F00) as usize] = value;
            }
            0x3F00..=0x3FFF => {
                self.palette_table[(addr - 0x3F00) as usize] = value;
            }
            _ => panic!("unexpected access to mirrored space {:04x}", addr),
        }
        self.increment_vram_addr();
    }

    fn read_data(&mut self) -> u8 {
        let addr = self.addr.get();
        self.increment_vram_addr();

        match addr {
            0..=0x1FFF => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.chr_rom[addr as usize];
                result
            }
            0x2000..=0x2FFF => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.vram[self.mirror_vram_addr(addr) as usize];
                result
            }
            0x3000..=0x3EFF => unimplemented!("Addr {:04x} shouldn't be used :/", addr),
            0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => {
                let add_mirror = addr - 0x0010;
                self.palette_table[(add_mirror - 0x3F00) as usize]
            }
            0x3F00..=0x3FFF => {
                self.palette_table[(addr - 0x3F00) as usize]
            }
            _ => panic!("unexpected access to mirrored space {:04x}", addr),
        }
    }

    fn write_to_mask(&mut self, value: u8) {
        self.mask.update(value);
    }

    fn read_status(&mut self) -> u8 {
        let data = self.status.snapshot();
        self.status.reset_vblank_status();
        self.addr.reset_latch();
        self.scroll.reset_latch();
        data
    }

    fn write_to_oam_addr(&mut self, value: u8) {
        self.oam_addr = value;
    }

    fn write_to_oam_data(&mut self, value: u8) {
        self.oam_data[self.oam_addr as usize] = value;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    fn read_oam_data(&self) -> u8 {
        self.oam_data[self.oam_addr as usize]
    }

    fn write_to_scroll(&mut self, value: u8) {
        self.scroll.write(value);
    }

    fn write_oam_dma(&mut self, value: &[u8; 256]) {
        for x in value.iter() {
            self.oam_data[self.oam_addr as usize] = *x;
            self.oam_addr = self.oam_addr.wrapping_add(1);
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_ppu_vram_writes() {
        let mut ppu = NesPPU::new_empty_rom();
        ppu.write_to_ppu_addr(0x23);
        ppu.write_to_ppu_addr(0x05);
        ppu.write_to_data(0x66);

        assert_eq!(ppu.vram[0x0305], 0x66);
    }

    #[test]
    fn test_ppu_vram_reads() {
        let mut ppu = NesPPU::new_empty_rom();
        ppu.write_to_ctrl(0);
        ppu.vram[0x0305] = 0x66;

        ppu.write_to_ppu_addr(0x23);
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load_into_buffer
        assert_eq!(ppu.addr.get(), 0x2306);
        assert_eq!(ppu.read_data(), 0x66);
    }

    #[test]
    fn test_ppu_vram_reads_cross_page() {
        let mut ppu = NesPPU::new_empty_rom();
        ppu.write_to_ctrl(0);
        ppu.vram[0x01ff] = 0x66;
        ppu.vram[0x0200] = 0x77;

        ppu.write_to_ppu_addr(0x21);
        ppu.write_to_ppu_addr(0xff);

        ppu.read_data(); //load_into_buffer
        assert_eq!(ppu.read_data(), 0x66);
        assert_eq!(ppu.read_data(), 0x77);
    }

    #[test]
    fn test_ppu_vram_reads_step_32() {
        let mut ppu = NesPPU::new_empty_rom();
        ppu.write_to_ctrl(0b100);
        ppu.vram[0x01ff] = 0x66;
        ppu.vram[0x01ff + 32] = 0x77;
        ppu.vram[0x01ff + 64] = 0x88;

        ppu.write_to_ppu_addr(0x21);
        ppu.write_to_ppu_addr(0xff);

        ppu.read_data(); //load_into_buffer
        assert_eq!(ppu.read_data(), 0x66);
        assert_eq!(ppu.read_data(), 0x77);
        assert_eq!(ppu.read_data(), 0x88);
    }

    // Horizontal: https://wiki.nesdev.com/w/index.php/Mirroring
    //   [0x2000 A ] [0x2400 a ]
    //   [0x2800 B ] [0x2C00 b ]
    #[test]
    fn test_vram_horizontal_mirror() {
        let mut ppu = NesPPU::new_empty_rom();
        ppu.write_to_ppu_addr(0x24);
        ppu.write_to_ppu_addr(0x05);

        ppu.write_to_data(0x66); //write to a

        ppu.write_to_ppu_addr(0x28);
        ppu.write_to_ppu_addr(0x05);

        ppu.write_to_data(0x77); //write to B

        ppu.write_to_ppu_addr(0x20);
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load into buffer
        assert_eq!(ppu.read_data(), 0x66); //read from A

        ppu.write_to_ppu_addr(0x2C);
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load into buffer
        assert_eq!(ppu.read_data(), 0x77); //read from b
    }

    // Vertical: https://wiki.nesdev.com/w/index.php/Mirroring
    //   [0x2000 A ] [0x2400 B ]
    //   [0x2800 a ] [0x2C00 b ]
    #[test]
    fn test_vram_vertical_mirror() {
        let mut ppu = NesPPU::new(vec![0; 2048], Mirroring::VERTICAL);

        ppu.write_to_ppu_addr(0x20);
        ppu.write_to_ppu_addr(0x05);

        ppu.write_to_data(0x66); //write to A

        ppu.write_to_ppu_addr(0x2C);
        ppu.write_to_ppu_addr(0x05);

        ppu.write_to_data(0x77); //write to b

        ppu.write_to_ppu_addr(0x28);
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load into buffer
        assert_eq!(ppu.read_data(), 0x66); //read from a

        ppu.write_to_ppu_addr(0x24);
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load into buffer
        assert_eq!(ppu.read_data(), 0x77); //read from B
    }

    #[test]
    fn test_read_status_resets_latch() {
        let mut ppu = NesPPU::new_empty_rom();
        ppu.vram[0x0305] = 0x66;

        ppu.write_to_ppu_addr(0x21);
        ppu.write_to_ppu_addr(0x23);
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load_into_buffer
        assert_ne!(ppu.read_data(), 0x66);

        ppu.read_status();

        ppu.write_to_ppu_addr(0x23);
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load_into_buffer
        assert_eq!(ppu.read_data(), 0x66);
    }

    #[test]
    fn test_ppu_vram_mirroring() {
        let mut ppu = NesPPU::new_empty_rom();
        ppu.write_to_ctrl(0);
        ppu.vram[0x0305] = 0x66;

        ppu.write_to_ppu_addr(0x63); //0x6305 -> 0x2305
        ppu.write_to_ppu_addr(0x05);

        ppu.read_data(); //load into_buffer
        assert_eq!(ppu.read_data(), 0x66);
        // assert_eq!(ppu.addr.read(), 0x0306)
    }

    #[test]
    fn test_read_status_resets_vblank() {
        let mut ppu = NesPPU::new_empty_rom();
        ppu.status.set_vblank_status(true);

        let status = ppu.read_status();

        assert_eq!(status >> 7, 1);
        assert_eq!(ppu.status.snapshot() >> 7, 0);
    }

    #[test]
    fn test_oam_read_write() {
        let mut ppu = NesPPU::new_empty_rom();
        ppu.write_to_oam_addr(0x10);
        ppu.write_to_oam_data(0x66);
        ppu.write_to_oam_data(0x77);

        ppu.write_to_oam_addr(0x10);
        assert_eq!(ppu.read_oam_data(), 0x66);

        ppu.write_to_oam_addr(0x11);
        assert_eq!(ppu.read_oam_data(), 0x77);
    }

    #[test]
    fn test_oam_dma() {
        let mut ppu = NesPPU::new_empty_rom();

        let mut data = [0x66; 256];
        data[0] = 0x77;
        data[255] = 0x88;

        ppu.write_to_oam_addr(0x10);
        ppu.write_oam_dma(&data);

        ppu.write_to_oam_addr(0xf); //wrap around
        assert_eq!(ppu.read_oam_data(), 0x88);

        ppu.write_to_oam_addr(0x10);
        assert_eq!(ppu.read_oam_data(), 0x77);
  
        ppu.write_to_oam_addr(0x11);
        assert_eq!(ppu.read_oam_data(), 0x66);
    }
}
