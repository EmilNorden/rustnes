use crate::cpuregisters::{CPURegisters, CPUFlags};
use crate::ram_controller::RamController;
use crate::stack;
use crate::cpu::OpCodeFn;
use crate::opcodes::AddressingMode::{Absolute, Indirect, ZeroPage, ZeroPageX, ZeroPageY, AbsoluteX, AbsoluteY, IndexedIndirect, IndirectIndexed, Immediate};

pub(crate) struct AddressingResult {
    pub address: AbsoluteAddress,
    pub page_boundary_crossed: bool,
}

pub(crate) struct ValueAddressingResult {
    pub address: Option<AbsoluteAddress>,
    pub value: u8,
    pub page_boundary_crossed: bool,
}

pub(crate) struct ZeroPageAddress(u8);

#[derive(Copy, Clone, PartialEq)]
pub(crate) struct AbsoluteAddress(u16);

pub(crate) struct RelativeAddress(i8);

impl ZeroPageAddress {
    pub(crate) fn from(addr: u8) -> ZeroPageAddress {
        ZeroPageAddress { 0: addr }
    }
    pub(crate) fn absolute(&self) -> AbsoluteAddress { AbsoluteAddress { 0: self.0 as u16 } }
    pub(crate) fn offset(&self, val: u8) -> ZeroPageAddress { ZeroPageAddress { 0: self.0.wrapping_add(val) } }
}

impl AbsoluteAddress {
    pub(crate) fn from(addr: u16) -> AbsoluteAddress {
        AbsoluteAddress { 0: addr }
    }

    pub(crate) fn from_u8(low: u8, high: u8) -> AbsoluteAddress {
        AbsoluteAddress::from((high as u16) << 8 | low as u16)
    }

    pub(crate) fn low(&self) -> u8 {
        self.0 as u8
    }

    pub(crate) fn high(&self) -> u8 {
        (self.0 >> 8) as u8
    }

    pub(crate) fn to_u16(&self) -> u16 { self.0 }

    pub(crate) fn offset_wrap(&self, value: i16) -> AbsoluteAddress {
        if value >= 0 {
            AbsoluteAddress { 0: self.0.wrapping_add(value as u16) }
        }
        else {
            AbsoluteAddress { 0: self.0.wrapping_sub(value.abs() as u16) }
        }
    }

    pub(crate) fn offset(&self, value: i8) -> AbsoluteAddress {
        AbsoluteAddress { 0: ((self.0 as i32) + value as i32) as u16 }
    }
}

impl RelativeAddress {
    pub(crate) fn from(addr: i8) -> RelativeAddress { RelativeAddress { 0: addr } }
    pub(crate) fn to_i8(&self) -> i8 { self.0 }
}

#[derive(Debug)]
pub(crate) enum InstructionType {
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DCP,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    ISB,
    JMP,
    JSR,
    LAX,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    RLA,
    RRA,
    ROL,
    ROR,
    RTI,
    RTS,
    SAX,
    SBC,
    SEC,
    SED,
    SEI,
    SLO,
    SRE,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
}

pub(crate) enum AddressingMode {
    Implicit,
    Accumulator,
    Immediate(AbsoluteAddress),
    Indirect(AbsoluteAddress),
    ZeroPage(ZeroPageAddress),
    ZeroPageX(ZeroPageAddress),
    ZeroPageY(ZeroPageAddress),
    Relative(RelativeAddress),
    Absolute(AbsoluteAddress),
    AbsoluteX(AbsoluteAddress),
    AbsoluteY(AbsoluteAddress),
    IndexedIndirect(ZeroPageAddress),
    IndirectIndexed(ZeroPageAddress),
}

pub(crate) struct OpCode {
    instruction: InstructionType,
    mode: AddressingMode,
    cycles: i32,
    func: OpCodeFn,
    code: u8,
    is_unofficial: bool,
}

impl OpCode {
    pub fn new(instruction: InstructionType, mode: AddressingMode, cycles: i32, func: OpCodeFn, code: u8, is_unofficial: bool) -> OpCode {
        OpCode {
            instruction,
            mode,
            cycles,
            func,
            code,
            is_unofficial,
        }
    }

    pub fn instruction(&self) -> &InstructionType {
        &self.instruction
    }

    pub fn mode(&self) -> &AddressingMode {
        &self.mode
    }

    pub fn cycles(&self) -> i32 { self.cycles }

