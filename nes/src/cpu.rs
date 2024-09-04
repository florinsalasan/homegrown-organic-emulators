use crate::opcodes::{init_opcodes, init_opcodes_hashmap};

// # Status Register (P) http://wiki.nesdev.com/w/index.php/Status_flags
//
//  7 6 5 4 3 2 1 0
//  N V _ B D I Z C
//  | |   | | | | +--- Carry Flag
//  | |   | | | +----- Zero Flag
//  | |   | | +------- Interrupt Disable
//  | |   | +--------- Decimal Mode (not used on NES)
//  | |   +----------- Break Command
//  | +--------------- Overflow Flag
//  +----------------- Negative Flag
// Access these flags with cpu.status then use bitwise operations

const CARRY_BIT: u8 = 0b0000_0001;
const ZERO_BIT: u8 = 0b0000_0010;
const INTERRUPT_DISABLE_BIT: u8 = 0b0000_0100;
const DECIMAL_MODE: u8 = 0b0000_1000; // not used on nes
const BREAK_BIT: u8 = 0b0001_0000;
const NOT_A_FLAG_BIT: u8 = 0b0010_0000; // Doesn't represent any flag
const OVERFLOW_BIT: u8 = 0b0100_0000;
const NEGATIVE_BIT: u8 = 0b1000_0000;

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8, 
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
    Accumulator,
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
    // ADC, add with carry, reading the value of a given address, add the value 
    // to the accumulator with the carry bit, if overflow occurs, carry bit is
    // set enabling multiple byte addition
    pub fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value_to_add = self.mem_read(addr);

        // save the sum, to be able to properly set the necessary flags
        let sum = (self.register_a as u16) + (value_to_add as u16) + (if self.status & CARRY_BIT == CARRY_BIT { 1 } else { 0 } as u16);

        let carry = sum > 0xff;

        if carry {
            self.status = self.status | CARRY_BIT; 
        } else {
            self.status = self.status & !CARRY_BIT;
        }

        let result  = sum as u8;

        // I don't understand what this is looking for, but there is an article
        // describing that overflow occurs when this LHS is nonzero, and I choose to
        // believe that he is correct as he explains the bit operations in depth.
        if (value_to_add ^ result) & (result ^ self.register_a) & 0x80 != 0 {
            self.status = self.status | OVERFLOW_BIT; 
        } else {
            // keep all of the other status flags while turning off the overflow_bit
            self.status = self.status & !OVERFLOW_BIT; 
        }

        // store the result to register_a
        self.register_a = result;

        // sets zero and negative flags, still need to set overflow and carry flags
        self.set_zero_and_neg_flags(self.register_a);
        // all 4 flags that can be set by this instruction are set
    }

    // AND - Logical AND is performed bit by bit on the accumulator (register_a) and the 
    // byte of memory that is accessed.
    pub fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a &= value; // surely this is too simple
        self.set_zero_and_neg_flags(self.register_a);
    }

    // ASL - Arithmetic Shift Left, the operation shifts all bits of the accumulator (register_a)
    // or the memory contents one bit to the left, bit 7 is placed into the carry 
    // flag and bit 0 is set to 0. Zero and Negative flags also need to be updated
    pub fn asl(&mut self, mode: &AddressingMode) {
        let mut value_to_modify: u8;
        let mut addr: u16 = 0;
        if matches!(mode, AddressingMode::Accumulator) {
            // modify accumulator directly
            value_to_modify = self.register_a;
        } else {
            addr = self.get_operand_address(mode);
            value_to_modify = self.mem_read(addr);
        }

        // shift left one bit after saving bit 7 as the carry bit
        if value_to_modify & CARRY_BIT == CARRY_BIT {
            self.status = self.status | CARRY_BIT
        } else {
            self.status = self.status & !CARRY_BIT;
        }

        // flag is set, shift it over by one, then set the zero and negative flags
        value_to_modify = value_to_modify << 1;

        self.set_zero_and_neg_flags(value_to_modify);

        if matches!(mode, AddressingMode::Accumulator) {
            // modify accumulator directly
            self.register_a = value_to_modify;
        } else {
            // this should only ever write to memory to the proper location, should
            // never run if addressingMode is Accumulator
            self.mem_write(addr, value_to_modify);
        }
    }

    // BCC - Branch if carry clear: if the carry flag is clear, add the relative
    // displacement to the program counter to cause a branch to a new location
    // absolutely no idea what that means
    pub fn bcc(&mut self) {
        todo!("Implement BCC");
    }

    // LDA that takes in different AddressingModes
    // loads a byte of memory into the accumulator (register_a) and sets zero and neg flags
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
        // does this actually check for the negative flag properly?
        self.set_zero_and_neg_flags(self.register_x);
    }

    pub fn set_zero_and_neg_flags(&mut self, result: u8) {

        // Set the Zero flag
        if result == 0 {
            self.status = self.status | ZERO_BIT;
        } else {
            self.status = self.status & !ZERO_BIT;
        }

        // Set the Negative flag
        if result & 0b1000_0000 != 0 {
            self.status = self.status | NEGATIVE_BIT;
        } else {
            self.status = self.status & !NEGATIVE_BIT;
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
        // Then NES typically uses 0x8000-0xFFFF for loading in the cartridge ROM
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

            AddressingMode::Accumulator => {
                // This just modifies the accumulator directly, shouldn't really return anything
                // here right?, Just throw in a check to see if the addressing mode is Accumulator 
                // in any functions that can modify it directly 
                return 0x00;
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
                    self.adc(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }
                
                // AND opcodes
                0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => {
                    self.and(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }
                
                // ASL opcodes
                0x0A | 0x06 | 0x16 | 0x0E | 0x1E => {
                    self.asl(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
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

                // INC opcodes
                0xE6 | 0xF6 | 0xEE | 0xFE => {
                    todo!("
                    self.inc(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                    ")
                }

                // INX
                0xE8 => self.inx(),
                
                // INY
                0xC8 => todo!("self.iny(),"),

                // JMP 
                0x4C | 0x6C => {
                    todo!("
                    self.jmp(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                    ")
                }
                
                // JSR
                0x20 => todo!("self.jsr(),"),

                // LDA opcodes
                0xA1 | 0xA5 | 0xA9 | 0xAD | 0xB1 | 0xB5 | 0xB9 | 0xBD => {
                    self.lda(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }
                
                // LDX opcodes
                0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => {
                    todo!("
                    self.ldx(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                    ")
                }
                
                // LDY opcodes
                0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => {
                    todo!("
                    self.ldy(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                    ")
                }
                
                // LSR opcodes
                0x4A | 0x46 | 0x56 | 0x4E | 0x5E => {
                    todo!("
                    self.lsr(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                    ")
                }
                
                // NOP
                0xEA => todo!("self.nop(),"),

                // ORA opcodes
                0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => {
                    todo!("
                    self.ora(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                    ")
                }
                
                // PHA
                0x48 => todo!("self.pha(),"),
                
                // PHP
                0x08 => todo!("self.php(),"),

                // PLA
                0x68 => todo!("self.pla(),"),

                // PLP
                0x28 => todo!("self.plp(),"),
                
                // ROL opcodes
                0x2A | 0x26 | 0x36 | 0x2E | 0x3E => {
                    todo!("
                    self.rol(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                    ")
                }
                
                // ROR opcodes
                0x6A | 0x66 | 0x76 | 0x6E | 0x7E => {
                    todo!("
                    self.ror(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                    ")
                }

                // RTI
                0x40 => todo!("self.rti(),"),

                // RTS
                0x60 => todo!("self.rts(),"),

                // SBC opcodes
                0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 | 0xF1 => {
                    todo!("
                    self.sbc(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                    ")
                }

                // SEC
                0x38 => todo!("self.sec(),"),

                // SED
                0xF8 => todo!("self.sed(),"),

                // SEI
                0x78 => todo!("self.sei(),"),

                // STA opcodes
                0x81 | 0x85 | 0x8D | 0x91 | 0x95 | 0x99 | 0x9D => {
                    self.sta(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }

                // STX opcodes
                0x86 | 0x96 | 0x8E => {
                    todo!("
                    self.stx(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                    ")
                }

                // STY opcodes
                0x84 | 0x94 | 0x8C => {
                    todo!("
                    self.sty(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                    ")
                }

                // TAX
                0xAA => self.tax(),

                // TAY
                0xA8 => todo!("self.tay(),"),

                // TSX
                0xBA => todo!("self.tsx(),"),

                // TXA
                0x8A => todo!("self.txa(),"),

                // TXS
                0x9A => todo!("self.txs(),"),

                // TYA
                0x98 => todo!("self.tya(),"),

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
