use crate::cpuregisters::CPURegisters;
use crate::ram_controller::RamController;
use crate::{opcodes, stack};
use crate::opcodes::{OpCode, InstructionType, AddressingMode, ZeroPageAddress, AbsoluteAddress, RelativeAddress, ValueAddressingResult, AddressingResult};
use crate::opcodes::AddressingMode::{ZeroPage, Relative, Absolute, Immediate, Indirect, ZeroPageX, ZeroPageY, Implicit, Accumulator, AbsoluteX, AbsoluteY, IndexedIndirect, IndirectIndexed};
use sdl2::render::BlendMode::Add;

const NMI_VECTOR: u16 = 0xFFFA;
const RESET_VECTOR: u16 = 0xFFFC;

const UNOFFICIAL_OPCODES: [i32; 80] = [
    0xC7, 0xD7, 0xCF, 0xDF, 0xDB, 0xC3, 0xD3, // DCP
    0x04, 0x14, 0x34, 0x44, 0x54, 0x64, 0x74, 0x80, 0x82, 0x89, 0xC2, 0xD4, 0xE2, 0xF4, // NOP
    0x1A, 0x3A, 0x5A, 0x7A, 0xDA, 0xFA, // NOP
    0x0C, 0x1C, 0x3C, 0x5C, 0x7C, 0xDC, 0xFC,
    0xE7, 0xF7, 0xEF, 0xFF, 0xFB, 0xE3, 0xF3, // ISC
    0xA7, 0xB7, 0xAF, 0xBF, 0xA3, 0xB3, // LAX
    0x27, 0x37, 0x2F, 0x3F, 0x3B, 0x23, 0x33, // RLA
    0x67, 0x77, 0x6F, 0x7F, 0x7B, 0x63, 0x73, // RRA
    0xEB, // SBC
    0x07, 0x17, 0x0F, 0x1F, 0x1B, 0x03, 0x13, // SLO
    0x47, 0x57, 0x4F, 0x5F, 0x5B, 0x43, 0x53, // SRE
    0x87, 0x97, 0x83, 0x8F, // SAX
];

pub(crate) type OpCodeFn = fn(&mut CPURegisters, &mut RamController, &AddressingMode) -> i32;

pub struct CPU<'ctl, 'data: 'ctl> {
    pub registers: CPURegisters,
    memory: &'ctl mut RamController<'data>,
}