    pub fn func(&self) -> OpCodeFn { self.func }

    pub fn code(&self) -> u8 { self.code }

    pub fn is_unofficial(&self) -> bool { self.is_unofficial }
}

pub(crate) fn brk(regs: &mut CPURegisters, mem: &mut RamController, _: &AddressingMode) -> i32 {
    // Write program counter and status flag to stack (TODO Make this code more expressive?)
    stack::push(regs, mem, regs.pc().low());
    stack::push(regs, mem, regs.pc().high());

    print!("        ");

    // From the nesdev wiki:
    // In the byte pushed, bit 5 is always set to 1, and bit 4 is 1 if from an
    // instruction (PHP or BRK) or 0 if from an interrupt line being pulled low
    // (/IRQ or /NMI)
    stack::push(regs, mem, regs.status() | CPUFlags::Unused as u8 | CPUFlags::BreakCommand as u8);

    regs.set_flag(CPUFlags::BreakCommand);

    // set PC to address at IRQ interrupt vector 0xFFFE
    regs.set_pc(AbsoluteAddress::from(mem.read16(0xFFFE)));

    0
}

fn set_u8(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode, value: u8) -> i32 {
    match mode {
        AddressingMode::Relative(_) => panic!("Cannot call set_u8 on relative addressing mode"),
        AddressingMode::Indirect(_) => panic!("Cannot call set_u8 on indirect addressing mode."),
        AddressingMode::Implicit => panic!("Cannot call set_u8 on implicit addressing mode"),
        AddressingMode::Accumulator => {
            regs.set_accumulator(value);
            0
        }
        _ => {
            let address = resolve_address(regs, mem, mode).unwrap().address;
            mem.write8x(&address, value)
        }
    }
}

fn resolve_relative(mode: &AddressingMode) -> &RelativeAddress {
    match mode {
        AddressingMode::Relative(address) => address,
        _ => panic!("Trying to take relative address from non-relative mode"),
    }
}

fn resolve_u8(regs: &CPURegisters, mem: &RamController, mode: &AddressingMode) -> ValueAddressingResult {
    match mode {
        AddressingMode::Relative(_) => panic!("Cannot take the value of a relative opcode. It is only used for branching."),
        AddressingMode::Indirect(_) => panic!("Cannot take the value of an indirect opcode. It is only used for JMP."),
        AddressingMode::Implicit => panic!("Cannot take the value of an implicit opcode"),
        AddressingMode::Accumulator => ValueAddressingResult { address: None, value: regs.accumulator(), page_boundary_crossed: false },
        _ => {
            let result = resolve_address(regs, mem, mode).unwrap();
            ValueAddressingResult {
                address: Some(result.address),
                value: mem.read8x(&result.address),
                page_boundary_crossed: result.page_boundary_crossed,
            }
        }
    }
}

fn resolve_address(regs: &CPURegisters, mem: &RamController, mode: &AddressingMode) -> Option<AddressingResult> {
    fn absolute_indexed(address: &AbsoluteAddress, index: u8) -> AddressingResult {
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

    /*let f = AddressingMode::Implicit;

    match f {
        AddressingMode::Implicit => {},
        AddressingMode::Accumulator => {},
        Immediate(_) => {},
        Indirect(_) => {},
        ZeroPage(_) => {},
        ZeroPageX(_) => {},
        ZeroPageY(_) => {},
        Relative(_) => {},
        Absolute(_) => {},
        AbsoluteX(_) => {},
        AbsoluteY(_) => {},
        IndexedIndirect(_) => {},
        IndirectIndexed(_) => {},
    }

    None*/


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

            let address_high = if address.low() == 0xFF { AbsoluteAddress::from_u8(0x00, address.high()) } else { address.offset_wrap(1) };

            let low = mem.read8x(address);
            let high = mem.read8x(&address_high);

            Some(AddressingResult {
                address: AbsoluteAddress::from_u8(low, high),
                page_boundary_crossed: false,
            })
        }
        ZeroPage(address) => Some(AddressingResult { address: address.absolute(), page_boundary_crossed: false }),
        ZeroPageX(address) => Some(AddressingResult { address: address.offset(regs.x()).absolute(), page_boundary_crossed: false }),
        ZeroPageY(address) => Some(AddressingResult { address: address.offset(regs.y()).absolute(), page_boundary_crossed: false }),
        AbsoluteX(address) => Some(absolute_indexed(address, regs.x())),
        AbsoluteY(address) => Some(absolute_indexed(address, regs.y())),
        IndexedIndirect(address) => {
            let zero_page_address = address.offset(regs.x());  // TODO: Should I do wrapping_add here?

            let low = mem.read8x(&zero_page_address.absolute());
            let high = mem.read8x(&zero_page_address.offset(1).absolute());

            Some(AddressingResult {
                address: AbsoluteAddress::from_u8(low, high),
                page_boundary_crossed: false,
            })
        }
        IndirectIndexed(address) => {
            let mut low = mem.read8x(&address.absolute());
            let mut high = mem.read8x(&address.offset(1).absolute());// TODO: Wrapping_add here?

            let mut page_boundary_crossed = false;
            low = low.wrapping_add(regs.y());
            if low < regs.y() {
                page_boundary_crossed = true;
                high = high.wrapping_add(1);
            }

            Some(AddressingResult {
                page_boundary_crossed,
                address: AbsoluteAddress::from_u8(low, high),
            })
        }
        Immediate(address) => Some(AddressingResult { address: *address, page_boundary_crossed: false }),
        _ => None,
    }
}

