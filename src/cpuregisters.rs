use crate::opcodes::{RelativeAddress, AbsoluteAddress};

#[derive(Clone)]
pub struct CPURegisters {
    accumulator: u8,
    x: u8,
    y: u8,
    // 0 - (C) Carry flag
    // 1 - (Z) Zero flag
    // 2 - (I) Interrupt disable flag
    // 3 - (D) Decimal mode status flag
    // 4 - (B) Set when software interrupt (BRK instruction) is executed
    // 5 - Not used. Supposed to be logical 1 at all times
    // 6 - (V) Overflow flag
    // 7 - (S) Sign flag
    status: u8,
    pc: AbsoluteAddress,
    stack: u16,
    pub rts_counter: u32,
}

pub enum CPUFlags {
    Carry = 0b00000001,
    Zero = 0b00000010,
    InterruptDisable = 0b00000100,
    ClearDecimalMode = 0b00001000,
    BreakCommand = 0b00010000,
    Unused = 0b00100000,
    Overflow = 0b01000000,
    Sign = 0b10000000,
}

impl CPURegisters {
    pub fn new() -> CPURegisters {
        CPURegisters {
            accumulator: 0,
            x: 0,
            y: 0,
            status: 0b00100100, // Processor starts with interrupts disabled
            pc: AbsoluteAddress::from(0),
            stack: 0x01FD,
            rts_counter: 0
        }
    }

    pub fn accumulator(&self) -> u8 {
        self.accumulator
    }

    pub fn x(&self) -> u8 {
        self.x
    }

    pub fn y(&self) -> u8 {
        self.y
    }

    pub fn status(&self) -> u8 {
        self.status
    }

    pub(crate) fn pc(&self) -> AbsoluteAddress {
        self.pc
    }

    pub fn stack(&self) -> u16 {
        self.stack
    }

    pub(crate) fn increment_pc(&mut self) -> AbsoluteAddress {
        let pc = self.pc;
        self.pc = self.pc.offset_wrap(1);

        pc
    }

    pub fn increment_stack(&mut self) {
        self.stack += 1;

        if self.stack > 0x01FF {
            self.stack = 0x0100;
        }
    }

    pub fn decrement_stack(&mut self) {
        if self.stack == 0x0100 {
            self.stack = 0x01FF;
        } else {
            self.stack -= 1;
        }
    }

    pub fn set_accumulator(&mut self, value: u8) {
        self.accumulator = value;

        self.set_flag_if(CPUFlags::Zero, self.accumulator == 0);
        self.set_flag_if(CPUFlags::Sign, (self.accumulator & 0b10000000) == 0b10000000);
    }

    pub fn set_x(&mut self, value: u8) {
        self.x = value;

        self.set_flag_if(CPUFlags::Zero, self.x == 0);
        self.set_flag_if(CPUFlags::Sign, (self.x & 0b10000000) == 0b10000000);
    }

    pub fn set_y(&mut self, value: u8) {
        self.y = value;

        self.set_flag_if(CPUFlags::Zero, self.y == 0);
        self.set_flag_if(CPUFlags::Sign, (self.y & 0b10000000) == 0b10000000);
    }

    pub(crate) fn set_pc(&mut self, value: AbsoluteAddress) {
        self.pc = value;
    }

    pub(crate) fn offset_pcx(&mut self, value: &RelativeAddress) -> bool {
        //TODO Rewrite to remove this wrapper method
        self.offset_pc(value.to_i8())
    }

    pub fn offset_pc(&mut self, value: i8) -> bool {
        let from_page = self.pc.high();
        self.pc = self.pc.offset(value);
        // self.pc = ((self.pc as i32) + value as i32) as u16;
        let to_page = self.pc.high();

        return from_page != to_page; // True if we have crossed page boundaries
    }

    pub fn set_stack(&mut self, value: u16) {
        self.stack = 0x0100 | value;
    }

    pub fn set_status(&mut self, value: u8) {
        self.status = value;
    }

    pub fn flag(&self, flag: CPUFlags) -> bool {
        let f = flag as u8;
        (self.status & f) == f
    }

    pub fn set_flag(&mut self, flag: CPUFlags) {
        self.status |= flag as u8;
    }

    pub(crate) fn clear_flag(&mut self, flag: CPUFlags) {
        self.status = self.status & !(flag as u8);
    }

    pub fn set_flag_if(&mut self, flag: CPUFlags, set: bool) {
        if set {
            self.set_flag(flag);
        } else {
            self.clear_flag(flag);
        }
    }
}