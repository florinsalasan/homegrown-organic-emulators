use crate::opcodes::{init_opcodes, init_opcodes_hashmap};

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8, // Use each bit of status as a different flag to be more
    // efficient(?), negative flag is the 7th bit number, bit number 1 is zero flag
    // this is stupid and I hate it, but each flag is one bit instead of one byte
    pub program_counter: u16,
    memory: [u8; 0xFFFF]
}

#[derive(Debug)]
#[derive(Clone)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    Relative,
    NoneAddressing,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: [0; 0xFFFF],
        }
    }

    // LDA that takes in different AddressingModes
    // 0xA9, 0xA5, 0xB5, 0xAD, 0xBD, 0xB9, 0xA1, 0xB1
    pub fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = value;
        self.set_zero_and_neg_flags(self.register_a);
    }

    // STA, copies value from register A into memory
    pub fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    // 0xAA TAX (Transfer accumulator to register X) set register_x
    // to the value in the accumulator, only one addressing mode
    pub fn tax(&mut self) {
        self.register_x = self.register_a;
        self.set_zero_and_neg_flags(self.register_x)
    }

    // 0xE8 INX (Increment Register X) Adds one to the register and
    // then sets the Zero flag, Negative flag if needed
    pub fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.set_zero_and_neg_flags(self.register_x);
    }

    pub fn set_zero_and_neg_flags(&mut self, result: u8) {

        // Set the Zero flag
        if result == 0 {
            self.status = self.status | 0b0000_0010;
        } else {
            self.status = self.status & 0b1111_1101;
        }

        // Set the Negative flag
        if result & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000;
        } else {
            self.status = self.status & 0b0111_1111;
        }

    }
    // read memory at a given address
    pub fn mem_read(&mut self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    // write data to memory at a given address
    pub fn mem_write(&mut self, address: u16, data: u8) {
        self.memory[address as usize] = data
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0;

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000)
    }

    // for mem_read_u16 and mem_write_u16 double check that this isn't breaking anything
    // since macs are little endian like nes was so this might not be necessary at all
    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        // remember in rust if every branch has a line like the one below, it is
        // an implicit return
        (hi << 8) | (lo as u16)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xFF) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }

    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {

        match mode {
            AddressingMode::Immediate => self.program_counter,

            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,

            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
            
            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }

            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }

            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }

            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }
            
            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);

                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }

            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u16).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                deref
            }

            AddressingMode::Relative => {
                todo!("Implement relative jumps: This mode is used by instructions that contain a signed 8bit
offset to add to the program counter if a condition is true.");
            }

            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
                // replace the panic with something else maybe? No reason for 
                // program to panic if an addressing mode isn't needed, for example 
                // TAX transferring the accumulator value to register_x
            }
        }
    }

    // The main CPU loop is:
    // Fetch next instruction from memory,
    // Decode the instruction,
    // Execute the instruction,
    // repeat;

    pub fn run(&mut self) {

        init_opcodes();
        // might as well remove the hashmap? But the method gets_or_inits the pub static
        // hashmap so maybe it is needed, I have no idea what is happening behind the curtain 
        let other_map = init_opcodes_hashmap();

        loop {
            let opcode = self.mem_read(self.program_counter);
            self.program_counter += 1;

            match opcode {
                // BRK 
                0x00 => return,

                // ADC opcodes
                0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => {
                    todo!("
                    self.adc(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                    ")
                }
                
                // AND opcodes
                0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => {
                    todo!("
                    self.and(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                    ")
                }
                
                // ASL opcodes
                0x0A | 0x06 | 0x16 | 0x0E | 0x1E => {
                    todo!("
                    self.asl(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                    ")
                }
                
                // BCC
                0x90 => todo!("self.bcc(),"),
                
                // BCS
                0xB0 => todo!("self.bcs(),"),
                
                // BEQ
                0xF0 => todo!("self.beq(),"),
                
                // BIT opcodes
                0x24 | 0x2C => {
                    todo!("
                    self.bit(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1
                    ");
                }

                // BMI
                0x30 => todo!("self.bmi(),"),

                // BNE
                0xD0 => todo!("self.bne(),"),

                // BPL
                0x10 => todo!("self.bpl(),"),
                
                // BVC
                0x50 => todo!("self.bvc(),"),

                // BVS
                0x70 => todo!("self.bvs(),"),

                // CLC
                0x18 => todo!("self.clc(),"),

                // CLD
                0xD8 => todo!("self.cld(),"),

                // CLI
                0x58 => todo!("self.cli(),"),

                // CLV
                0xB8 => todo!("self.clv(),"),

                // CMP opcodes
                0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => {
                    todo!("
                    self.cmp(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                    ")
                }

                // CPX opcodes
                0xE0 | 0xE4 | 0xEC => {
                    todo!("
                    self.cpx(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                    ")
                }

                // CPY opcodes
                0xC0 | 0xC4 | 0xCC => {
                    todo!("
                    self.cpy(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                    ")
                }

                // DEC opcodes
                0xC6 | 0xD6 | 0xCE | 0xDE => {
                    todo!("
                    self.dec(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                    ")
                }
                
                // DEX
                0xCA => todo!("self.dex(),"),

                // DEY
                0x88 => todo!("self.dex(),"),
                
                // EOR opcodes
                0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => {
                    todo!("
                    self.eor(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                    ")
                }

                // INX
                0xE8 => self.inx(),
                
                // LDA opcodes
                0xA1 | 0xA5 | 0xA9 | 0xAD | 0xB1 | 0xB5 | 0xB9 | 0xBD => {
                    self.lda(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }

                // STA opcodes
                0x81 | 0x85 | 0x8D | 0x91 | 0x95 | 0x99 | 0x9D => {
                    // init the opcode hashtable somewhere and then start accessing it to enter the
                    // parameters of the helper functions here
                    self.sta(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }

                // TAX
                0xAA => self.tax(),

                _ => todo!("Build out the massive switch statement for opcodes")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 0xA5
    #[test]
    fn test_0xa5_lda_zeropage_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa5_lda_zeropage_from_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);

        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_0xa5_lda_zeropage_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_0xa5_lda_zeropage_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xF0, 0x00]);
        assert!(cpu.status & 0b1000_0000 == 0b1000_0000);
    }

    // 0xA9
    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_immediate_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_0xa9_lda_immediate_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xF0, 0x00]);
        assert!(cpu.status & 0b1000_0000 == 0b1000_0000);
    }

    // 0xAA, could have just set register_a manually lol, not anymore, this was for the better
    #[test]
    fn test_0xaa_tax() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0xaa, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xaa_tax_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        cpu.status = 0;
        cpu.load_and_run(vec![0xaa, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_0xaa_tax_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xF0, 0xaa, 0x00]);
        assert!(cpu.status & 0b1000_0000 == 0b1000_0000);
    }
    
    // 0xE8 
    #[test]
    fn test_0xe8_inx() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xe8, 0x00]);
        assert!(cpu.register_x == 1);
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xe8_inx_zero_flag() {
        // load_and_run now resets the registers so need to create a program to test this properly
        // without setting registers manually
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0x00]);
        assert!(cpu.register_x == 0);
        assert!(cpu.status & 0b0000_0010 == 0b0000_0010);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xe8_inx_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xfe, 0xaa, 0xe8, 0x00]);
        assert_eq!(cpu.register_x, 255);
        assert!(cpu.status & 0b0000_0010 == 0);
        assert!(cpu.status & 0b1000_0000 == 0b1000_0000);
    }

    #[test]
    fn test_5_ops_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
        assert!(cpu.register_a == 0xc0);
        assert!(cpu.register_a == cpu.register_x - 1);
        assert_eq!(cpu.register_x, 0xc1);
    }

    #[test]
    fn test_0xe8_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]);
        assert_eq!(cpu.register_x, 1)
    }

    // 0x85 
    #[test]
    fn test_0x85_sta_zeropage() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x15, 0x85, 0x00]);
        // STA writes to the start of memory, honestly not sure if that's what it's
        // supposed to do
        assert_eq!(cpu.memory[0x00], 0x15);
    }
}