pub(crate) fn ora(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let addressing = resolve_u8(regs, mem, mode);

    regs.set_accumulator(regs.accumulator() | addressing.value);

    if addressing.page_boundary_crossed { 1 } else { 0 }
}

pub(crate) fn asl(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let old_value = resolve_u8(regs, mem, mode).value;
    let new_value = old_value << 1;

    regs.set_flag_if(CPUFlags::Carry, (old_value & 0x80) == 0x80);
    regs.set_flag_if(CPUFlags::Zero, new_value == 0);
    regs.set_flag_if(CPUFlags::Sign, (new_value & 0x80) == 0x80);

    set_u8(regs, mem, mode, new_value)
}

pub(crate) fn pla(regs: &mut CPURegisters, mem: &mut RamController, _: &AddressingMode) -> i32 {
    let value = stack::pop(regs, mem);
    regs.set_accumulator(value);

    0
}

pub(crate) fn rts(regs: &mut CPURegisters, mem: &mut RamController, _: &AddressingMode) -> i32 {
    let low = stack::pop(regs, mem);
    let high = stack::pop(regs, mem);

    let address = AbsoluteAddress::from_u8(low, high);
    regs.set_pc(address.offset(1));

    0
}

pub(crate) fn adc(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let value = resolve_u8(regs, mem, mode).value;
    let result = regs.accumulator() as u16 + value as u16 + (regs.status() as u16 & CPUFlags::Carry as u16);

    // If we wrapped around, set carry flag
    regs.set_flag_if(CPUFlags::Carry, result > 0xFF);

    // Set overflow if sign bit is incorrect
    // That is, if the numbers added have identical signs, but the sign of the
    // result differs, set overflow.
    let new_value = result as u8;
    regs.set_flag_if(CPUFlags::Overflow, (((!(regs.accumulator() ^ value)) & (regs.accumulator() ^ new_value)) & 0x80) == 0x80);

    regs.set_accumulator(new_value);

    0
}

pub(crate) fn sei(regs: &mut CPURegisters, _: &mut RamController, _: &AddressingMode) -> i32 {
    regs.set_flag(CPUFlags::InterruptDisable);

    0
}

pub(crate) fn bcs(regs: &mut CPURegisters, _: &mut RamController, mode: &AddressingMode) -> i32
{
    let address = resolve_relative(mode);
    branch(regs, address, regs.flag(CPUFlags::Carry))
}

pub(crate) fn bne(regs: &mut CPURegisters, _: &mut RamController, mode: &AddressingMode) -> i32 {
    let address = resolve_relative(mode);
    branch(regs, address, !regs.flag(CPUFlags::Zero))
}

pub(crate) fn cld(regs: &mut CPURegisters, _: &mut RamController, _: &AddressingMode) -> i32 {
    regs.clear_flag(CPUFlags::ClearDecimalMode);

    0
}

pub(crate) fn ldy(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let addressing = resolve_u8(regs, mem, mode);
    regs.set_y(addressing.value);

    if addressing.page_boundary_crossed { 1 } else { 0 }
}

pub(crate) fn ldx(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let addressing = resolve_u8(regs, mem, mode);
    regs.set_x(addressing.value);

    if addressing.page_boundary_crossed { 1 } else { 0 }
}

