use std::usize;

use crate::opcodes::{init_opcodes, init_opcodes_hashmap};
use crate::bus::Bus;

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
const DECIMAL_MODE: u8 = 0b0000_1000; // not used on nes, still an instruction that clears it
const BREAK_BIT: u8 = 0b0001_0000;
const NOT_A_FLAG_BIT: u8 = 0b0010_0000; // Doesn't represent any flag
const OVERFLOW_BIT: u8 = 0b0100_0000;
const NEGATIVE_BIT: u8 = 0b1000_0000;

#[derive(Debug)]
pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8, 
    pub program_counter: u16,
    pub stack_pointer: u8, // This points to the top of the stack, decrementing
    // when a byte of data is pushed to the stack and incrementing when popped
    pub bus: Bus,
}

const STACK: u16 = 0x0100; // Starting address for the stack in the NES in memory
const STACK_RESET_CODE: u8 = 0xFD;

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
    NoneAddressing,
}

// Take some of the common functions and rewrite them into traits.
pub trait Memory {
    fn mem_read(&self, addr: u16) -> u8;

    fn mem_write(&mut self, addr: u16, data: u8);

    fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xFF) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }
}

impl Memory for CPU {
    fn mem_read(&self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.mem_write(addr, data)
    }

    fn mem_read_u16(&self, pos: u16) -> u16 {
        self.bus.mem_read_u16(pos)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        self.bus.mem_write_u16(pos, data)
    }
}

