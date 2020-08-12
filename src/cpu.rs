use crate::cpuregisters::{CPURegisters};
use crate::ram_controller::RamController;
use crate::{opcodes, stack};
use crate::opcodes::{OpCode, InstructionType, AddressingMode, ZeroPageAddress, AbsoluteAddress, RelativeAddress, ValueAddressingResult, AddressingResult};
use crate::opcodes::AddressingMode::{ZeroPage, Relative, Absolute, Immediate, Indirect, ZeroPageX, ZeroPageY, Implicit, Accumulator, AbsoluteX, AbsoluteY, IndexedIndirect, IndirectIndexed};
use sdl2::render::BlendMode::Add;

const NMI_VECTOR: u16 = 0xFFFA;
const RESET_VECTOR: u16 = 0xFFFC;

pub struct CPU<'ctl, 'data:'ctl> {
    pub registers: CPURegisters,
    memory: &'ctl mut RamController<'data>,
}


pub trait ValueSource {
    fn get_value(&self) -> u8;
    fn set_value(&mut self, val: u8);
}

pub struct MemoryValueSource<'ctl, 'data:'ctl> {
    address: AbsoluteAddress,
    memory: &'ctl mut RamController<'data>,
}

impl ValueSource for MemoryValueSource<'_, '_> {
    fn get_value(&self) -> u8 {
        self.memory.read8x(&self.address)
    }

    fn set_value(&mut self, val: u8) {
        self.memory.write8x(&self.address, val);
    }
}

pub struct AccumulatorValueSource<'b> {
    registers: &'b mut CPURegisters
}

pub(crate) enum ValueSourceResult<'b, 'ctl, 'data:'ctl> {
    AccumulatorX(AccumulatorValueSource<'b>),
    MemoryX(MemoryValueSource<'ctl, 'data>),
}

impl ValueSource for AccumulatorValueSource<'_> {
    fn get_value(&self) -> u8 {
        self.registers.accumulator()
    }

    fn set_value(&mut self, val: u8) {
        self.registers.set_accumulator(val);
    }
}