pub(crate) fn sta(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    set_u8(regs, mem, mode, regs.accumulator())
}

pub(crate) fn stx(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    set_u8(regs, mem, mode, regs.x())
}

pub(crate) fn sty(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    set_u8(regs, mem, mode, regs.y())
}

pub(crate) fn tay(regs: &mut CPURegisters, _: &mut RamController, _: &AddressingMode) -> i32 {
    regs.set_y(regs.accumulator());

    0
}

pub(crate) fn tax(regs: &mut CPURegisters, _: &mut RamController, _: &AddressingMode) -> i32 {
    regs.set_x(regs.accumulator());

    0
}

pub(crate) fn tsx(regs: &mut CPURegisters, _: &mut RamController, _: &AddressingMode) -> i32 {
    regs.set_x(regs.stack() as u8);

    0
}

pub(crate) fn txs(regs: &mut CPURegisters, _: &mut RamController, _: &AddressingMode) -> i32 {
    regs.set_stack(regs.x().into());

    0
}

pub(crate) fn txa(regs: &mut CPURegisters, _: &mut RamController, _: &AddressingMode) -> i32 {
    regs.set_accumulator(regs.x());

    0
}

pub(crate) fn tya(regs: &mut CPURegisters, _: &mut RamController, _: &AddressingMode) -> i32 {
    regs.set_accumulator(regs.y());

    0
}

pub(crate) fn branch(regs: &mut CPURegisters, relative_address: &RelativeAddress, condition: bool) -> i32 {
    if condition {
        if regs.offset_pcx(relative_address) {
            return 2;
        }

        return 1;
    }

    0
}

pub(crate) fn bcc(regs: &mut CPURegisters, _: &mut RamController, mode: &AddressingMode) -> i32 {
    let address = resolve_relative(mode);
    branch(regs, address, !regs.flag(CPUFlags::Carry))
}

pub(crate) fn lax(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let addressing = resolve_u8(regs, mem, mode);
    regs.set_accumulator(addressing.value);
    regs.set_x(addressing.value);

    if addressing.page_boundary_crossed { 1 } else { 0 }
}

pub(crate) fn sax(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    set_u8(regs, mem, mode, regs.accumulator() & regs.x())
}

pub(crate) fn lda(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let addressing = resolve_u8(regs, mem, mode);
    regs.set_accumulator(addressing.value);

    if addressing.page_boundary_crossed { 1 } else { 0 }
}

pub(crate) fn bpl(regs: &mut CPURegisters, _: &mut RamController, mode: &AddressingMode) -> i32 {
    let address = resolve_relative(mode);
    branch(regs, address, !regs.flag(CPUFlags::Sign))
}

pub(crate) fn clc(regs: &mut CPURegisters, _: &mut RamController, _: &AddressingMode) -> i32 {
    regs.clear_flag(CPUFlags::Carry);

    0
}

pub(crate) fn jsr(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let routine_address = resolve_address(regs, mem, mode).unwrap().address;
    let return_address = regs.pc().offset(-1);

    // I think high byte should be pushed first
    stack::push(regs, mem, return_address.high());
    stack::push(regs, mem, return_address.low());

    regs.set_pc(routine_address);

    0
}

pub(crate) fn bit(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let value = resolve_u8(regs, mem, mode).value;
    regs.set_flag_if(CPUFlags::Zero, (value & regs.accumulator()) == 0);
    regs.set_flag_if(CPUFlags::Overflow, (value & CPUFlags::Overflow as u8) != 0);
    regs.set_flag_if(CPUFlags::Sign, (value & CPUFlags::Sign as u8) != 0);

    // TODO Verify that this still works. Not exactly as the original C++ version
    0
}

pub(crate) fn rol(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let old_value = resolve_u8(regs, mem, mode).value;
    let mut new_value = old_value << 1;
    if regs.flag(CPUFlags::Carry) {
        new_value |= 1;
    }

    regs.set_flag_if(CPUFlags::Carry, (old_value & 0x80) == 0x80);
    regs.set_flag_if(CPUFlags::Sign, (new_value & 0x80) == 0x80);

    set_u8(regs, mem, mode, new_value)
}