impl CPU {
    pub fn new(bus_: Bus) -> Self {
        CPU {
            register_a: 0, // accumulator but I can't be bothered to change the name atm
            register_x: 0,
            register_y: 0,
            status: 0 | INTERRUPT_DISABLE_BIT | NEGATIVE_BIT, // 8 bit register, representing 7 flags
            program_counter: 0,
            stack_pointer: STACK_RESET_CODE, // The stack in the nes is 256 bytes and stored in 
            bus: bus_,
        }
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

            AddressingMode::NoneAddressing => {
                // replace the panic with something else maybe? No reason for 
                // program to panic if an addressing mode isn't needed, for example 
                // TAX transferring the accumulator value to register_x
                panic!("This addressing mode doesn't exist {:?}\n", mode);
            }
        }
    }

    // read and pop a value off of the stack, called pulling on nesdev
    pub fn stack_pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1); 
        self.mem_read((STACK as u16) + self.stack_pointer as u16) // stack_pointer is a mem address directly
    }

    pub fn stack_push(&mut self, data: u8) {
        self.mem_write((STACK as u16) + self.stack_pointer as u16, data);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1); 
    }

    pub fn stack_push_u16(&mut self, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xFF) as u8;
        self.stack_push(hi);
        self.stack_push(lo);
    }

    pub fn stack_pop_u16(&mut self) -> u16 {
        let lo = self.stack_pop() as u16;
        let hi = self.stack_pop() as u16;

        hi << 8 | lo
    }

    pub fn branch(&mut self, condition: bool) {
        if condition {
            let jump: i8 = self.mem_read(self.program_counter) as i8;
            let jump_addr = self
                .program_counter
                .wrapping_add(1)
                .wrapping_add(jump as u16);

            self.program_counter = jump_addr;
        }
    }

    // ADC, add with carry, reading the value of a given address, add the value 
    // to the accumulator with the carry bit, if overflow occurs, carry bit is
    // set enabling multiple byte addition
    pub fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value_to_add = self.mem_read(addr);

        // save the sum, to be able to properly set the necessary flags
        let sum = (self.register_a as u16) + (value_to_add as u16) + (if self.status & CARRY_BIT == CARRY_BIT { 1 } else { 0 }) as u16;

        let carry = sum > 0xFF;

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
        self.register_a = self.register_a & value; 
        self.set_zero_and_neg_flags(self.register_a);
    }

    // ASL - Arithmetic Shift Left, the operation shifts all bits of the accumulator (register_a)
    // or the memory contents one bit to the left, bit 7 is placed into the carry 
    // flag and bit 0 is set to 0. Zero and Negative flags also need to be updated
    pub fn asl(&mut self, mode: &AddressingMode) {
        let mut value_to_modify: u8;
        let mut addr: u16 = 0;
        if matches!(mode, AddressingMode::NoneAddressing) {
            // modify accumulator directly
            value_to_modify = self.register_a;
        } else {
            addr = self.get_operand_address(mode);
            value_to_modify = self.mem_read(addr);
        }

        // shift left one bit after saving bit 7 as the carry bit
        // Carry bit is the 0th bit so this won't work, probably a better way
        // to determine if the 7th bit is set or not
        // if value_to_modify & NEGATIVE_BIT == NEGATIVE_BIT {
        if value_to_modify >> 7 == 1 {
            // can instead call self.set_carry_flag()
            self.status = self.status | CARRY_BIT
        } else {
            // can instead call self.clear_carry_flag()
            self.status = self.status & !CARRY_BIT;
        }

        // flag is set, shift it over by one, then set the zero and negative flags
        value_to_modify = value_to_modify << 1;

        if matches!(mode, AddressingMode::NoneAddressing) {
            // modify accumulator directly
            self.register_a = value_to_modify;
        } else {
            // this should only ever write to memory to the proper location, should
            // never run if addressingMode is Accumulator
            self.mem_write(addr, value_to_modify);
        }

        self.set_zero_and_neg_flags(value_to_modify);
    }

    // BCC - Branch if carry clear: if the carry flag is clear, add the relative
    // displacement to the program counter to cause a branch to a new location
    // absolutely no idea what that means
    pub fn bcc(&mut self) {
        self.branch(self.status & CARRY_BIT != CARRY_BIT);
    }

    // BCS - Branch if carry set: If the carry flag is set, add the relative displacement
    // to the program counter to cause a branch to a new location assuming this is the
    // opposite of BCC
    pub fn bcs(&mut self) {
        self.branch(self.status & CARRY_BIT == CARRY_BIT);
    }

    // BEQ - Branch if equal: if the zero flag is set then add the relative displacement
    // to the program counter to cause a branch to a new location
    pub fn beq(&mut self) {
        self.branch(self.status & ZERO_BIT == ZERO_BIT);
    }

    // BIT - bit test: used to test if one or more bits are set in a target memory location.
    // The mask pattern in the Accumulator (register_a) is ANDed with the value in memory to 
    // set or clear the zero flag, without keeping the result. Bits 7 and 6 of the value in
    // memory are copied into the Negative and Overflow flags respectively
    pub fn bit(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode); // should only be zero page and absolute
        let value_in_memory = self.mem_read(addr);
        
        // set the zero flag
        let anded_value = value_in_memory & self.register_a;
        if anded_value == 0 {
            self.status = self.status | ZERO_BIT;
        } else {
            self.status = self.status & !ZERO_BIT;
        }

        // copy bit values into overflow and negative flags
        let new_overflow = value_in_memory & OVERFLOW_BIT;
        if new_overflow > 0 {
            self.status = self.status | OVERFLOW_BIT;
        } else {
            self.status = self.status & !OVERFLOW_BIT;
        }

        let new_negative = value_in_memory & NEGATIVE_BIT;
        if new_negative > 0 {
            self.status = self.status | NEGATIVE_BIT;
        } else {
            self.status = self.status & !NEGATIVE_BIT;
        }
        // There's gotta be a better way to set these flags than repeating this verbose
        // method for each flag toggle in the emulator. But at least it should be obvious
        // what it's doing each time. So it should be hard to not understand this in the future

    }

    // BMI - Branch if Minus: if the negative flag is set then add the relative
    // displacement to the program_counter to cause a branch to a new location
    // just like the other branch instructions I need to implement relative addressing and
    // find out what is meant by branching.
    pub fn bmi(&mut self) {
        self.branch(self.status & NEGATIVE_BIT == NEGATIVE_BIT);
    }

    // BNE - Branch if not equal: if zero flag is clear, add relative displacement to the 
    // program counter to cause a branch to a new location.
    pub fn bne(&mut self) {
        self.branch(self.status & ZERO_BIT != ZERO_BIT);
    }

    // BPL - Branch if Positive: if the negative flag is clear then add the relative 
    // displacement to the program counter to cause a branch to a new location
    pub fn bpl(&mut self) {
        self.branch(self.status & NEGATIVE_BIT != NEGATIVE_BIT);
    }
    
    // BRK - Force interrupt: Program counter and processor status are pushed on the stack
    // then the IRQ interrupt vector at $FFFE/F is loaded into the PC and the break flag in
    // the status is set to one.
    pub fn brk(&mut self) {
        self.mem_write_u16(self.stack_pointer.into(), self.program_counter);
        self.mem_write(self.stack_pointer.wrapping_add(2).into(), self.status);
        self.stack_pointer = self.stack_pointer.wrapping_add(3);
        self.status = self.status | BREAK_BIT;
        self.program_counter = 0xFFFE;
        return 
    }

    // BVC - Branch if Overflow clear: if the overflow flag is clear then add the relative
    // displacement to the program counter to cause a branch to a new location
    pub fn bvc(&mut self) {
        self.branch(self.status & OVERFLOW_BIT != OVERFLOW_BIT);
    }

    // BVS - Branch if Overflow set: if the overflow flag is set then add the relative
    // displacement to the program counter to cause a branch to a new location
    pub fn bvs(&mut self) {
        self.branch(self.status & OVERFLOW_BIT == OVERFLOW_BIT);
    }

    // CLC - Clear Carry Flag: Set the carry flag to 0
    pub fn clc(&mut self) {
        // simple enough I guess.
        self.status = self.status & !CARRY_BIT;
    }

    // CLD - Clear decimal mode: Set the decimal mode flag to 0.
    pub fn cld(&mut self) {
        self.status = self.status & !DECIMAL_MODE;
    }

    // CLI - Clear interrupt disable flag, this allows normal interrupt requests to 
    // be serviced again.
    pub fn cli(&mut self) {
        self.status = self.status & !INTERRUPT_DISABLE_BIT;
    }

    // CLV - Clear overflow flag,
    pub fn clv(&mut self) {
        self.status = self.status & !OVERFLOW_BIT;
    }

    // CMP - Compare: The instruction compares the contents of the accumulator (register_a)
    // with another memory held value and sets the zero, negative, and carry flags as needed.
    pub fn cmp(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        if self.register_a >= value {
            self.status = self.status | CARRY_BIT;
        } else {
            self.status = self.status & !CARRY_BIT;
        }

        // this might be extremely incorrect implementation of what the instruction is 
        // actually asking for. TODO: CHECK IF MUTATING
        let diff_in_values = self.register_a.wrapping_sub(value);
        self.set_zero_and_neg_flags(diff_in_values);

    }

    // CPX - Compare X register: the instruction compares the contents of the X register
    // with another memory held value setting carry, zero, and negative flags
    pub fn cpx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        if self.register_x >= value {
            self.status = self.status | CARRY_BIT;
        } else {
            self.status = self.status & !CARRY_BIT;
        }

        // this might be extremely incorrect implementation of what the instruction is 
        // actually asking for. I'm really hoping this isn't modifying the value of 
        // register_x, I'm pretty sure that it isn't meant to. TODO: CHECK IF MUTATING
        let diff_in_values = self.register_x.wrapping_sub(value);
        self.set_zero_and_neg_flags(diff_in_values);
    }

    // CPY - Compare Y register: the instruction compares the contents of the Y register
    // with another memory held value setting carry, zero, and negative flags
    pub fn cpy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        if self.register_y >= value {
            self.status = self.status | CARRY_BIT;
        } else {
            self.status = self.status & !CARRY_BIT;
        }

        // this might be extremely incorrect implementation of what the instruction is 
        // actually asking for. I'm really hoping this isn't modifying the value of 
        // register_x, I'm pretty sure that it isn't meant to. TODO: CHECK IF MUTATING
        let diff_in_values = self.register_y.wrapping_sub(value);
        self.set_zero_and_neg_flags(diff_in_values);
    }

    // DEC - Decrement memory: Subtract one from the value held a the specified memory
    // location setting zero and negative flags as needed overflow is ignored for some reason.
    pub fn dec(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut value = self.mem_read(addr);

        value = value.wrapping_sub(1);
        self.mem_write(addr, value);

        self.set_zero_and_neg_flags(value);
    }

    // DEX - Decrement X register: Subtract one from the value held in register_x
    // setting zero and negative flags as needed overflow is ignored for some reason.
    pub fn dex(&mut self) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.set_zero_and_neg_flags(self.register_x);
    }

    // DEY - Decrement Y register: Subtract one from the value held in register_y
    // setting zero and negative flags as needed overflow is ignored for some reason.
    pub fn dey(&mut self) {

        self.register_y = self.register_y.wrapping_sub(1);
        self.set_zero_and_neg_flags(self.register_y);
    }

    // EOR - Exclusive OR: Perform an exclusive or on the accumulator (register_a) and the 
    // value held in a specified memory location
    pub fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = self.register_a ^ value;
        self.set_zero_and_neg_flags(self.register_a);
    }

    // INC - Increment the value held at a specified memory address, by one, 
    // set the zero and negative flags from the result, guide returns this value
    // for some reason
    pub fn inc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut value = self.mem_read(addr);

        value = value.wrapping_add(1);

        self.mem_write(addr, value);
        self.set_zero_and_neg_flags(value);
    }

    // INX (Increment Register X) Adds one to the register and
    // then sets the Zero flag, Negative flag if needed
    pub fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.set_zero_and_neg_flags(self.register_x);
    }

    // INY - Increment Register Y; setting flags
    pub fn iny(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);
        self.set_zero_and_neg_flags(self.register_y);
    }

    // JMP - Jump, setting the program counter to the address specified
    // in memory, no flags are affected
    pub fn jmp(&mut self, mode: &AddressingMode) {
        if matches!(mode, AddressingMode::Immediate) {
            // Absolute JMP
            let addr = self.mem_read_u16(self.program_counter);
            self.program_counter = addr;
        } else {
            // Indirect JMP
            let mem_addr = self.mem_read_u16(self.program_counter);

            let indirect_ref = if mem_addr & 0x00FF == 0x00FF {
                let lo = self.mem_read(mem_addr);
                let hi = self.mem_read(mem_addr & 0xFF00);
                (hi as u16) << 8 | (lo as u16)
            } else {
                self.mem_read_u16(mem_addr)
            };

            self.program_counter = indirect_ref;
        }
    }

    // JSR - Jump to a subroutine: pushes the address (minus 1) of the return point on to the stack 
    // then sets the program counter to the target memory address
    // I'm calling this straight from the match statement in the run_with_callback function
    // pub fn jsr(&mut self) {
        // self.stack_push_u16((self.program_counter + 2) - 1);
        // let target_address = self.mem_read_u16(self.program_counter);
        // self.program_counter = target_address; 
    // }

    // LDA that takes in different AddressingModes
    // loads a byte of memory into the accumulator (register_a) and sets zero and neg flags
    // 0xA9, 0xA5, 0xB5, 0xAD, 0xBD, 0xB9, 0xA1, 0xB1
    pub fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = value;
        self.set_zero_and_neg_flags(self.register_a);
    }

    // LDX - Load register_x; setting zero and negative flags as needed.
    pub fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_x = value;
        self.set_zero_and_neg_flags(self.register_x);
    }

    // LDY - Load register_y; setting zero and negative flags as needed.
    pub fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_y = value;
        self.set_zero_and_neg_flags(self.register_y);
    }

    // LSR - Logical Shift Right: each of the bits in the accumulator or at the memory
    // location is shifted one place to the right, former bit 0 is stored in the carry flag,
    // bit 7 is set to 0, in the guide he returns value_to_modify if not using Accumulator mode
    pub fn lsr(&mut self, mode: &AddressingMode) {
        let mut value_to_modify: u8;
        let mut addr: u16 = 0;
        if matches!(mode, AddressingMode::NoneAddressing) {
            // modify accumulator directly
            value_to_modify = self.register_a;
        } else {
            addr = self.get_operand_address(mode);
            value_to_modify = self.mem_read(addr);
        }

        // shift right one bit after saving bit 0 as the carry bit
        if value_to_modify & CARRY_BIT == CARRY_BIT {
            // can use self.set_carry_flag()
            self.status = self.status | CARRY_BIT
        } else {
            // can use self.clear_carry_flag()
            self.status = self.status & !CARRY_BIT;
        }

        // flag is set, shift it over by one, then set the zero and negative flags
        // TODO: READ DOCUMENTATION ABOUT BIT SHIFTING TO ENSURE THIS ACTUALLY
        // DOES WHAT I WANT IT TO DO
        value_to_modify = value_to_modify >> 1;

        self.set_zero_and_neg_flags(value_to_modify);

        if matches!(mode, AddressingMode::NoneAddressing) {
            // modify accumulator directly
            self.register_a = value_to_modify;
        } else {
            // this should only ever write to memory to the proper location, should
            // never run if addressingMode is Accumulator
            self.mem_write(addr, value_to_modify);
        }
    }

    // NOP - Do nothing, just allow the program_counter to increment
    pub fn nop(&mut self) {
        return;
    }

    // ORA - Logical inclusive or on the accumulator with the value stored in memory
    // set the zero and negative flags after
    pub fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = self.register_a | value;
        self.set_zero_and_neg_flags(self.register_a);
    }

    // PHA - Push Accumulator; Pushes a copy of the accumulator onto the stack
    pub fn pha(&mut self) {
        self.stack_push(self.register_a);
    }

    // PHP - Push Processor Status; Pushes a copy of the cpu status onto the stack, nesdev
    // says flags are not set at all with this instruction, guide sets both break and NOT_A_FLAG BITs
    pub fn php(&mut self) {
        let mut cur_flags = self.status.clone();
        cur_flags = cur_flags | BREAK_BIT | NOT_A_FLAG_BIT;  
        self.stack_push(cur_flags);
    }

    // PLA - Pull Accumulator: Pull an 8 bit value from the stack and into the 
    // accumulator, setting zero and negative flags based on the value in the accumulator
    pub fn pla(&mut self) {
        self.register_a = self.stack_pop();
        self.set_zero_and_neg_flags(self.register_a);
    }

    // PLP - Pull Processor Status: Pull an 8 bit value from the stack and into the 
    // CPU status, setting zero and negative flags based on the value in the cpu status
    // nesdev says to set all flags from the value pulled from the stack, guide sets NOT_A_FLAG_BIT
    // and clears BREAK_BIT
    pub fn plp(&mut self) {
        self.status = self.stack_pop();
        self.status = (self.status | NOT_A_FLAG_BIT) & !BREAK_BIT;
    }

    // ROL - Rotate left: Move each of the bits in either Accumulator or Memory one place 
    // to the left. Bit 0 is filled with the current value of the carry flag whilst the old bit 
    // 7 becomes the new carry flag value. TODO: Double check the order of operations here
    // this is broken, i misread the bits that are set as carry flag and which is filled with old
    // carry flag, redo the shifting.
    pub fn rol(&mut self, mode: &AddressingMode) {
        let mut value_to_modify: u8;
        let mut addr: u16 = 0;
        if matches!(mode, AddressingMode::NoneAddressing) {
            // modify accumulator directly
            value_to_modify = self.register_a;
        } else {
            addr = self.get_operand_address(mode);
            value_to_modify = self.mem_read(addr);
        }

        let is_carry_set: bool = self.status & CARRY_BIT == CARRY_BIT;

        // shift left one bit after saving bit 0 as the carry bit
        // if value_to_modify & CARRY_BIT == CARRY_BIT {
        if value_to_modify >> 7 == 1 {
            self.status = self.status | CARRY_BIT
        } else {
            self.status = self.status & !CARRY_BIT;
        }

        // Now we shift left and set the 0th bit to the saved value from earlier
        value_to_modify = value_to_modify << 1;
        if is_carry_set {
            value_to_modify = value_to_modify | 1;
        } // else rust should have already set it to zero when shifting, I think
        // TODO: DOUBLE CHECK RUST DEFAULT BEHAVIOUR ON SHIFTING

        self.set_zero_and_neg_flags(value_to_modify);

        if matches!(mode, AddressingMode::NoneAddressing) {
            // modify accumulator directly
            self.register_a = value_to_modify;
        } else {
            // this should only ever write to memory to the proper location, should
            // never run if addressingMode is Accumulator
            self.mem_write(addr, value_to_modify);
        }
    }

    // ROR - rotate right, same as rol, only shift right, fill bit 7 with carry flag, and 
    // old bit 0 is new carry flag. This is also broken possibly, redo shifts and flags
    pub fn ror(&mut self, mode: &AddressingMode) {
        let mut value_to_modify: u8;
        let mut addr: u16 = 0;
        if matches!(mode, AddressingMode::NoneAddressing) {
            // modify accumulator directly
            value_to_modify = self.register_a;
        } else {
            addr = self.get_operand_address(mode);
            value_to_modify = self.mem_read(addr);
        }

        let is_carry_set = self.status & CARRY_BIT == CARRY_BIT;
        if value_to_modify & 1 == 1 {
            self.status = self.status | CARRY_BIT;
        } else {
            self.status = self.status & !CARRY_BIT;
        }

        // Now we shift right and set the 0th bit to the saved value from earlier
        value_to_modify = value_to_modify >> 1;
        if is_carry_set {
            value_to_modify = value_to_modify | CARRY_BIT;
        } // else rust should have already set it to zero when shifting, I think
        // TODO: DOUBLE CHECK RUST DEFAULT BEHAVIOUR ON SHIFTING

        self.set_zero_and_neg_flags(value_to_modify);

        if matches!(mode, AddressingMode::NoneAddressing) {
            // modify accumulator directly
            self.register_a = value_to_modify;
        } else {
            // this should only ever write to memory to the proper location, should
            // never run if addressingMode is Accumulator
            self.mem_write(addr, value_to_modify);
        }
    }

    // RTI - Return from Interrupt: Used at the end of an interrupt processing routine,
    // pulls the processor flags from the stack followed by the program counter, guide 
    // sets break and not a flag manually, nesdev says just keep the values pulled from stack
    pub fn rti(&mut self) {
        self.status = self.stack_pop();
        self.status = self.status & !BREAK_BIT;
        self.status = self.status | NOT_A_FLAG_BIT;

        self.program_counter = self.stack_pop_u16();
    }

    // RTS - Return from subroutine: Used at the end of a subroutine,
    // pulls the program counter (minus 1) from the stack
    pub fn rts(&mut self) {
        self.program_counter = self.stack_pop_u16() + 1;
    }

    // SBC - Subtract with Carry: Subtracts the contents of a memory location to the accumulator 
    // with the not of the carry bit, if overflow occurs, the carry bit is clear, enabling multi 
    // byte subtraction to be performed. (A - M -(1-C));
    pub fn sbc(&mut self, mode: &AddressingMode) {
        // A - B = A + (-B) = A + (!B + 1);
        // Use the code from adc, and just change the value read from memory
        let addr = self.get_operand_address(mode);
        // TODO: determine if this is working and not breaking in an unexpected way.
        let mut value_to_add = self.mem_read(addr);
        value_to_add = (value_to_add as i8).wrapping_neg().wrapping_sub(1) as u8;

        // save the sum, to be able to properly set the necessary flags
        let sum = (self.register_a as u16) + (value_to_add as u16) + (if self.status & CARRY_BIT == CARRY_BIT { 1 } else { 0 } as u16);

        let carry = sum > 0xFF;

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

    // SEC - Set carry flag: set the carry flag to 1;
    pub fn sec(&mut self) {
        self.status = self.status | CARRY_BIT;
    }

    // SED - Set decimal flag;
    pub fn sed(&mut self) {
        self.status = self.status | DECIMAL_MODE;
    }

    // SEI - Set interrupt disable flag;
    pub fn sei(&mut self) {
        self.status = self.status | INTERRUPT_DISABLE_BIT;
    }

    // STA, copies value from register A into memory
    pub fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    // STX, copies value from register X into memory
    pub fn stx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x);
    }

    // STY, copies value from register Y into memory
    pub fn sty(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_y);
    }

    // 0xAA TAX (Transfer accumulator to register X) set register_x
    // to the value in the accumulator, only one addressing mode
    pub fn tax(&mut self) {
        self.register_x = self.register_a;
        self.set_zero_and_neg_flags(self.register_x);
    }

    // TAY (Transfer accumulator to register Y) set register_y
    // to the value in the accumulator, only one addressing mode
    pub fn tay(&mut self) {
        self.register_y = self.register_a;
        self.set_zero_and_neg_flags(self.register_y);
    }

    // TSX - transfer stack pointer to X
   // copies current contents of the stack register into the X register, setting 
    // zero and negative flags
    pub fn tsx(&mut self) {
        self.register_x = self.stack_pointer;
        self.set_zero_and_neg_flags(self.register_x);
    }

    // TXA - transfer x to accumulator;
    // Copies the current contents of the x register into the accumulator, set zero & neg flags
    pub fn txa(&mut self) {
        self.register_a = self.register_x;
        self.set_zero_and_neg_flags(self.register_a);
    }
    
    // TXS - transfer x to stack pointer;
    // Copies the current contents of the x register into the stack register 
    pub fn txs(&mut self) {
        self.stack_pointer = self.register_x;
    }

    // TYA transfer reg_y to accumulator; setting flags as needed
    pub fn tya(&mut self) {
        self.register_a = self.register_y;
        self.set_zero_and_neg_flags(self.register_a);
    }

    pub fn set_zero_and_neg_flags(&mut self, result: u8) {

        // Set the Zero flag
        if result == 0 {
            self.status = self.status | ZERO_BIT;
        } else {
            self.status = self.status & !ZERO_BIT;
        }

        // Set the Negative flag
        // if result & 0b1000_0000 != 0 {
        if result >> 7 == 1 {
            self.status = self.status | NEGATIVE_BIT;
        } else {
            self.status = self.status & !NEGATIVE_BIT;
        }

    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        // For testing purposes only, delete line setting program_counter when running
        // self.program_counter = self.mem_read_u16(0x0600);
        self.run();
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.status = 0 | INTERRUPT_DISABLE_BIT | NEGATIVE_BIT;
        self.stack_pointer = STACK_RESET_CODE;
        // Not going to reset memory yet because I'd need to rewrite tests to call memory writing
        // in machine code
        // self.memory = [0; 0xFFFF];

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        // Then NES typically uses 0x8000-0xFFFF for loading in the cartridge ROM
        // self.memory[0x0600..(0x0600 + program.len())].copy_from_slice(&program[..]);
        // self.mem_write_u16(0xFFFC, 0x0600); // The NES reads the address that is stored here
        // and sets the program counter to this address stored at 0xFFFC to begin running.
        for i in 0..(program.len() as u16) {
            self.mem_write(0x8600 + i, program[i as usize]);
        }
        // self.mem_write_u16(0xFFFC, 0x8600);
    }

    // The main CPU loop is:
    // Fetch next instruction from memory,
    // Decode the instruction,
    // Execute the instruction,
    // repeat;

    pub fn run(&mut self) {
        self.run_with_callback(|_| {}); // What is this parameter?? :O
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where 
        F: FnMut(&mut CPU),
    {
        init_opcodes();
        // might as well remove the hashmap? But the method gets_or_inits the pub static
        // hashmap so maybe it is needed, I have no idea what is happening behind the curtain 
        let other_map = init_opcodes_hashmap();

        loop {

            let opcode = self.mem_read(self.program_counter);
            self.program_counter += 1; // could wrapping add this maybe?
            let program_counter_state = self.program_counter;

            match opcode {
                // BRK 
                0x00 => return, // self.brk(),

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
                0x90 => self.bcc(),
                
                // BCS
                0xB0 => self.bcs(),
                
                // BEQ
                0xF0 => self.beq(),
                
                // BIT opcodes
                0x24 | 0x2C => {
                    self.bit(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1
                }

                // BMI
                0x30 => self.bmi(),

                // BNE
                0xD0 => self.bne(),

                // BPL
                0x10 => self.bpl(),
                
                // BVC
                0x50 => self.bvc(),

                // BVS
                0x70 => self.bvs(),

                // CLC
                0x18 => self.clc(),

                // CLD
                0xD8 => self.cld(),

                // CLI
                0x58 => self.cli(),

                // CLV
                0xB8 => self.clv(),

                // CMP opcodes
                0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => {
                    self.cmp(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }

                // CPX opcodes
                0xE0 | 0xE4 | 0xEC => {
                    self.cpx(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }

                // CPY opcodes
                0xC0 | 0xC4 | 0xCC => {
                    self.cpy(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }

                // DEC opcodes
                0xC6 | 0xD6 | 0xCE | 0xDE => {
                    self.dec(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }
                
                // DEX
                0xCA => self.dex(),

                // DEY
                0x88 => self.dey(),
                
                // EOR opcodes
                0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => {
                    self.eor(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }

                // INC opcodes
                0xE6 | 0xF6 | 0xEE | 0xFE => {
                    self.inc(&other_map[&opcode].addressing_mode);
                    self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }

                // INX
                0xE8 => self.inx(),
                
                // INY
                0xC8 => self.iny(),

                // JMP 
                0x4C | 0x6C => {
                    self.jmp(&other_map[&opcode].addressing_mode);
                    // self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }
                
                // JSR
                0x20 => {
                    self.stack_push_u16(self.program_counter + 2 - 1);
                    let target_address = self.mem_read_u16(self.program_counter);
                    self.program_counter = target_address;
                },
                //self.jsr(),

                // LDA opcodes
                0xA1 | 0xA5 | 0xA9 | 0xAD | 0xB1 | 0xB5 | 0xB9 | 0xBD => {
                    print!("addressing_mode for lda: {:?}", &other_map[&opcode].addressing_mode);
                    self.lda(&other_map[&opcode].addressing_mode);
                    // self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }
                
                // LDX opcodes
                0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => {
                    self.ldx(&other_map[&opcode].addressing_mode);
                    // self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }
                
                // LDY opcodes
                0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => {
                    self.ldy(&other_map[&opcode].addressing_mode);
                    // self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }
                
                // LSR opcodes
                0x4A | 0x46 | 0x56 | 0x4E | 0x5E => {
                    self.lsr(&other_map[&opcode].addressing_mode);
                    // self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }
                
                // NOP
                0xEA => self.nop(),

                // ORA opcodes
                0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => {
                    self.ora(&other_map[&opcode].addressing_mode);
                    // self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }
                
                // PHA
                0x48 => self.pha(),
                
                // PHP
                0x08 => self.php(),

                // PLA
                0x68 => self.pla(),

                // PLP
                0x28 => self.plp(),
                
                // ROL opcodes
                0x2A | 0x26 | 0x36 | 0x2E | 0x3E => {
                    self.rol(&other_map[&opcode].addressing_mode);
                    // self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }
                
                // ROR opcodes
                0x6A | 0x66 | 0x76 | 0x6E | 0x7E => {
                    self.ror(&other_map[&opcode].addressing_mode);
                    // self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }

                // RTI
                0x40 => self.rti(),

                // RTS
                0x60 => self.rts(),

                // SBC opcodes
                0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 | 0xF1 => {
                    self.sbc(&other_map[&opcode].addressing_mode);
                    // self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }

                // SEC
                0x38 => self.sec(),

                // SED
                0xF8 => self.sed(),

                // SEI
                0x78 => self.sei(),

                // STA opcodes
                0x81 | 0x85 | 0x8D | 0x91 | 0x95 | 0x99 | 0x9D => {
                    self.sta(&other_map[&opcode].addressing_mode);
                    // self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }

                // STX opcodes
                0x86 | 0x96 | 0x8E => {
                    self.stx(&other_map[&opcode].addressing_mode);
                    // self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }

                // STY opcodes
                0x84 | 0x94 | 0x8C => {
                    self.sty(&other_map[&opcode].addressing_mode);
                    // self.program_counter += (other_map[&opcode].bytes as u16) - 1;
                }

                // TAX
                0xAA => self.tax(),

                // TAY
                0xA8 => self.tay(),

                // TSX
                0xBA => self.tsx(),

                // TXA
                0x8A => self.txa(),

                // TXS
                0x9A => self.txs(),

                // TYA
                0x98 => self.tya(),

                _ => {
                    self.program_counter = self.program_counter.wrapping_add(1);
                    print!("Build out the massive switch statement for opcodes, this time it broke on {:#04x} \n", opcode);
                    return;
                }
            }

            if program_counter_state == self.program_counter {
                self.program_counter += (other_map[&opcode].bytes - 1) as u16;
            }

            callback(self);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cartridge::test;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let bus = Bus::new(test::test_rom());
        let mut cpu = CPU::new(bus);
        print!("CPU dump before load_and_run: {:?}", cpu);
        dbg!(cpu.load_and_run(vec![0xa9, 0x05, 0x00]));
        print!("CPU dump after load_and_run: {:?}", cpu);
        assert_eq!(cpu.register_a, 5);
        // assert!(cpu.status & 0b0000_0010 == 0b00);
        // assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let bus = Bus::new(test::test_rom());
        let mut cpu = CPU::new(bus);
        cpu.register_a = 10;
        cpu.load_and_run(vec![0xaa, 0x00]);

        assert_eq!(cpu.register_x, 10)
    }

    #[test]
    fn test_5_ops_working_together() {
        let bus = Bus::new(test::test_rom());
        let mut cpu = CPU::new(bus);
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let bus = Bus::new(test::test_rom());
        let mut cpu = CPU::new(bus);
        cpu.register_x = 0xff;
        cpu.load_and_run(vec![0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }

    #[test]
    fn test_lda_from_memory() {
        let bus = Bus::new(test::test_rom());
        let mut cpu = CPU::new(bus);
        cpu.mem_write(0x10, 0x55);

        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

        assert_eq!(cpu.register_a, 0x55);
    }
}