impl<'ctl, 'data: 'ctl> CPU<'ctl, 'data> {
    pub(crate) fn new(mem: &'ctl mut RamController<'data>) -> CPU<'ctl, 'data> {
        CPU {
            memory: mem,
            registers: CPURegisters::new(),
        }
    }

    pub(crate) fn reset(&mut self) {
        self.registers = CPURegisters::new();

        let address = self.memory.read16(RESET_VECTOR);
        self.registers.set_pc(AbsoluteAddress::from(address));

        // Only used with nestest
        // self.registers.set_pc(AbsoluteAddress::from(0xC000));
    }

    pub(crate) fn trigger_nmi(&mut self) {
        let pc = self.registers.pc();
        stack::push(&mut self.registers, &mut self.memory, pc.low());
        stack::push(&mut self.registers, &mut self.memory, pc.high());

        let address = self.memory.read16(NMI_VECTOR);
        self.registers.set_pc(AbsoluteAddress::from(address));
    }

    pub(crate) fn print_foo(&self) {
        println!("ppuaddr start");
        for x in &self.memory.foobar {
            println!("{:04X}", x);
        }
        println!("ppuaddr end");
    }

    /*pub(crate) fn decode2(&mut self) -> OpCode {
        fn next_byte(cpu: &mut CPU) -> u8 {
            cpu.memory.read8x(&cpu.registers.increment_pc())
        }

        fn next_short(cpu: &mut CPU) -> u16 {
            let low = cpu.memory.read8x(&cpu.registers.increment_pc());
            let high = cpu.memory.read8x(&cpu.registers.increment_pc());

            low as u16 | ((high as u16) << 8)
        }

        let code = next_byte(self);

        let opcode_index = code & 0b00000011;
        let instruction_index = ((code & 0b11100000) >> 5) + (opcode_index * 8);
        let addressing_mode_index = ((code & 0b00011100) >> 2) + (opcode_index * 8);


        let instr_table = [
            InstructionType::ORA,
            InstructionType::AND,
            InstructionType::EOR,
            InstructionType::ADC,
            InstructionType::STA,
            InstructionType::LDA,
            InstructionType::CMP,
            InstructionType::SBC,

            InstructionType::ASL,
            InstructionType::ROL,
            InstructionType::LSR,
            InstructionType::ROR,
            InstructionType::STX,
            InstructionType::LDX,
            InstructionType::DEC,
            InstructionType::INC
        ];

        let instruction_type = instr_table[instruction_index];
        let addressing_mode = match addressing_mode_index {
            0 => AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))),
            1 => AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))),
            2 => AddressingMode::Immediate(AbsoluteAddress::from(next_short(self))),
            3 => AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))),
            4 => AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))),
            5 => AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))),
            6 => AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))),
            7 => AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))),

            8 => AddressingMode::Immediate(AbsoluteAddress::from(next_short(self))),
            9 => AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))),
            10 => AddressingMode::Accumulator,
            11 => AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))),
            12 => AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))),
            13 => AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))),
            _ => panic!("Unknown addressing mode index!")
        };
    }*/

    pub(crate) fn decode(&mut self) -> OpCode {
        fn next_byte(cpu: &mut CPU) -> u8 {
            cpu.memory.read8x(&cpu.registers.increment_pc())
        }

        fn next_short(cpu: &mut CPU) -> u16 {
            let low = cpu.memory.read8x(&cpu.registers.increment_pc());
            let high = cpu.memory.read8x(&cpu.registers.increment_pc());

            low as u16 | ((high as u16) << 8)
        }

        let opcode = next_byte(self);

        let foo: (InstructionType, OpCodeFn, i32, AddressingMode) = match opcode {
            0x00 => {
                (InstructionType::BRK, opcodes::brk, 7, AddressingMode::Implicit)
            }
            0x01 => {
                (InstructionType::ORA, opcodes::ora, 6, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0x03 => {
                (InstructionType::SLO, opcodes::slo, 8, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0x04 => {
                (InstructionType::NOP, opcodes::nop, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x05 => {
                (InstructionType::ORA, opcodes::ora, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x06 => {
                (InstructionType::ASL, opcodes::asl, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x07 => {
                (InstructionType::SLO, opcodes::slo, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x08 => {
                (InstructionType::PHP, opcodes::php, 3, AddressingMode::Implicit)
            }
            0x09 => {
                (InstructionType::ORA, opcodes::ora, 2, AddressingMode::Immediate(self.registers.increment_pc()))
            }
            0x0A => {
                (InstructionType::ASL, opcodes::asl, 2, AddressingMode::Accumulator)
            }
            0x0C => {
                (InstructionType::NOP, opcodes::nop, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x0D => {
                (InstructionType::ORA, opcodes::ora, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x0E => {
                (InstructionType::ASL, opcodes::asl, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x0F => {
                (InstructionType::SLO, opcodes::slo, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x10 => {
                (InstructionType::BPL, opcodes::bpl, 2, AddressingMode::Relative(RelativeAddress::from(next_byte(self) as i8)))
            }
            0x11 => {
                (InstructionType::ORA, opcodes::ora, 5, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0x13 => {
                (InstructionType::SLO, opcodes::slo, 8, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0x14 => {
                (InstructionType::NOP, opcodes::nop, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x15 => {
                (InstructionType::ORA, opcodes::ora, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x16 => {
                (InstructionType::ASL, opcodes::asl, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x17 => {
                (InstructionType::SLO, opcodes::slo, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x18 => {
                (InstructionType::CLC, opcodes::clc, 2, AddressingMode::Implicit)
            }
            0x19 => {
                (InstructionType::ORA, opcodes::ora, 4, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0x1A => {
                (InstructionType::NOP, opcodes::nop, 2, AddressingMode::Implicit)
            }
            0x1B => {
                (InstructionType::SLO, opcodes::slo, 7, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0x1C => {
                (InstructionType::NOP, opcodes::nop, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x1D => {
                (InstructionType::ORA, opcodes::ora, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x1E => {
                (InstructionType::ASL, opcodes::asl, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x1F => {
                (InstructionType::SLO, opcodes::slo, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x20 => {
                (InstructionType::JSR, opcodes::jsr, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x21 => {
                (InstructionType::AND, opcodes::and, 6, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0x23 => {
                (InstructionType::RLA, opcodes::rla, 8, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0x24 => {
                (InstructionType::BIT, opcodes::bit, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x25 => {
                (InstructionType::AND, opcodes::and, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x26 => {
                (InstructionType::ROL, opcodes::rol, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x27 => {
                (InstructionType::RLA, opcodes::rla, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x28 => {
                (InstructionType::PLP, opcodes::plp, 4, AddressingMode::Implicit)
            }
            0x29 => {
                (InstructionType::AND, opcodes::and, 2, AddressingMode::Immediate(self.registers.increment_pc()))
            }
            0x2A => {
                (InstructionType::ROL, opcodes::rol, 2, AddressingMode::Accumulator)
            }
            0x2C => {
                (InstructionType::BIT, opcodes::bit, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x2D => {
                (InstructionType::AND, opcodes::and, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x2E => {
                (InstructionType::ROL, opcodes::rol, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x2F => {
                (InstructionType::RLA, opcodes::rla, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x30 => {
                (InstructionType::BMI, opcodes::bmi, 2, AddressingMode::Relative(RelativeAddress::from(next_byte(self) as i8)))
            }
            0x31 => {
                (InstructionType::AND, opcodes::and, 5, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0x33 => {
                (InstructionType::RLA, opcodes::rla, 8, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0x34 => {
                (InstructionType::NOP, opcodes::nop, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x35 => {
                (InstructionType::AND, opcodes::and, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x36 => {
                (InstructionType::ROL, opcodes::rol, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x37 => {
                (InstructionType::RLA, opcodes::rla, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x38 => {
                (InstructionType::SEC, opcodes::sec, 2, AddressingMode::Implicit)
            }
            0x39 => {
                (InstructionType::AND, opcodes::and, 4, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0x3A => {
                (InstructionType::NOP, opcodes::nop, 2, AddressingMode::Implicit)
            }
            0x3B => {
                (InstructionType::RLA, opcodes::rla, 7, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0x3C => {
                (InstructionType::NOP, opcodes::nop, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x3D => {
                (InstructionType::AND, opcodes::and, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x3E => {
                (InstructionType::ROL, opcodes::rol, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x3F => {
                (InstructionType::RLA, opcodes::rla, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x40 => {
                (InstructionType::RTI, opcodes::rti, 6, AddressingMode::Implicit)
            }
            0x41 => {
                (InstructionType::EOR, opcodes::eor, 6, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0x43 => {
                (InstructionType::SRE, opcodes::sre, 8, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0x44 => {
                (InstructionType::NOP, opcodes::nop, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x45 => {
                (InstructionType::EOR, opcodes::eor, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x46 => {
                (InstructionType::LSR, opcodes::lsr, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x47 => {
                (InstructionType::SRE, opcodes::sre, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x48 => {
                (InstructionType::PHA, opcodes::pha, 3, AddressingMode::Implicit)
            }
            0x49 => {
                (InstructionType::EOR, opcodes::eor, 2, AddressingMode::Immediate(self.registers.increment_pc()))
            }
            0x4A => {
                (InstructionType::LSR, opcodes::lsr, 2, AddressingMode::Accumulator)
            }
            0x4C => {
                (InstructionType::JMP, opcodes::jmp, 3, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x4D => {
                (InstructionType::EOR, opcodes::eor, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x4E => {
                (InstructionType::LSR, opcodes::lsr, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x4F => {
                (InstructionType::SRE, opcodes::sre, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x50 => {
                (InstructionType::BVC, opcodes::bvc, 2, AddressingMode::Relative(RelativeAddress::from(next_byte(self) as i8)))
            }
            0x51 => {
                (InstructionType::EOR, opcodes::eor, 5, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0x53 => {
                (InstructionType::SRE, opcodes::sre, 8, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0x54 => {
                (InstructionType::NOP, opcodes::nop, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x55 => {
                (InstructionType::EOR, opcodes::eor, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x56 => {
                (InstructionType::LSR, opcodes::lsr, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x57 => {
                (InstructionType::SRE, opcodes::sre, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x58 => {
                (InstructionType::CLI, opcodes::cli, 2, AddressingMode::Implicit)
            }
            0x59 => {
                (InstructionType::EOR, opcodes::eor, 4, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0x5A => {
                (InstructionType::NOP, opcodes::nop, 2, AddressingMode::Implicit)
            }
            0x5B => {
                (InstructionType::SRE, opcodes::sre, 7, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0x5C => {
                (InstructionType::NOP, opcodes::nop, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x5D => {
                (InstructionType::EOR, opcodes::eor, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x5E => {
                (InstructionType::LSR, opcodes::lsr, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x5F => {
                (InstructionType::SRE, opcodes::sre, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x60 => {
                (InstructionType::RTS, opcodes::rts, 6, AddressingMode::Implicit)
            }
            0x61 => {
                (InstructionType::ADC, opcodes::adc, 6, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0x63 => {
                (InstructionType::RRA, opcodes::rra, 8, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0x64 => {
                (InstructionType::NOP, opcodes::nop, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x65 => {
                (InstructionType::ADC, opcodes::adc, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x66 => {
                (InstructionType::ROR, opcodes::ror, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x67 => {
                (InstructionType::RRA, opcodes::rra, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x68 => {
                (InstructionType::PLA, opcodes::pla, 4, AddressingMode::Implicit)
            }
            0x69 => {
                (InstructionType::ADC, opcodes::adc, 2, AddressingMode::Immediate(self.registers.increment_pc()))
            }
            0x6A => {
                (InstructionType::ROR, opcodes::ror, 2, AddressingMode::Accumulator)
            }
            0x6C => {
                (InstructionType::JMP, opcodes::jmp, 5, AddressingMode::Indirect(AbsoluteAddress::from(next_short(self))))
            }
            0x6D => {
                (InstructionType::ADC, opcodes::adc, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x6E => {
                (InstructionType::ROR, opcodes::ror, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x6F => {
                (InstructionType::RRA, opcodes::rra, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x70 => {
                (InstructionType::BVS, opcodes::bvs, 2, AddressingMode::Relative(RelativeAddress::from(next_byte(self) as i8)))
            }
            0x71 => {
                (InstructionType::ADC, opcodes::adc, 5, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0x73 => {
                (InstructionType::RRA, opcodes::rra, 8, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0x74 => {
                (InstructionType::NOP, opcodes::nop, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x75 => {
                (InstructionType::ADC, opcodes::adc, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x76 => {
                (InstructionType::ROR, opcodes::ror, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x77 => {
                (InstructionType::RRA, opcodes::rra, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x78 => {
                (InstructionType::SEI, opcodes::sei, 2, AddressingMode::Implicit)
            }
            0x79 => {
                (InstructionType::ADC, opcodes::adc, 4, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0x7A => {
                (InstructionType::NOP, opcodes::nop, 2, AddressingMode::Implicit)
            }
            0x7B => {
                (InstructionType::RRA, opcodes::rra, 7, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0x7C => {
                (InstructionType::NOP, opcodes::nop, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x7D => {
                (InstructionType::ADC, opcodes::adc, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x7E => {
                (InstructionType::ROR, opcodes::ror, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x7F => {
                (InstructionType::RRA, opcodes::rra, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0x80 => {
                (InstructionType::NOP, opcodes::nop, 2, AddressingMode::Immediate(self.registers.increment_pc()))
            }
            0x81 => {
                (InstructionType::STA, opcodes::sta, 6, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0x83 => {
                (InstructionType::SAX, opcodes::sax, 6, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0x84 => {
                (InstructionType::STY, opcodes::sty, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x85 => {
                (InstructionType::STA, opcodes::sta, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x86 => {
                (InstructionType::STX, opcodes::stx, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x87 => {
                (InstructionType::SAX, opcodes::sax, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0x88 => {
                (InstructionType::DEY, opcodes::dey, 2, AddressingMode::Implicit)
            }
            0x8A => {
                (InstructionType::TXA, opcodes::txa, 2, AddressingMode::Implicit)
            }
            0x8C => {
                (InstructionType::STY, opcodes::sty, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x8D => {
                (InstructionType::STA, opcodes::sta, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x8E => {
                (InstructionType::STX, opcodes::stx, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x8F => {
                (InstructionType::SAX, opcodes::sax, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0x90 => {
                (InstructionType::BCC, opcodes::bcc, 2, AddressingMode::Relative(RelativeAddress::from(next_byte(self) as i8)))
            }
            0x91 => {
                (InstructionType::STA, opcodes::sta, 6, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0x94 => {
                (InstructionType::STY, opcodes::sty, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x95 => {
                (InstructionType::STA, opcodes::sta, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0x96 => {
                (InstructionType::STX, opcodes::stx, 4, AddressingMode::ZeroPageY(ZeroPageAddress::from(next_byte(self))))
            }
            0x97 => {
                (InstructionType::SAX, opcodes::sax, 4, AddressingMode::ZeroPageY(ZeroPageAddress::from(next_byte(self))))
            }
            0x98 => {
                (InstructionType::TYA, opcodes::tya, 2, AddressingMode::Implicit)
            }
            0x99 => {
                (InstructionType::STA, opcodes::sta, 5, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0x9A => {
                (InstructionType::TXS, opcodes::txs, 2, AddressingMode::Implicit)
            }
            0x9D => {
                (InstructionType::STA, opcodes::sta, 5, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0xA0 => {
                (InstructionType::LDY, opcodes::ldy, 2, AddressingMode::Immediate(self.registers.increment_pc()))
            }
            0xA1 => {
                (InstructionType::LDA, opcodes::lda, 6, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0xA2 => {
                (InstructionType::LDX, opcodes::ldx, 2, AddressingMode::Immediate(self.registers.increment_pc()))
            }
            0xA3 => {
                (InstructionType::LAX, opcodes::lax, 6, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0xA4 => {
                (InstructionType::LDY, opcodes::ldy, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xA5 => {
                (InstructionType::LDA, opcodes::lda, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xA6 => {
                (InstructionType::LDX, opcodes::ldx, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xA7 => {
                (InstructionType::LAX, opcodes::lax, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xA8 => {
                (InstructionType::TAY, opcodes::tay, 2, AddressingMode::Implicit)
            }
            0xA9 => {
                (InstructionType::LDA, opcodes::lda, 2, AddressingMode::Immediate(self.registers.increment_pc()))
            }
            0xAA => {
                (InstructionType::TAX, opcodes::tax, 2, AddressingMode::Implicit)
            }
            0xAC => {
                (InstructionType::LDY, opcodes::ldy, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xAD => {
                (InstructionType::LDA, opcodes::lda, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xAE => {
                (InstructionType::LDX, opcodes::ldx, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xAF => {
                (InstructionType::LAX, opcodes::lax, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xB0 => {
                (InstructionType::BCS, opcodes::bcs, 2, AddressingMode::Relative(RelativeAddress::from(next_byte(self) as i8)))
            }
            0xB1 => {
                (InstructionType::LDA, opcodes::lda, 5, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0xB3 => {
                (InstructionType::LAX, opcodes::lax, 5, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0xB4 => {
                (InstructionType::LDY, opcodes::ldy, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0xB5 => {
                (InstructionType::LDA, opcodes::lda, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0xB6 => {
                (InstructionType::LDX, opcodes::ldx, 4, AddressingMode::ZeroPageY(ZeroPageAddress::from(next_byte(self))))
            }
            0xB7 => {
                (InstructionType::LAX, opcodes::lax, 4, AddressingMode::ZeroPageY(ZeroPageAddress::from(next_byte(self))))
            }
            0xB8 => {
                (InstructionType::CLV, opcodes::clv, 2, AddressingMode::Implicit)
            }
            0xB9 => {
                (InstructionType::LDA, opcodes::lda, 4, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0xBA => {
                (InstructionType::TSX, opcodes::tsx, 2, AddressingMode::Implicit)
            }
            0xBC => {
                (InstructionType::LDY, opcodes::ldy, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0xBD => {
                (InstructionType::LDA, opcodes::lda, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0xBE => {
                (InstructionType::LDX, opcodes::ldx, 4, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0xBF => {
                (InstructionType::LAX, opcodes::lax, 4, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0xC0 => {
                (InstructionType::CPY, opcodes::cpy, 2, AddressingMode::Immediate(self.registers.increment_pc()))
            }
            0xC1 => {
                (InstructionType::CMP, opcodes::cmp, 6, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0xC3 => {
                (InstructionType::DCP, opcodes::dcp, 8, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0xC4 => {
                (InstructionType::CPY, opcodes::cpy, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xC5 => {
                (InstructionType::CMP, opcodes::cmp, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xC6 => {
                (InstructionType::DEC, opcodes::dec, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xC7 => {
                (InstructionType::DCP, opcodes::dcp, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xC8 => {
                (InstructionType::INY, opcodes::iny, 2, AddressingMode::Implicit)
            }
            0xC9 => {
                (InstructionType::CMP, opcodes::cmp, 2, AddressingMode::Immediate(self.registers.increment_pc()))
            }
            0xCA => {
                (InstructionType::DEX, opcodes::dex, 2, AddressingMode::Implicit)
            }
            0xCC => {
                (InstructionType::CPY, opcodes::cpy, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xCD => {
                (InstructionType::CMP, opcodes::cmp, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xCE => {
                (InstructionType::DEC, opcodes::dec, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xCF => {
                (InstructionType::DCP, opcodes::dcp, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xD0 => {
                (InstructionType::BNE, opcodes::bne, 2, AddressingMode::Relative(RelativeAddress::from(next_byte(self) as i8)))
            }
            0xD1 => {
                (InstructionType::CMP, opcodes::cmp, 5, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0xD3 => {
                (InstructionType::DCP, opcodes::dcp, 8, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0xD4 => {
                (InstructionType::NOP, opcodes::nop, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0xD5 => {
                (InstructionType::CMP, opcodes::cmp, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0xD6 => {
                (InstructionType::DEC, opcodes::dec, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0xD7 => {
                (InstructionType::DCP, opcodes::dcp, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0xD8 => {
                (InstructionType::CLD, opcodes::cld, 2, AddressingMode::Implicit)
            }
            0xD9 => {
                (InstructionType::CMP, opcodes::cmp, 4, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0xDA => {
                (InstructionType::NOP, opcodes::nop, 2, AddressingMode::Implicit)
            }
            0xDB => {
                (InstructionType::DCP, opcodes::dcp, 7, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0xDC => {
                (InstructionType::NOP, opcodes::nop, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0xDD => {
                (InstructionType::CMP, opcodes::cmp, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0xDE => {
                (InstructionType::DEC, opcodes::dec, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0xDF => {
                (InstructionType::DCP, opcodes::dcp, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0xE0 => {
                (InstructionType::CPX, opcodes::cpx, 2, AddressingMode::Immediate(self.registers.increment_pc()))
            }
            0xE1 => {
                (InstructionType::SBC, opcodes::sbc, 6, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0xE3 => {
                (InstructionType::ISB, opcodes::isc, 8, AddressingMode::IndexedIndirect(ZeroPageAddress::from(next_byte(self))))
            }
            0xE4 => {
                (InstructionType::CPX, opcodes::cpx, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xE5 => {
                (InstructionType::SBC, opcodes::sbc, 3, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xE6 => {
                (InstructionType::INC, opcodes::inc, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xE7 => {
                (InstructionType::ISB, opcodes::isc, 5, AddressingMode::ZeroPage(ZeroPageAddress::from(next_byte(self))))
            }
            0xE8 => {
                (InstructionType::INX, opcodes::inx, 2, AddressingMode::Implicit)
            }
            0xE9 => {
                (InstructionType::SBC, opcodes::sbc, 2, AddressingMode::Immediate(self.registers.increment_pc()))
            }
            0xEA => {
                (InstructionType::NOP, opcodes::nop, 2, AddressingMode::Implicit)
            }
            0xEB => {
                (InstructionType::SBC, opcodes::sbc, 2, AddressingMode::Immediate(self.registers.increment_pc()))
            }
            0xEC => {
                (InstructionType::CPX, opcodes::cpx, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xED => {
                (InstructionType::SBC, opcodes::sbc, 4, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xEE => {
                (InstructionType::INC, opcodes::inc, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xEF => {
                (InstructionType::ISB, opcodes::isc, 6, AddressingMode::Absolute(AbsoluteAddress::from(next_short(self))))
            }
            0xF0 => {
                (InstructionType::BEQ, opcodes::beq, 2, AddressingMode::Relative(RelativeAddress::from(next_byte(self) as i8)))
            }
            0xF1 => {
                (InstructionType::SBC, opcodes::sbc, 5, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0xF3 => {
                (InstructionType::ISB, opcodes::isc, 8, AddressingMode::IndirectIndexed(ZeroPageAddress::from(next_byte(self))))
            }
            0xF4 => {
                (InstructionType::NOP, opcodes::nop, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0xF5 => {
                (InstructionType::SBC, opcodes::sbc, 4, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0xF6 => {
                (InstructionType::INC, opcodes::inc, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0xF7 => {
                (InstructionType::ISB, opcodes::isc, 6, AddressingMode::ZeroPageX(ZeroPageAddress::from(next_byte(self))))
            }
            0xF8 => {
                (InstructionType::SED, opcodes::sed, 2, AddressingMode::Implicit)
            }
            0xF9 => {
                (InstructionType::SBC, opcodes::sbc, 4, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0xFA => {
                (InstructionType::NOP, opcodes::nop, 2, AddressingMode::Implicit)
            }
            0xFB => {
                (InstructionType::ISB, opcodes::isc, 7, AddressingMode::AbsoluteY(AbsoluteAddress::from(next_short(self))))
            }
            0xFC => {
                (InstructionType::NOP, opcodes::nop, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0xFD => {
                (InstructionType::SBC, opcodes::sbc, 4, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0xFE => {
                (InstructionType::INC, opcodes::inc, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            0xFF => {
                (InstructionType::ISB, opcodes::isc, 7, AddressingMode::AbsoluteX(AbsoluteAddress::from(next_short(self))))
            }
            _ => panic!("Unknown opcode {}", opcode)
        };

        OpCode::new(foo.0, foo.3, foo.2, foo.1, opcode, UNOFFICIAL_OPCODES.contains(&(opcode as i32)))
    }

    fn debug_print(&mut self, op: &OpCode) {
        self.debug_print_op(&op);
    }

    fn debug_print_op(&mut self, op: &OpCode) {
        print!("{:02X}", op.code());

        match op.mode() {
            Implicit => { print!("       ") }
            Accumulator => { print!("       ") }
            Immediate(addr) => { print!(" {:02X}    ", self.memory.read8x(addr)) }
            Indirect(addr) => { print!(" {:02X} {:02X} ", addr.low(), addr.high()) }
            ZeroPage(addr) => { print!(" {:02X}    ", addr.absolute().low()) }
            ZeroPageX(addr) => { print!(" {:02X}    ", addr.absolute().low()) }
            ZeroPageY(addr) => { print!(" {:02X}    ", addr.absolute().low()) }
            Relative(addr) => { print!(" {:02X}    ", addr.to_i8()) }
            Absolute(addr) => { print!(" {:02X} {:02X} ", addr.low(), addr.high()) }
            AbsoluteX(addr) => { print!(" {:02X} {:02X} ", addr.low(), addr.high()) }
            AbsoluteY(addr) => { print!(" {:02X} {:02X} ", addr.low(), addr.high()) }
            IndexedIndirect(addr) => { print!(" {:02X}    ", addr.absolute().low()) }
            IndirectIndexed(addr) => { print!(" {:02X}    ", addr.absolute().low()) }
        }

        if op.is_unofficial() {
            print!("*");
        }
        else {
            print!(" ");
        }

        print!("{:?}", op.instruction());

        match op.mode() {
            Implicit => {print!("                             ")},
            Accumulator => {print!(" A                           ")},
            Immediate(addr) => {print!(" #${:02X}                        ", self.memory.read8x(addr))},
            Indirect(addr) => {
                let address_high = if addr.low() == 0xFF { AbsoluteAddress::from_u8(0x00, addr.high()) } else { addr.offset_wrap(1) };

                let low = self.memory.read8x(addr);
                let high = self.memory.read8x(&address_high);

                print!(" (${:02X}{:02X}) = {:02X}{:02X}              ", addr.high(), addr.low(), high, low);
            },
            ZeroPage(addr) => {print!(" ${:02X} = {:02X}                    ", addr.absolute().low(), self.memory.read8x(&addr.absolute()))},
            ZeroPageX(addr) => {
                let absolute_addr = addr.offset(self.registers.x()).absolute();
                let value = self.memory.read8x(&absolute_addr);
                print!(" ${:02X},X @ {:02X} = {:02X}             ", addr.absolute().low(), absolute_addr.low(), value);
            },
            ZeroPageY(addr) => {
                let absolute_addr = addr.offset(self.registers.y()).absolute();
                let value = self.memory.read8x(&absolute_addr);
                print!(" ${:02X},Y @ {:02X} = {:02X}             ", addr.absolute().low(), absolute_addr.low(), value);
            },
            Relative(addr) => {
                let absolute_address = self.registers.pc().to_u16() as i32 + (addr.to_i8() as i32);
                print!(" ${:04X}                       ", absolute_address as u16)
            },
            Absolute(addr) => {
                let instr_is_jump = match op.instruction() {
                    InstructionType::JMP => true,
                    InstructionType::JSR => true,
                    _ => false
                };
                /*let target_is_value = match op.instruction() {
                    InstructionType::LDX => true,
                    InstructionType::STX => true,
                    InstructionType::LDY => true,
                    InstructionType::STY => true,
                    InstructionType::LDA => true,
                    InstructionType::STA => true,
                    InstructionType::BIT => true,
                    InstructionType::ORA => true,
                    InstructionType::AND => true,
                    InstructionType::EOR => true,
                    InstructionType::ADC => true,
                    InstructionType::CMP => true,
                    InstructionType::SBC => true,
                    InstructionType::CPX => true,
                    InstructionType::CPY => true,
                    InstructionType::LSR => true,
                    InstructionType::ASL => true,
                    InstructionType::ROR => true,
                    InstructionType::ROL => true,

                    _ => false
                };*/
                print!(" ${:02X}{:02X} {}                  ", addr.high(), addr.low(), if !instr_is_jump { format!("= {:02X}", self.memory.read8x(addr)) } else { "    ".to_string() } );
            },
            AbsoluteX(addr) => {

                let mut low = addr.low();
                let mut high = addr.high();

                low = low.wrapping_add(self.registers.x());

                if low < self.registers.x() {
                    // page_boundary_crossed = true;
                    high = high.wrapping_add(1);
                }

                let value = self.memory.read8x(&AbsoluteAddress::from_u8(low, high));

                print!(" ${:02X}{:02X},X @ {:02X}{:02X} = {:02X}         ", addr.high(), addr.low(), high, low, value);
            },
            AbsoluteY(addr) => {

                let mut low = addr.low();
                let mut high = addr.high();

                low = low.wrapping_add(self.registers.y());

                if low < self.registers.y() {
                    // page_boundary_crossed = true;
                    high = high.wrapping_add(1);
                }

                let value = self.memory.read8x(&AbsoluteAddress::from_u8(low, high));

                print!(" ${:02X}{:02X},Y @ {:02X}{:02X} = {:02X}         ", addr.high(), addr.low(), high, low, value);
            },
            IndexedIndirect(addr) => {
                let result_zero_page = addr.offset(self.registers.x());
                let low = self.memory.read8x(&result_zero_page.absolute());
                let high = self.memory.read8x(&result_zero_page.offset(1).absolute());
                let absolute_address = AbsoluteAddress::from_u8(low, high);
                let value = self.memory.read8x(&absolute_address);
                print!(" (${:02X},X) @ {:02X} = {:04X} = {:02X}    ", addr.absolute().low(), result_zero_page.absolute().low(), absolute_address.to_u16(), value);
            },
            IndirectIndexed(addr) => {

                let low1 = self.memory.read8x(&addr.absolute());
                let high1 = self.memory.read8x(&addr.offset(1).absolute());// TODO: Wrapping_add here?

                let low2 = low1.wrapping_add(self.registers.y());
                let mut high2 = high1;
                if low2 < self.registers.y() {
                    // page_boundary_crossed = true;
                    high2 = high1.wrapping_add(1);
                }

                let value = self.memory.read8x(&AbsoluteAddress::from_u8(low2, high2));

                print!(" (${:02X}),Y = {:02X}{:02X} @ {:02X}{:02X} = {:02X}  ", addr.absolute().low(), high1, low1, high2, low2, value);
            },
        }

        print!("A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} ",
               self.registers.accumulator(), self.registers.x(), self.registers.y(), self.registers.status(),
               self.registers.stack() & 0xFF);

    }

    pub(crate) fn process(&mut self, op: OpCode) -> i32 {
        self.debug_print(&op);

        op.func()(&mut self.registers, self.memory, op.mode()) + op.cycles()
    }
}