pub(crate) fn ror(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let old_value = resolve_u8(regs, mem, mode).value;
    let mut new_value = old_value >> 1;

    if regs.flag(CPUFlags::Carry) {
        new_value |= 0x80;
    }

    regs.set_flag_if(CPUFlags::Carry, (old_value & 1) == 1);
    regs.set_flag_if(CPUFlags::Sign, (new_value & 0x80) == 0x80);

    set_u8(regs, mem, mode, new_value)
}

pub(crate) fn and(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let addressing = resolve_u8(regs, mem, mode);
    regs.set_accumulator(regs.accumulator() & addressing.value);

    if addressing.page_boundary_crossed { 1 } else { 0 }
}

pub(crate) fn bmi(regs: &mut CPURegisters, _: &mut RamController, mode: &AddressingMode) -> i32 {
    let address = resolve_relative(mode);
    branch(regs, address, regs.flag(CPUFlags::Sign))
}

pub(crate) fn sec(regs: &mut CPURegisters, _: &mut RamController, _: &AddressingMode) -> i32 {
    regs.set_flag(CPUFlags::Carry);

    0
}

pub(crate) fn pha(regs: &mut CPURegisters, mem: &mut RamController, _: &AddressingMode) -> i32 {
    stack::push(regs, mem, regs.accumulator());

    0
}

pub(crate) fn lsr(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let old_value = resolve_u8(regs, mem, mode).value;
    let new_value = old_value >> 1;
    regs.set_flag_if(CPUFlags::Carry, (old_value & 1) == 1);
    regs.set_flag_if(CPUFlags::Zero, new_value == 0);
    regs.set_flag_if(CPUFlags::Sign, (new_value & 0x80) == 0x80);

    set_u8(regs, mem, mode, new_value)
}

pub(crate) fn jmp(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let address = resolve_address(regs, mem, mode).unwrap().address;
    regs.set_pc(address);

    0
}

pub(crate) fn bvc(regs: &mut CPURegisters, _: &mut RamController, mode: &AddressingMode) -> i32 {
    let address = resolve_relative(mode);
    branch(regs, address, !regs.flag(CPUFlags::Overflow))
}

pub(crate) fn bvs(regs: &mut CPURegisters, _: &mut RamController, mode: &AddressingMode) -> i32 {
    let address = resolve_relative(mode);
    branch(regs, address, regs.flag(CPUFlags::Overflow))
}

pub(crate) fn cli(regs: &mut CPURegisters, _: &mut RamController, _: &AddressingMode) -> i32 {
    regs.clear_flag(CPUFlags::InterruptDisable);

    0
}

pub(crate) fn clv(regs: &mut CPURegisters, _: &mut RamController, _: &AddressingMode) -> i32 {
    regs.clear_flag(CPUFlags::Overflow);

    0
}

pub(crate) fn cmp(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let addressing = resolve_u8(regs, mem, mode);
    compare(regs, regs.accumulator(), addressing.value);

    if addressing.page_boundary_crossed { 1 } else { 0 }
}

pub(crate) fn cpx(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let addressing = resolve_u8(regs, mem, mode);
    compare(regs, regs.x(), addressing.value);

    0
}

pub(crate) fn cpy(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let addressing = resolve_u8(regs, mem, mode);
    compare(regs, regs.y(), addressing.value);

    0
}

fn compare(regs: &mut CPURegisters, register_value: u8, value: u8) {
    regs.set_flag_if(CPUFlags::Carry, register_value >= value);
    regs.set_flag_if(CPUFlags::Zero, register_value == value);

    let result = register_value.wrapping_sub(value);
    regs.set_flag_if(CPUFlags::Sign, (result & CPUFlags::Sign as u8) == CPUFlags::Sign as u8);
}

pub(crate) fn dec(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let value = resolve_u8(regs, mem, mode).value.wrapping_sub(1);
    let cycles = set_u8(regs, mem, mode, value);

    regs.set_flag_if(CPUFlags::Zero, value == 0);
    regs.set_flag_if(CPUFlags::Sign, (value & CPUFlags::Sign as u8) == CPUFlags::Sign as u8);

    cycles
}

pub(crate) fn dex(regs: &mut CPURegisters, _: &mut RamController, _: &AddressingMode) -> i32 {
    regs.set_x(regs.x().wrapping_sub(1));

    0
}

pub(crate) fn dey(regs: &mut CPURegisters, _: &mut RamController, _: &AddressingMode) -> i32 {
    regs.set_y(regs.y().wrapping_sub(1));

    0
}

