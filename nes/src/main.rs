pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub status: u8, // Use each bit of status as a different flag to be more
    // efficient(?), negative flag is the first bit, second last is zero flag
    pub program_counter: u16,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            status: 0,
            program_counter: 0,
        }
    }

    // The main CPU loop is:
    // Fetch next instruction from memory,
    // Decode the instruction,
    // Execute the instruction,
    // repeat;

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.program_counter = 0;

        loop {
            let opcode = program[self.program_counter as usize];
            self.program_counter += 1;

            match opcode {
                0x00 => { 
                    return;
                }
                0xA9 => {
                    // 0xA9 LDA (Load accumulator) in immediate addressing mode,
                    // 2 bytes, 2 cycles according to the reference table
                    let param = program[self.program_counter as usize];
                    self.program_counter += 1;
                    self.register_a = param;

                    if self.register_a == 0 {
                        self.status = self.status | 0b0000_0010;
                    } else {
                        self.status = self.status & 0b1111_1101;
                    }

                    if self.register_a & 0b1000_0000 != 0 {
                        self.status = self.status | 0b1000_0000;
                    } else {
                        self.status = self.status & 0b0111_1111;
                    }
                }
                0xAA => {
                    // 0xAA TAX (Transfer accumulator to register X) set register_x
                    // to the value in the accumulator, only one addressing mode
                    self.register_x = self.register_a;

                    if self.register_x == 0 {
                        self.status = self.status | 0b0000_0010;
                    } else {
                        self.status = self.status & 0b1111_1101;
                    }

                    if self.register_x & 0b1000_0000 != 0 {
                        self.status = self.status | 0b1000_0000;
                    } else {
                        self.status = self.status & 0b0111_1111;
                    }
                }
                0xE8 => {
                    // 0xE8 INX (Increment Register X) Adds one to the register and
                    // then sets the Zero flag, Negative flag if needed
                    self.register_x = self.register_x + 1;

                    if self.register_x == 0 {
                        self.status = self.status | 0b0000_0010;
                    } else {
                        self.status = self.status & 0b1111_1101;
                    }

                    if self.register_x & 0b1000_0000 != 0 {
                        self.status = self.status | 0b1000_0000;
                    } else {
                        self.status = self.status & 0b0111_1111;
                    }
                }
                _ => todo!("Build out the massive switch statement for opcodes")
            }
        }
    }
}

fn main() {
    
}

#[cfg(test)]
mod test {
    use super::*;

    // 0xA9
    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0xF0, 0x00]);
        assert!(cpu.status & 0b1000_0000 == 0b1000_0000);
    }

    // 0xAA, could have just set register_a manually lol
    #[test]
    fn test_0xaa_tax() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        cpu.status = 0;
        cpu.interpret(vec![0xaa, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xaa_tax_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x00, 0x00]);
        cpu.status = 0;
        cpu.interpret(vec![0xaa, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_0xaa_tax_negative_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0xF0, 0x00]);
        cpu.status = 0;
        cpu.interpret(vec![0xaa, 0x00]);
        assert!(cpu.status & 0b1000_0000 == 0b1000_0000);
    }
    
    // 0xE8 
    #[test]
    fn test_0xe8_inx() {
        let mut cpu = CPU::new();
        cpu.register_x = 1;
        cpu.interpret(vec![0xe8, 0x00]);
        assert!(cpu.register_x == 2);
        // double check flags, aka finish the test
        assert!(cpu.status & 0b0000_0010 == 0b10);
    }

    // TODO: Modify the next two tests, at the moment they were just copied from 
    // the last set of tests for 0xA9

    #[test]
    fn test_0xe8_inx_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x00, 0x00]);
        cpu.status = 0;
        cpu.interpret(vec![0xaa, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_0xe8_inx_negative_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0xF0, 0x00]);
        cpu.status = 0;
        cpu.interpret(vec![0xaa, 0x00]);
        assert!(cpu.status & 0b1000_0000 == 0b1000_0000);
    }
}