impl<'ctl, 'data:'ctl> CPU<'ctl, 'data> {
    pub(crate) fn new(mem: &'ctl mut RamController<'data>) -> CPU<'ctl, 'data> {
        CPU {
            memory: mem,
            registers: CPURegisters::new(),
        }
    }
    pub(crate) fn reset(&mut self) {
        self.registers = CPURegisters::new();

        let address = self.memory.read16(RESET_VECTOR);
        self.registers.set_pc(address);

        // Only used with nestest
        // self.registers.set_pc(0xC000);
    }

    pub(crate) fn trigger_nmi(&mut self) {
        let pc = self.registers.pc();
        stack::push(&mut self.registers, &mut self.memory, (pc & 0xFF) as u8);
        stack::push(&mut self.registers, &mut self.memory, ((pc >> 8) & 0xFF) as u8);

        let address = self.memory.read16(NMI_VECTOR);
        self.registers.set_pc(address);
    }

    pub(crate) fn print_foo(&self) {
        println!("ppuaddr start");
        for x in &self.memory.foobar {
            println!("{:04X}", x);
        }
        println!("ppuaddr end");
    }

    pub(crate) fn decode(&mut self) -> OpCode {
        fn next_byte(cpu: &mut CPU) -> u8 {
            cpu.memory.read8(cpu.registers.increment_pc())
        }

        fn next_short(cpu: &mut CPU) -> u16 {
            let low = cpu.memory.read8(cpu.registers.increment_pc());
            let high = cpu.memory.read8(cpu.registers.increment_pc());

            low as u16 | ((high as u16) << 8)
        }

        let opcode = next_byte(self);

        let foo: (InstructionType, i32, AddressingMode) = match opcode {
            0x00 => {
                (InstructionType::BRK, 7, AddressingMode::Implicit)
            }
            0x01 => {
                (InstructionType::ORA, 6, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0x03 => {
                (InstructionType::SLO, 8, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0x04 => {
                (InstructionType::NOP, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x05 => {
                (InstructionType::ORA, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x06 => {
                (InstructionType::ASL, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x07 => {
                (InstructionType::SLO, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x08 => {
                (InstructionType::PHP, 3, AddressingMode::Implicit)
            }
            0x09 => {
                (InstructionType::ORA, 2, AddressingMode::Immediate(next_byte(self)))
            }
            0x0A => {
                (InstructionType::ASL, 2, AddressingMode::Accumulator)
            }
            0x0C => {
                (InstructionType::NOP, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x0D => {
                (InstructionType::ORA, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x0E => {
                (InstructionType::ASL, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x0F => {
                (InstructionType::SLO, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x10 => {
                (InstructionType::BPL, 2, AddressingMode::Relative(RelativeAddress::from(next_byte(self) as i8)))
            }
            0x11 => {
                (InstructionType::ORA, 5, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0x13 => {
                (InstructionType::SLO, 8, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0x14 => {
                (InstructionType::NOP, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x15 => {
                (InstructionType::ORA, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x16 => {
                (InstructionType::ASL, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x17 => {
                (InstructionType::SLO, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x18 => {
                (InstructionType::CLC, 2, AddressingMode::Implicit)
            }
            0x19 => {
                (InstructionType::ORA, 4, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0x1A => {
                (InstructionType::NOP, 2, AddressingMode::Implicit)
            }
            0x1B => {
                (InstructionType::SLO, 7, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0x1C => {
                (InstructionType::NOP, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x1D => {
                (InstructionType::ORA, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x1E => {
                (InstructionType::ASL, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x1F => {
                (InstructionType::SLO, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x20 => {
                (InstructionType::JSR, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x21 => {
                (InstructionType::AND, 6, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0x23 => {
                (InstructionType::RLA, 8, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0x24 => {
                (InstructionType::BIT, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x25 => {
                (InstructionType::AND, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x26 => {
                (InstructionType::ROL, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x27 => {
                (InstructionType::RLA, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x28 => {
                (InstructionType::PLP, 4, AddressingMode::Implicit)
            }
            0x29 => {
                (InstructionType::AND, 2, AddressingMode::Immediate(next_byte(self)))
            }
            0x2A => {
                (InstructionType::ROL, 2, AddressingMode::Accumulator)
            }
            0x2C => {
                (InstructionType::BIT, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x2D => {
                (InstructionType::AND, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x2E => {
                (InstructionType::ROL, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x2F => {
                (InstructionType::RLA, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x30 => {
                (InstructionType::BMI, 2, AddressingMode::Relative(RelativeAddress::from(next_byte(self) as i8)))
            }
            0x31 => {
                (InstructionType::AND, 5, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0x33 => {
                (InstructionType::RLA, 8, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0x34 => {
                (InstructionType::NOP, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x35 => {
                (InstructionType::AND, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x36 => {
                (InstructionType::ROL, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x37 => {
                (InstructionType::RLA, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x38 => {
                (InstructionType::SEC, 2, AddressingMode::Implicit)
            }
            0x39 => {
                (InstructionType::AND, 4, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0x3A => {
                (InstructionType::NOP, 2, AddressingMode::Implicit)
            }
            0x3B => {
                (InstructionType::RLA, 7, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0x3C => {
                (InstructionType::NOP, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x3D => {
                (InstructionType::AND, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x3E => {
                (InstructionType::ROL, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x3F => {
                (InstructionType::RLA, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x40 => {
                (InstructionType::RTI, 6, AddressingMode::Implicit)
            }
            0x41 => {
                (InstructionType::EOR, 6, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0x43 => {
                (InstructionType::SRE, 8, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0x44 => {
                (InstructionType::NOP, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x45 => {
                (InstructionType::EOR, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x46 => {
                (InstructionType::LSR, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x47 => {
                (InstructionType::SRE, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x48 => {
                (InstructionType::PHA, 3, AddressingMode::Implicit)
            }
            0x49 => {
                (InstructionType::EOR, 2, AddressingMode::Immediate(next_byte(self)))
            }
            0x4A => {
                (InstructionType::LSR, 2, AddressingMode::Accumulator)
            }
            0x4C => {
                (InstructionType::JMP, 3, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x4D => {
                (InstructionType::EOR, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x4E => {
                (InstructionType::LSR, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x4F => {
                (InstructionType::SRE, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x50 => {
                (InstructionType::BVC, 2, AddressingMode::Relative(RelativeAddress::from(next_byte(self) as i8)))
            }
            0x51 => {
                (InstructionType::EOR, 5, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0x53 => {
                (InstructionType::SRE, 8, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0x54 => {
                (InstructionType::NOP, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x55 => {
                (InstructionType::EOR, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x56 => {
                (InstructionType::LSR, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x57 => {
                (InstructionType::SRE, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x58 => {
                (InstructionType::CLI, 2, AddressingMode::Implicit)
            }
            0x59 => {
                (InstructionType::EOR, 4, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0x5A => {
                (InstructionType::NOP, 2, AddressingMode::Implicit)
            }
            0x5B => {
                (InstructionType::SRE, 7, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0x5C => {
                (InstructionType::NOP, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x5D => {
                (InstructionType::EOR, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x5E => {
                (InstructionType::LSR, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x5F => {
                (InstructionType::SRE, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x60 => {
                (InstructionType::RTS, 6, AddressingMode::Implicit)
            }
            0x61 => {
                (InstructionType::ADC, 6, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0x63 => {
                (InstructionType::RRA, 8, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0x64 => {
                (InstructionType::NOP, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x65 => {
                (InstructionType::ADC, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x66 => {
                (InstructionType::ROR, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x67 => {
                (InstructionType::RRA, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x68 => {
                (InstructionType::PLA, 4, AddressingMode::Implicit)
            }
            0x69 => {
                (InstructionType::ADC, 2, AddressingMode::Immediate(next_byte(self)))
            }
            0x6A => {
                (InstructionType::ROR, 2, AddressingMode::Accumulator)
            }
            0x6C => {
                (InstructionType::JMP, 5, AddressingMode::Indirect(AbsoluteAddress::from(next_short(self))))
            }
            0x6D => {
                (InstructionType::ADC, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x6E => {
                (InstructionType::ROR, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x6F => {
                (InstructionType::RRA, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x70 => {
                (InstructionType::BVS, 2, AddressingMode::Relative(RelativeAddress::from(next_byte(self) as i8)))
            }
            0x71 => {
                (InstructionType::ADC, 5, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0x73 => {
                (InstructionType::RRA, 8, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0x74 => {
                (InstructionType::NOP, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x75 => {
                (InstructionType::ADC, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x76 => {
                (InstructionType::ROR, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x77 => {
                (InstructionType::RRA, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x78 => {
                (InstructionType::SEI, 2, AddressingMode::Implicit)
            }
            0x79 => {
                (InstructionType::ADC, 4, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0x7A => {
                (InstructionType::NOP, 2, AddressingMode::Implicit)
            }
            0x7B => {
                (InstructionType::RRA, 7, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0x7C => {
                (InstructionType::NOP, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x7D => {
                (InstructionType::ADC, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x7E => {
                (InstructionType::ROR, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x7F => {
                (InstructionType::RRA, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x80 => {
                (InstructionType::NOP, 2, AddressingMode::Immediate(next_byte(self)))
            }
            0x81 => {
                (InstructionType::STA, 6, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0x83 => {
                (InstructionType::SAX, 6, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0x84 => {
                (InstructionType::STY, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x85 => {
                (InstructionType::STA, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x86 => {
                (InstructionType::STX, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x87 => {
                (InstructionType::SAX, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x88 => {
                (InstructionType::DEY, 2, AddressingMode::Implicit)
            }
            0x8A => {
                (InstructionType::TXA, 2, AddressingMode::Implicit)
            }
            0x8C => {
                (InstructionType::STY, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x8D => {
                (InstructionType::STA, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x8E => {
                (InstructionType::STX, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x8F => {
                (InstructionType::SAX, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x90 => {
                (InstructionType::BCC, 2, AddressingMode::Relative(RelativeAddress::from(next_byte(self) as i8)))
            }
            0x91 => {
                (InstructionType::STA, 6, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0x94 => {
                (InstructionType::STY, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x95 => {
                (InstructionType::STA, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x96 => {
                (InstructionType::STX, 4, AddressingMode::ZeroPageY(ZeroPageAddress::from(next_byte(self))))
            }
            0x97 => {
                (InstructionType::SAX, 4, AddressingMode::ZeroPageY(ZeroPageAddress::from(next_byte(self))))
            }
            0x98 => {
                (InstructionType::TYA, 2, AddressingMode::Implicit)
            }
            0x99 => {
                (InstructionType::STA, 5, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0x9A => {
                (InstructionType::TXS, 2, AddressingMode::Implicit)
            }
            0x9D => {
                (InstructionType::STA, 5, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0xA0 => {
                (InstructionType::LDY, 2, AddressingMode::Immediate(next_byte(self)))
            }
            0xA1 => {
                (InstructionType::LDA, 6, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0xA2 => {
                (InstructionType::LDX, 2, AddressingMode::Immediate(next_byte(self)))
            }
            0xA3 => {
                (InstructionType::LAX, 6, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0xA4 => {
                (InstructionType::LDY, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xA5 => {
                (InstructionType::LDA, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xA6 => {
                (InstructionType::LDX, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xA7 => {
                (InstructionType::LAX, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xA8 => {
                (InstructionType::TAY, 2, AddressingMode::Implicit)
            }
            0xA9 => {
                (InstructionType::LDA, 2, AddressingMode::Immediate(next_byte(self)))
            }
            0xAA => {
                (InstructionType::TAX, 2, AddressingMode::Implicit)
            }
            0xAC => {
                (InstructionType::LDY, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xAD => {
                (InstructionType::LDA, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xAE => {
                (InstructionType::LDX, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xAF => {
                (InstructionType::LAX, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xB0 => {
                (InstructionType::BCS, 2, AddressingMode::Relative(RelativeAddress::from(next_byte(self) as i8)))
            }
            0xB1 => {
                (InstructionType::LDA, 5, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0xB3 => {
                (InstructionType::LAX, 5, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0xB4 => {
                (InstructionType::LDY, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0xB5 => {
                (InstructionType::LDA, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0xB6 => {
                (InstructionType::LDX, 4, AddressingMode::ZeroPageY(ZeroPageAddress::from(next_byte(self))))
            }
            0xB7 => {
                (InstructionType::LAX, 4, AddressingMode::ZeroPageY(ZeroPageAddress::from(next_byte(self))))
            }
            0xB8 => {
                (InstructionType::CLV, 2, AddressingMode::Implicit)
            }
            0xB9 => {
                (InstructionType::LDA, 4, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0xBA => {
                (InstructionType::TSX, 2, AddressingMode::Implicit)
            }
            0xBC => {
                (InstructionType::LDY, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0xBD => {
                (InstructionType::LDA, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0xBE => {
                (InstructionType::LDX, 4, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0xBF => {
                (InstructionType::LAX, 4, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0xC0 => {
                (InstructionType::CPY, 2, AddressingMode::Immediate(next_byte(self)))
            }
            0xC1 => {
                (InstructionType::CMP, 6, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0xC3 => {
                (InstructionType::DCP, 8, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0xC4 => {
                (InstructionType::CPY, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xC5 => {
                (InstructionType::CMP, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xC6 => {
                (InstructionType::DEC, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xC7 => {
                (InstructionType::DCP, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xC8 => {
                (InstructionType::INY, 2, AddressingMode::Implicit)
            }
            0xC9 => {
                (InstructionType::CMP, 2, AddressingMode::Immediate(next_byte(self)))
            }
            0xCA => {
                (InstructionType::DEX, 2, AddressingMode::Implicit)
            }
            0xCC => {
                (InstructionType::CPY, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xCD => {
                (InstructionType::CMP, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xCE => {
                (InstructionType::DEC, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xCF => {
                (InstructionType::DCP, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xD0 => {
                (InstructionType::BNE, 2, AddressingMode::Relative(RelativeAddress::from(next_byte(self) as i8)))
            }
            0xD1 => {
                (InstructionType::CMP, 5, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0xD3 => {
                (InstructionType::DCP, 8, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0xD4 => {
                (InstructionType::NOP, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0xD5 => {
                (InstructionType::CMP, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0xD6 => {
                (InstructionType::DEC, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0xD7 => {
                (InstructionType::DCP, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0xD8 => {
                (InstructionType::CLD, 2, AddressingMode::Implicit)
            }
            0xD9 => {
                (InstructionType::CMP, 4, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0xDA => {
                (InstructionType::NOP, 2, AddressingMode::Implicit)
            }
            0xDB => {
                (InstructionType::DCP, 7, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0xDC => {
                (InstructionType::NOP, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0xDD => {
                (InstructionType::CMP, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0xDE => {
                (InstructionType::DEC, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0xDF => {
                (InstructionType::DCP, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0xE0 => {
                (InstructionType::CPX, 2, AddressingMode::Immediate(next_byte(self)))
            }
            0xE1 => {
                (InstructionType::SBC, 6, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0xE3 => {
                (InstructionType::ISC, 8, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0xE4 => {
                (InstructionType::CPX, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xE5 => {
                (InstructionType::SBC, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xE6 => {
                (InstructionType::INC, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xE7 => {
                (InstructionType::ISC, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xE8 => {
                (InstructionType::INX, 2, AddressingMode::Implicit)
            }
            0xE9 => {
                (InstructionType::SBC, 2, AddressingMode::Immediate(next_byte(self)))
            }
            0xEA => {
                (InstructionType::NOP, 2, AddressingMode::Implicit)
            }
            0xEB => {
                (InstructionType::SBC, 2, AddressingMode::Immediate(next_byte(self)))
            }
            0xEC => {
                (InstructionType::CPX, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xED => {
                (InstructionType::SBC, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xEE => {
                (InstructionType::INC, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xEF => {
                (InstructionType::ISC, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xF0 => {
                (InstructionType::BEQ, 2, AddressingMode::Relative(RelativeAddress::from(next_byte(self) as i8)))
            }
            0xF1 => {
                (InstructionType::SBC, 5, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0xF3 => {
                (InstructionType::ISC, 8, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0xF4 => {
                (InstructionType::NOP, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0xF5 => {
                (InstructionType::SBC, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0xF6 => {
                (InstructionType::INC, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0xF7 => {
                (InstructionType::ISC, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0xF8 => {
                (InstructionType::SED, 2, AddressingMode::Implicit)
            }
            0xF9 => {
                (InstructionType::SBC, 4, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0xFA => {
                (InstructionType::NOP, 2, AddressingMode::Implicit)
            }
            0xFB => {
                (InstructionType::ISC, 7, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0xFC => {
                (InstructionType::NOP, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0xFD => {
                (InstructionType::SBC, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0xFE => {
                (InstructionType::INC, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0xFF => {
                (InstructionType::ISC, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            _ => panic!("Unknown opcode {}", opcode)
        };

        OpCode::new(foo.0, foo.2, foo.1)
    }

    pub(crate) fn process(&mut self, op: OpCode) {
        let address = self.resolve_address(op.mode());

        let mut memory_value_source = MemoryValueSource {
            address: match address { Some(a) => a.address, None => AbsoluteAddress::from(0) },
            memory: self.memory
        };

        let mut accumulator_value_source = AccumulatorValueSource {
            registers: &mut self.registers
        };

        let g = match op.mode() {
            Accumulator => &mut accumulator_value_source as &mut dyn ValueSource,
            
          _ => panic!("aaaa")
        };

        /*let f = match source {
            ValueSourceResult::AccumulatorX(s) => s as &mut ValueSource,
            ValueSourceResult::MemoryX(s) => s as &mut ValueSource,
        };*/

        match op.instruction() {
            InstructionType::ADC => {
                let result = self.resolve_u8(op.mode());
                opcodes::adc(&mut self.registers, result.value)
            }
            InstructionType::AND => {
                let result = self.resolve_u8(op.mode());
                opcodes::andx(&mut self.registers, result.value)
            }
            InstructionType::ASL => {
                let result = self.resolve_u8(op.mode());
                // opcodes::aslx()
            }
            _ => panic!("asdsa")
        }
    }

    /*fn write(&mut self, mode: &AddressingMode, val: u8) {
        match mode {
            Accumulator => self.registers.set_accumulator(val),

        }
    }
*/
    fn resolve_address(&self, mode: &AddressingMode) -> Option<AddressingResult> {
        fn absolute_indexed(cpu: &CPU, address: &AbsoluteAddress, index: u8) -> AddressingResult {
            let mut page_boundary_crossed = false;
            let mut low = address.low();
            let mut high = address.high();

            low = low.wrapping_add(index);

            if low < index {
                page_boundary_crossed = true;
                high = high.wrapping_add(1);
            }

            AddressingResult {
                page_boundary_crossed,
                address: AbsoluteAddress::from_u8(low, high),
            }
        }


        match mode {
            Absolute(address) => Some(AddressingResult { address: *address, page_boundary_crossed: false }),
            Indirect(address) => {
                // from documentation at obelisk.me.uk:
                // NB:
                // An original 6502 has does not correctly fetch the target address if the
                // indirect vector falls on a page boundary (e.g. $xxFF where xx is any value
                // from $00 to $FF). In this case fetches the LSB from $xxFF as expected but
                // takes the MSB from $xx00. This is fixed in some later chips like the 65SC02
                // so for compatibility always ensure the indirect vector is not at the end of
                // the page.

                let address_high = if address.low() == 0xFF { AbsoluteAddress::from_u8(address.high(), 0x00) } else { address.offset(1) };

                let low = self.memory.read8x(address);
                let high = self.memory.read8x(&address_high);

                Some(AddressingResult {
                    address: AbsoluteAddress::from_u8(low, high),
                    page_boundary_crossed: false
                })
            },
            ZeroPage(address) => Some(AddressingResult { address: address.absolute(), page_boundary_crossed: false }),
            ZeroPageX(address) => Some(AddressingResult { address: address.offset(self.registers.x()).absolute(), page_boundary_crossed: false }),
            ZeroPageY(address) => Some(AddressingResult { address: address.offset(self.registers.y()).absolute(), page_boundary_crossed: false }),
            AbsoluteX(address) => Some(absolute_indexed(self, address, self.registers.x())),
            AbsoluteY(address) => Some(absolute_indexed(self, address, self.registers.y())),
            IndexedIndirect(address) => {
                let zero_page_address = address.offset(self.registers.x());  // TODO: Should I do wrapping_add here?

                let low = self.memory.read8x(&zero_page_address.absolute());
                let high = self.memory.read8x(&zero_page_address.offset(1).absolute());

                Some(AddressingResult {
                    address: AbsoluteAddress::from_u8(low, high),
                    page_boundary_crossed: false,
                })
            },
            IndirectIndexed(address) => {
                let mut low = self.memory.read8x(&address.absolute());
                let mut high = self.memory.read8x(&address.offset(1).absolute());// TODO: Wrapping_add here?

                let mut page_boundary_crossed = false;
                low = low.wrapping_add(self.registers.y());
                if low < self.registers.y() {
                    page_boundary_crossed = true;
                    high = high.wrapping_add(1);
                }

                Some(AddressingResult {
                    page_boundary_crossed,
                    address: AbsoluteAddress::from_u8(low, high),
                })
            },
            Implicit => None,
            Accumulator => None,
            Immediate(_) => None,
            Relative(_) => None,
        }
    }

    fn resolve_u8(&self, mode: &AddressingMode) -> ValueAddressingResult {
        match mode {
            AddressingMode::Relative(_) => panic!("Cannot take the value of a relative opcode. It is only used for branching."),
            AddressingMode::Indirect(_) => panic!("Cannot take the value of an indirect opcode. It is only used for JMP."),
            AddressingMode::Implicit => panic!("Cannot take the value of an implicit opcode"),
            AddressingMode::Accumulator => ValueAddressingResult { address: None,  value: self.registers.accumulator(), page_boundary_crossed: false },
            AddressingMode::Immediate(value) => ValueAddressingResult { address: None, value: *value, page_boundary_crossed: false },
            _ => {
                let result = self.resolve_address(mode).unwrap();
                ValueAddressingResult {
                    address: Some(result.address),
                    value: self.memory.read8x(&result.address),
                    page_boundary_crossed: result.page_boundary_crossed,
                }
            }
        }
    }

    pub(crate) fn process_instruction(&mut self) -> i32 {
        if self.registers.pc() == 0xF0E9 {
            let foo = 2;
        }
        let opcode = self.memory.read8(self.registers.increment_pc());
        print!("{:02X} ", opcode);

        match opcode {
            0x00 => opcodes::brk_implied(&mut self.registers, &mut self.memory),
            0x01 => opcodes::ora_indirect_x(&mut self.registers, &self.memory),
            0x03 => opcodes::slo_indirect_x(&mut self.registers, &mut self.memory),
            0x04 => opcodes::nop_zero_page(&mut self.registers, &self.memory),
            0x05 => opcodes::ora_zero_page(&mut self.registers, &self.memory),
            0x06 => opcodes::asl_zero_page(&mut self.registers, &mut self.memory),
            0x07 => opcodes::slo_zero_page(&mut self.registers, &mut self.memory),
            0x08 => opcodes::php_implied(&mut self.registers, &mut self.memory),
            0x09 => opcodes::ora_immediate(&mut self.registers, &self.memory),
            0x0A => opcodes::asl_accumulator(&mut self.registers),
            0x0C => opcodes::nop_absolute(&mut self.registers, &self.memory),
            0x0D => opcodes::ora_absolute(&mut self.registers, &self.memory),
            0x0E => opcodes::asl_absolute(&mut self.registers, &mut self.memory),
            0x0F => opcodes::slo_absolute(&mut self.registers, &mut self.memory),
            0x10 => opcodes::bpl_relative(&mut self.registers, &self.memory),
            0x11 => opcodes::ora_indirect_y(&mut self.registers, &self.memory),
            0x13 => opcodes::slo_indirect_y(&mut self.registers, &mut self.memory),
            0x14 => opcodes::nop_zero_page_x(&mut self.registers, &self.memory),
            0x15 => opcodes::ora_zero_page_x(&mut self.registers, &self.memory),
            0x16 => opcodes::asl_zero_page_x(&mut self.registers, &mut self.memory),
            0x17 => opcodes::slo_zero_page_x(&mut self.registers, &mut self.memory),
            0x18 => opcodes::clc_implied(&mut self.registers),
            0x19 => opcodes::ora_absolute_y(&mut self.registers, &self.memory),
            0x1A => opcodes::nop_implied(),
            0x1B => opcodes::slo_absolute_y(&mut self.registers, &mut self.memory),
            0x1C => opcodes::nop_absolute_x(&mut self.registers, &self.memory),
            0x1D => opcodes::ora_absolute_x(&mut self.registers, &self.memory),
            0x1E => opcodes::asl_absolute_x(&mut self.registers, &mut self.memory),
            0x1F => opcodes::slo_absolute_x(&mut self.registers, &mut self.memory),
            0x20 => opcodes::jsr_absolute(&mut self.registers, &mut self.memory),
            0x21 => opcodes::and_indirect_x(&mut self.registers, &self.memory),
            0x23 => opcodes::rla_indirect_x(&mut self.registers, &mut self.memory),
            0x24 => opcodes::bit_zero_page(&mut self.registers, &self.memory),
            0x25 => opcodes::and_zero_page(&mut self.registers, &self.memory),
            0x26 => opcodes::rol_zero_page(&mut self.registers, &mut self.memory),
            0x27 => opcodes::rla_zero_page(&mut self.registers, &mut self.memory),
            0x28 => opcodes::plp_implied(&mut self.registers, &mut self.memory),
            0x29 => opcodes::and_immediate(&mut self.registers, &self.memory),
            0x2A => opcodes::rol_accumulator(&mut self.registers),
            0x2C => opcodes::bit_absolute(&mut self.registers, &self.memory),
            0x2D => opcodes::and_absolute(&mut self.registers, &self.memory),
            0x2E => opcodes::rol_absolute(&mut self.registers, &mut self.memory),
            0x2F => opcodes::rla_absolute(&mut self.registers, &mut self.memory),
            0x30 => opcodes::bmi_relative(&mut self.registers, &self.memory),
            0x31 => opcodes::and_indirect_y(&mut self.registers, &self.memory),
            0x33 => opcodes::rla_indirect_y(&mut self.registers, &mut self.memory),
            0x34 => opcodes::nop_zero_page_x(&mut self.registers, &self.memory),
            0x35 => opcodes::and_zero_page_x(&mut self.registers, &self.memory),
            0x36 => opcodes::rol_zero_page_x(&mut self.registers, &mut self.memory),
            0x37 => opcodes::rla_zero_page_x(&mut self.registers, &mut self.memory),
            0x38 => opcodes::sec_implied(&mut self.registers),
            0x39 => opcodes::and_absolute_y(&mut self.registers, &self.memory),
            0x3A => opcodes::nop_implied(),
            0x3B => opcodes::rla_absolute_y(&mut self.registers, &mut self.memory),
            0x3C => opcodes::nop_absolute_x(&mut self.registers, &self.memory),
            0x3D => opcodes::and_absolute_x(&mut self.registers, &self.memory),
            0x3E => opcodes::rol_absolute_x(&mut self.registers, &mut self.memory),
            0x3F => opcodes::rla_absolute_x(&mut self.registers, &mut self.memory),
            0x40 => opcodes::rti_implied(&mut self.registers, &self.memory),
            0x41 => opcodes::eor_indirect_x(&mut self.registers, &self.memory),
            0x43 => opcodes::sre_indirect_x(&mut self.registers, &mut self.memory),
            0x44 => opcodes::nop_zero_page(&mut self.registers, &self.memory),
            0x45 => opcodes::eor_zero_page(&mut self.registers, &self.memory),
            0x46 => opcodes::lsr_zero_page(&mut self.registers, &mut self.memory),
            0x47 => opcodes::sre_zero_page(&mut self.registers, &mut self.memory),
            0x48 => opcodes::pha_implied(&mut self.registers, &mut self.memory),
            0x49 => opcodes::eor_immediate(&mut self.registers, &self.memory),
            0x4A => opcodes::lsr_accumulator(&mut self.registers),
            0x4C => opcodes::jmp_absolute(&mut self.registers, &self.memory),
            0x4D => opcodes::eor_absolute(&mut self.registers, &self.memory),
            0x4E => opcodes::lsr_absolute(&mut self.registers, &mut self.memory),
            0x4F => opcodes::sre_absolute(&mut self.registers, &mut self.memory),
            0x50 => opcodes::bvc_relative(&mut self.registers, &self.memory),
            0x51 => opcodes::eor_indirect_y(&mut self.registers, &self.memory),
            0x53 => opcodes::sre_indirect_y(&mut self.registers, &mut self.memory),
            0x54 => opcodes::nop_zero_page_x(&mut self.registers, &self.memory),
            0x55 => opcodes::eor_zero_page_x(&mut self.registers, &self.memory),
            0x56 => opcodes::lsr_zero_page_x(&mut self.registers, &mut self.memory),
            0x57 => opcodes::sre_zero_page_x(&mut self.registers, &mut self.memory),
            0x58 => opcodes::cli_implied(&mut self.registers),
            0x59 => opcodes::eor_absolute_y(&mut self.registers, &self.memory),
            0x5A => opcodes::nop_implied(),
            0x5B => opcodes::sre_absolute_y(&mut self.registers, &mut self.memory),
            0x5C => opcodes::nop_absolute_x(&mut self.registers, &self.memory),
            0x5D => opcodes::eor_absolute_x(&mut self.registers, &self.memory),
            0x5E => opcodes::lsr_absolute_x(&mut self.registers, &mut self.memory),
            0x5F => opcodes::sre_absolute_x(&mut self.registers, &mut self.memory),
            0x60 => opcodes::rts_implied(&mut self.registers, &self.memory),
            0x61 => opcodes::adc_indirect_x(&mut self.registers, &self.memory),
            0x63 => opcodes::rra_indirect_x(&mut self.registers, &mut self.memory),
            0x64 => opcodes::nop_zero_page(&mut self.registers, &self.memory),
            0x65 => opcodes::adc_zero_page(&mut self.registers, &self.memory),
            0x66 => opcodes::ror_zero_page(&mut self.registers, &mut self.memory),
            0x67 => opcodes::rra_zero_page(&mut self.registers, &mut self.memory),
            0x68 => opcodes::pla_implied(&mut self.registers, &self.memory),
            0x69 => opcodes::adc_immediate(&mut self.registers, &self.memory),
            0x6A => opcodes::ror_accumulator(&mut self.registers),
            0x6C => opcodes::jmp_indirect(&mut self.registers, &self.memory),
            0x6D => opcodes::adc_absolute(&mut self.registers, &self.memory),
            0x6E => opcodes::ror_absolute(&mut self.registers, &mut self.memory),
            0x6F => opcodes::rra_absolute(&mut self.registers, &mut self.memory),
            0x70 => opcodes::bvs_relative(&mut self.registers, &self.memory),
            0x71 => opcodes::adc_indirect_y(&mut self.registers, &self.memory),
            0x73 => opcodes::rra_indirect_y(&mut self.registers, &mut self.memory),
            0x74 => opcodes::nop_zero_page_x(&mut self.registers, &self.memory),
            0x75 => opcodes::adc_zero_page_x(&mut self.registers, &self.memory),
            0x76 => opcodes::ror_zero_page_x(&mut self.registers, &mut self.memory),
            0x77 => opcodes::rra_zero_page_x(&mut self.registers, &mut self.memory),
            0x78 => opcodes::sei_implied(&mut self.registers),
            0x79 => opcodes::adc_absolute_y(&mut self.registers, &self.memory),
            0x7A => opcodes::nop_implied(),
            0x7B => opcodes::rra_absolute_y(&mut self.registers, &mut self.memory),
            0x7C => opcodes::nop_absolute_x(&mut self.registers, &self.memory),
            0x7D => opcodes::adc_absolute_x(&mut self.registers, &self.memory),
            0x7E => opcodes::ror_absolute_x(&mut self.registers, &mut self.memory),
            0x7F => opcodes::rra_absolute_x(&mut self.registers, &mut self.memory),
            0x80 => opcodes::nop_immediate(&mut self.registers, &self.memory),
            0x81 => opcodes::sta_indirect_x(&mut self.registers, &mut self.memory),
            0x83 => opcodes::sax_indirect_x(&mut self.registers, &mut self.memory),
            0x84 => opcodes::sty_zero_page(&mut self.registers, &mut self.memory),
            0x85 => opcodes::sta_zero_page(&mut self.registers, &mut self.memory),
            0x86 => opcodes::stx_zero_page(&mut self.registers, &mut self.memory),
            0x87 => opcodes::sax_zero_page(&mut self.registers, &mut self.memory),
            0x88 => opcodes::dey_implied(&mut self.registers),
            0x8A => opcodes::txa_implied(&mut self.registers),
            0x8C => opcodes::sty_absolute(&mut self.registers, &mut self.memory),
            0x8D => opcodes::sta_absolute(&mut self.registers, &mut self.memory),
            0x8E => opcodes::stx_absolute(&mut self.registers, &mut self.memory),
            0x8F => opcodes::sax_absolute(&mut self.registers, &mut self.memory),
            0x90 => opcodes::bcc_relative(&mut self.registers, &mut self.memory),
            0x91 => opcodes::sta_indirect_y(&mut self.registers, &mut self.memory),
            0x94 => opcodes::sty_zero_page_x(&mut self.registers, &mut self.memory),
            0x95 => opcodes::sta_zero_page_x(&mut self.registers, &mut self.memory),
            0x96 => opcodes::stx_zero_page_y(&mut self.registers, &mut self.memory),
            0x97 => opcodes::sax_zero_page_y(&mut self.registers, &mut self.memory),
            0x98 => opcodes::tya_implied(&mut self.registers),
            0x99 => opcodes::sta_absolute_y(&mut self.registers, &mut self.memory),
            0x9A => opcodes::txs_implied(&mut self.registers),
            0x9D => opcodes::sta_absolute_x(&mut self.registers, &mut self.memory),
            0xA0 => opcodes::ldy_immediate(&mut self.registers, &self.memory),
            0xA1 => opcodes::lda_indirect_x(&mut self.registers, &self.memory),
            0xA2 => opcodes::ldx_immediate(&mut self.registers, &self.memory),
            0xA3 => opcodes::lax_indirect_x(&mut self.registers, &self.memory),
            0xA4 => opcodes::ldy_zero_page(&mut self.registers, &self.memory),
            0xA5 => opcodes::lda_zero_page(&mut self.registers, &self.memory),
            0xA6 => opcodes::ldx_zero_page(&mut self.registers, &self.memory),
            0xA7 => opcodes::lax_zero_page(&mut self.registers, &self.memory),
            0xA8 => opcodes::tay_implied(&mut self.registers),
            0xA9 => opcodes::lda_immediate(&mut self.registers, &self.memory),
            0xAA => opcodes::tax_implied(&mut self.registers),
            0xAC => opcodes::ldy_absolute(&mut self.registers, &self.memory),
            0xAD => opcodes::lda_absolute(&mut self.registers, &self.memory),
            0xAE => opcodes::ldx_absolute(&mut self.registers, &self.memory),
            0xAF => opcodes::lax_absolute(&mut self.registers, &self.memory),
            0xBA => opcodes::tsx_implied(&mut self.registers),
            0xB0 => opcodes::bcs_relative(&mut self.registers, &self.memory),
            0xB1 => opcodes::lda_indirect_y(&mut self.registers, &self.memory),
            0xB3 => opcodes::lax_indirect_y(&mut self.registers, &self.memory),
            0xB4 => opcodes::ldy_zero_page_x(&mut self.registers, &self.memory),
            0xB5 => opcodes::lda_zero_page_x(&mut self.registers, &self.memory),
            0xB6 => opcodes::ldx_zero_page_y(&mut self.registers, &self.memory),
            0xB7 => opcodes::lax_zero_page_y(&mut self.registers, &self.memory),
            0xB8 => opcodes::clv_implied(&mut self.registers),
            0xB9 => opcodes::lda_absolute_y(&mut self.registers, &self.memory),
            0xBC => opcodes::ldy_absolute_x(&mut self.registers, &self.memory),
            0xBD => opcodes::lda_absolute_x(&mut self.registers, &self.memory),
            0xBE => opcodes::ldx_absolute_y(&mut self.registers, &self.memory),
            0xBF => opcodes::lax_absolute_y(&mut self.registers, &self.memory),
            0xC0 => opcodes::cpy_immediate(&mut self.registers, &self.memory),
            0xC1 => opcodes::cmp_indirect_x(&mut self.registers, &self.memory),
            0xC3 => opcodes::dcp_indirect_x(&mut self.registers, &mut self.memory),
            0xC4 => opcodes::cpy_zero_page(&mut self.registers, &self.memory),
            0xC5 => opcodes::cmp_zero_page(&mut self.registers, &self.memory),
            0xC6 => opcodes::dec_zero_page(&mut self.registers, &mut self.memory),
            0xC7 => opcodes::dcp_zero_page(&mut self.registers, &mut self.memory),
            0xC8 => opcodes::iny_implied(&mut self.registers),
            0xC9 => opcodes::cmp_immediate(&mut self.registers, &self.memory),
            0xCA => opcodes::dex_implied(&mut self.registers),
            0xCC => opcodes::cpy_absolute(&mut self.registers, &self.memory),
            0xCD => opcodes::cmp_absolute(&mut self.registers, &self.memory),
            0xCE => opcodes::dec_absolute(&mut self.registers, &mut self.memory),
            0xCF => opcodes::dcp_absolute(&mut self.registers, &mut self.memory),
            0xD0 => opcodes::bne_relative(&mut self.registers, &self.memory),
            0xD1 => opcodes::cmp_indirect_y(&mut self.registers, &self.memory),
            0xD3 => opcodes::dcp_indirect_y(&mut self.registers, &mut self.memory),
            0xD4 => opcodes::nop_zero_page_x(&mut self.registers, &self.memory),
            0xD5 => opcodes::cmp_zero_page_x(&mut self.registers, &self.memory),
            0xD6 => opcodes::dec_zero_page_x(&mut self.registers, &mut self.memory),
            0xD7 => opcodes::dcp_zero_page_x(&mut self.registers, &mut self.memory),
            0xD8 => opcodes::cld_implied(&mut self.registers),
            0xD9 => opcodes::cmp_absolute_y(&mut self.registers, &self.memory),
            0xDA => opcodes::nop_implied(),
            0xDB => opcodes::dcp_absolute_y(&mut self.registers, &mut self.memory),
            0xDC => opcodes::nop_absolute_x(&mut self.registers, &self.memory),
            0xDD => opcodes::cmp_absolute_x(&mut self.registers, &self.memory),
            0xDE => opcodes::dec_absolute_x(&mut self.registers, &mut self.memory),
            0xDF => opcodes::dcp_absolute_x(&mut self.registers, &mut self.memory),
            0xE0 => opcodes::cpx_immediate(&mut self.registers, &self.memory),
            0xE1 => opcodes::sbc_indirect_x(&mut self.registers, &self.memory),
            0xE3 => opcodes::isc_indirect_x(&mut self.registers, &mut self.memory),
            0xE4 => opcodes::cpx_zero_page(&mut self.registers, &self.memory),
            0xE5 => opcodes::sbc_zero_page(&mut self.registers, &self.memory),
            0xE6 => opcodes::inc_zero_page(&mut self.registers, &mut self.memory),
            0xE7 => opcodes::isc_zero_page(&mut self.registers, &mut self.memory),
            0xE8 => opcodes::inx_implied(&mut self.registers),
            0xE9 => opcodes::sbc_immediate(&mut self.registers, &self.memory),
            0xEA => opcodes::nop_implied(),
            0xEB => opcodes::sbc_immediate(&mut self.registers, &self.memory),
            0xEC => opcodes::cpx_absolute(&mut self.registers, &self.memory),
            0xED => opcodes::sbc_absolute(&mut self.registers, &self.memory),
            0xEE => opcodes::inc_absolute(&mut self.registers, &mut self.memory),
            0xEF => opcodes::isc_absolute(&mut self.registers, &mut self.memory),
            0xF0 => opcodes::beq_relative(&mut self.registers, &self.memory),
            0xF1 => opcodes::sbc_indirect_y(&mut self.registers, &self.memory),
            0xF3 => opcodes::isc_indirect_y(&mut self.registers, &mut self.memory),
            0xF4 => opcodes::nop_zero_page_x(&mut self.registers, &self.memory),
            0xF5 => opcodes::sbc_zero_page_x(&mut self.registers, &self.memory),
            0xF6 => opcodes::inc_zero_page_x(&mut self.registers, &mut self.memory),
            0xF7 => opcodes::isc_zero_page_x(&mut self.registers, &mut self.memory),
            0xF8 => opcodes::sed_implied(&mut self.registers),
            0xF9 => opcodes::sbc_absolute_y(&mut self.registers, &self.memory),
            0xFA => opcodes::nop_implied(),
            0xFB => opcodes::isc_absolute_y(&mut self.registers, &mut self.memory),
            0xFC => opcodes::nop_absolute_x(&mut self.registers, &self.memory),
            0xFD => opcodes::sbc_absolute_x(&mut self.registers, &self.memory),
            0xFE => opcodes::inc_absolute_x(&mut self.registers, &mut self.memory),
            0xFF => opcodes::isc_absolute_x(&mut self.registers, &mut self.memory),
            _ => panic!("Unknown opcode {}", opcode)
        }
    }
}