pub(crate) fn eor(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let addressing = resolve_u8(regs, mem, mode);
    regs.set_accumulator(regs.accumulator() ^ addressing.value);

    if addressing.page_boundary_crossed { 1 } else { 0 }
}

pub(crate) fn rti(regs: &mut CPURegisters, mem: &mut RamController, _: &AddressingMode) -> i32 {
    let new_status = stack::pop(regs, mem) | CPUFlags::Unused as u8;
    regs.set_status(new_status);

    let low = stack::pop(regs, mem);
    let high = stack::pop(regs, mem);
    let new_pc = AbsoluteAddress::from_u8(low, high);
    regs.set_pc(new_pc);

    0
}

pub(crate) fn inc(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let value = resolve_u8(regs, mem, mode).value.wrapping_add(1);
    let cycles = set_u8(regs, mem, mode, value);

    regs.set_flag_if(CPUFlags::Zero, value == 0);
    regs.set_flag_if(CPUFlags::Sign, (value & CPUFlags::Sign as u8) == CPUFlags::Sign as u8);

    cycles
}

pub(crate) fn inx(regs: &mut CPURegisters, _: &mut RamController, _: &AddressingMode) -> i32 {
    regs.set_x(regs.x().wrapping_add(1));

    0
}

pub(crate) fn iny(regs: &mut CPURegisters, _: &mut RamController, _: &AddressingMode) -> i32 {
    regs.set_y(regs.y().wrapping_add(1));

    0
}


pub(crate) fn sbc(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let addressing = resolve_u8(regs, mem, mode);
    let value = !resolve_u8(regs, mem, mode).value;
    let result = regs.accumulator() as u16 + value as u16 + (regs.status() as u16 & CPUFlags::Carry as u16);

    // If we wrapped around, set carry flag
    regs.set_flag_if(CPUFlags::Carry, result > 0xFF);

    // Set overflow if sign bit is incorrect
    // That is, if the numbers added have identical signs, but the sign of the
    // result differs, set overflow.
    let new_value = result as u8;
    regs.set_flag_if(CPUFlags::Overflow, (((!(regs.accumulator() ^ value)) & (regs.accumulator() ^ new_value)) & 0x80) == 0x80);

    regs.set_accumulator(new_value);

    if addressing.page_boundary_crossed { 1 } else { 0 }
}

pub(crate) fn sed(regs: &mut CPURegisters, _: &mut RamController, _: &AddressingMode) -> i32 {
    regs.set_flag(CPUFlags::ClearDecimalMode);

    0
}

pub(crate) fn nop(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    match resolve_address(regs, mem, mode) {
        None => 0,
        Some(x) => if x.page_boundary_crossed { 1 } else { 0 }
    }
}


pub(crate) fn beq(regs: &mut CPURegisters, _: &mut RamController, mode: &AddressingMode) -> i32 {
    let address = resolve_relative(mode);
    branch(regs, address, regs.flag(CPUFlags::Zero))
}

pub(crate) fn dcp(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let addressing = resolve_u8(regs, mem, mode);

    // DEC
    let value = addressing.value.wrapping_sub(1);
    let cycles = set_u8(regs, mem, mode, value);

    regs.set_flag_if(CPUFlags::Zero, value == 0);
    regs.set_flag_if(CPUFlags::Sign, (value & CPUFlags::Sign as u8) == CPUFlags::Sign as u8);

    compare(regs, regs.accumulator(), value);

    cycles
}

pub(crate) fn isc(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let cycles;
    // INC
    {
        let addressing = resolve_u8(regs, mem, mode);
        let value = addressing.value.wrapping_add(1);
        cycles = set_u8(regs, mem, mode, value);

        regs.set_flag_if(CPUFlags::Zero, value == 0);
        regs.set_flag_if(CPUFlags::Sign, (value & CPUFlags::Sign as u8) == CPUFlags::Sign as u8);
    }


    // SBC
    {
        let value = !resolve_u8(regs, mem, mode).value;
        let result = regs.accumulator() as u16 + value as u16 + (regs.status() as u16 & CPUFlags::Carry as u16);

        // If we wrapped around, set carry flag
        regs.set_flag_if(CPUFlags::Carry, result > 0xFF);

        // Set overflow if sign bit is incorrect
        // That is, if the numbers added have identical signs, but the sign of the
        // result differs, set overflow.
        let new_value = result as u8;
        regs.set_flag_if(CPUFlags::Overflow, (((!(regs.accumulator() ^ value)) & (regs.accumulator() ^ new_value)) & 0x80) == 0x80);

        regs.set_accumulator(new_value);
    }


    cycles
}

pub(crate) fn slo(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let result = resolve_u8(regs, mem, mode);
    let old_value = result.value;
    // ASL

    let new_value = old_value << 1;
    regs.set_flag_if(CPUFlags::Carry, (old_value & 0x80) == 0x80);
    regs.set_flag_if(CPUFlags::Zero, new_value == 0);
    regs.set_flag_if(CPUFlags::Sign, (new_value & 0x80) == 0x80);
    let cycles = set_u8(regs, mem, mode, new_value);

    // ORA
    regs.set_accumulator(regs.accumulator() | new_value);

    cycles
}

pub(crate) fn rla(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let addressing = resolve_u8(regs, mem, mode);
    // ROL
    let old_value = addressing.value;
    let mut new_value = old_value << 1;
    if regs.flag(CPUFlags::Carry) {
        new_value |= 1;
    }

    regs.set_flag_if(CPUFlags::Carry, (old_value & 0x80) == 0x80);
    regs.set_flag_if(CPUFlags::Sign, (new_value & 0x80) == 0x80);

    let cycles = set_u8(regs, mem, mode, new_value);

    // AND
    let value = mem.read8x(&addressing.address.unwrap());
    regs.set_accumulator(regs.accumulator() & value);

    cycles
}

pub(crate) fn sre(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let addressing = resolve_u8(regs, mem, mode);

    // LSR
    let old_value = addressing.value;
    let new_value = old_value >> 1;
    regs.set_flag_if(CPUFlags::Carry, (old_value & 1) == 1);
    regs.set_flag_if(CPUFlags::Zero, new_value == 0);
    regs.set_flag_if(CPUFlags::Sign, (new_value & 0x80) == 0x80);

    let cycles = set_u8(regs, mem, mode, new_value);

    // EOR
    let value = mem.read8x(&addressing.address.unwrap());
    regs.set_accumulator(regs.accumulator() ^ value);

    cycles
}

pub(crate) fn rra(regs: &mut CPURegisters, mem: &mut RamController, mode: &AddressingMode) -> i32 {
    let addressing = resolve_u8(regs, mem, mode);
    // ROR
    let old_value = addressing.value;
    let mut new_value = old_value >> 1;

    if regs.flag(CPUFlags::Carry) {
        new_value |= 0x80;
    }

    regs.set_flag_if(CPUFlags::Carry, (old_value & 1) == 1);
    regs.set_flag_if(CPUFlags::Sign, (new_value & 0x80) == 0x80);

    let cycles = set_u8(regs, mem, mode, new_value);

    // ADC
    let value = mem.read8x(&addressing.address.unwrap());
    let result = regs.accumulator() as u16 + value as u16 + (regs.status() as u16 & CPUFlags::Carry as u16);

    // If we wrapped around, set carry flag
    regs.set_flag_if(CPUFlags::Carry, result > 0xFF);

    // Set overflow if sign bit is incorrect
    // That is, if the numbers added have identical signs, but the sign of the
    // result differs, set overflow.
    let new_value = result as u8;
    regs.set_flag_if(CPUFlags::Overflow, (((!(regs.accumulator() ^ value)) & (regs.accumulator() ^ new_value)) & 0x80) == 0x80);

    regs.set_accumulator(new_value);

    cycles
}

pub(crate) fn php(regs: &mut CPURegisters, mem: &mut RamController, _: &AddressingMode) -> i32 {
    // From the nesdev wiki:
    // In the byte pushed, bit 5 is always set to 1, and bit 4 is 1 if from an
    // instruction (PHP or BRK) or 0 if from an interrupt line being pulled low
    // (/IRQ or /NMI)
    stack::push(regs, mem, regs.status() | CPUFlags::Unused as u8 | CPUFlags::BreakCommand as u8);

    0
}

pub(crate) fn plp(regs: &mut CPURegisters, mem: &mut RamController, _: &AddressingMode) -> i32 {
    // PLP ignores bit 4 and 5. 5 is unused and should always be 1.
    let status = stack::pop(regs, mem);

    regs.set_status((status | CPUFlags::Unused as u8) & !(CPUFlags::BreakCommand as u8));

    0
}