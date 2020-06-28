use crate::cpuregisters::{CPURegisters, CPUFlags};
use crate::ram_controller::RamController;
use crate::opcodes::addressing_mode as mode;

fn push_stack(regs: &mut CPURegisters, mem: &mut RamController, value: u8) {
    mem.write8(regs.stack(), value);
    regs.decrement_stack();
}

fn pop_stack(regs: &mut CPURegisters, mem: &RamController) -> u8 {
    regs.increment_stack();
    mem.read8(regs.stack())
}

pub(crate) fn brk_implied(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    // Write program counter and status flag to stack (TODO Make this code more expressive?)
    push_stack(regs, mem, (regs.pc() & 0xFF) as u8);
    push_stack(regs, mem, ((regs.pc() >> 8) & 0xFF) as u8);

    // From the nesdev wiki:
    // In the byte pushed, bit 5 is always set to 1, and bit 4 is 1 if from an
    // instruction (PHP or BRK) or 0 if from an interrupt line being pulled low
    // (/IRQ or /NMI)
    push_stack(regs, mem, regs.status() | CPUFlags::Unused as u8 | CPUFlags::BreakCommand as u8);

    regs.set_flag(CPUFlags::BreakCommand);

    // set PC to address at IRQ interrupt vector 0xFFFE
    regs.set_pc(mem.read16(0xFFFE));

    7
}

fn ora(regs: &mut CPURegisters, value: u8) {
    regs.set_accumulator(regs.accumulator() | value);
}

pub(crate) fn ora_immediate(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::immediate(regs, mem);
    ora(regs, mem.read8(address));

    2
}

pub(crate) fn ora_zero_page(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    ora(regs, mem.read8(address));

    3
}

pub(crate) fn ora_zero_page_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::zero_page_x(regs, mem);
    ora(regs, mem.read8(address));

    4
}

pub(crate) fn ora_absolute(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    ora(regs, mem.read8(address));

    4
}

pub(crate) fn ora_absolute_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let result = mode::absolute_indexed(regs, mem, regs.x());
    ora(regs, mem.read8(result.address));

    if result.page_boundary_crossed { 5 } else { 4 }
}

pub(crate) fn ora_absolute_y(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let result = mode::absolute_indexed(regs, mem, regs.y());
    ora(regs, mem.read8(result.address));

    if result.page_boundary_crossed { 5 } else { 4 }
}

pub(crate) fn ora_indirect_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::indexed_indirect(regs, mem);
    ora(regs, mem.read8(address));

    6
}

pub(crate) fn ora_indirect_y(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let result = mode::indirect_indexed(regs, mem);
    ora(regs, mem.read8(result.address));

    if result.page_boundary_crossed { 6 } else { 5 }
}

fn asl(regs: &mut CPURegisters, mem: &mut RamController, address: u16) {
    let old_value = mem.read8(address);
    let new_value = old_value << 1;

    regs.set_flag_if(CPUFlags::Carry, (old_value & 0x80) == 0x80);
    regs.set_flag_if(CPUFlags::Zero, new_value == 0);
    regs.set_flag_if(CPUFlags::Sign, (new_value & 0x80) == 0x80);

    mem.write8(address, new_value);
}

pub(crate) fn asl_accumulator(regs: &mut CPURegisters) -> i32 {
    regs.set_flag_if(CPUFlags::Carry, (regs.accumulator() & 0x80) == 0x80);
    regs.set_accumulator(regs.accumulator() << 1);

    2
}

pub(crate) fn asl_zero_page(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    asl(regs, mem, address);

    5
}

pub(crate) fn asl_zero_page_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page_x(regs, mem);
    asl(regs, mem, address);

    6
}

pub(crate) fn asl_absolute(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    asl(regs, mem, address);

    6
}

pub(crate) fn asl_absolute_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::absolute_indexed(regs, mem, regs.x()).address;
    asl(regs, mem, address);

    //TODO: Why is this opcode not affected by page boundary crosses?
    7
}

pub(crate) fn pla_implied(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let value = pop_stack(regs, mem);
    regs.set_accumulator(value);

    4
}

pub(crate) fn rts_implied(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let low = pop_stack(regs, mem);
    let high = pop_stack(regs, mem);

    let address = low as u16 | (((high as u16) << 8) + 1);
    regs.set_pc(address);

    6
}

fn adc(regs: &mut CPURegisters, value: u8) {
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

pub(crate) fn adc_immediate(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::immediate(regs, mem);
    adc(regs, mem.read8(address));

    2
}

pub(crate) fn adc_zero_page(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    adc(regs, mem.read8(address));

    3
}

pub(crate) fn adc_zero_page_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::zero_page_x(regs, mem);
    adc(regs, mem.read8(address));

    4
}

pub(crate) fn adc_absolute(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    adc(regs, mem.read8(address));

    4
}

pub(crate) fn adc_absolute_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let result = mode::absolute_indexed(regs, mem, regs.x());

    adc(regs, mem.read8(result.address));

    if result.page_boundary_crossed { 5 } else { 4 }
}

pub(crate) fn adc_absolute_y(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let result = mode::absolute_indexed(regs, mem, regs.y());

    adc(regs, mem.read8(result.address));

    if result.page_boundary_crossed { 5 } else { 4 }
}

pub(crate) fn adc_indirect_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::indexed_indirect(regs, mem);
    adc(regs, mem.read8(address));

    6
}

pub(crate) fn adc_indirect_y(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let result = mode::indirect_indexed(regs, mem);

    adc(regs, mem.read8(result.address));

    if result.page_boundary_crossed { 6 } else { 5 }
}

pub(crate) fn sei_implied(regs: &mut CPURegisters) -> i32 {
    regs.set_flag(CPUFlags::InterruptDisable);

    2
}

pub(crate) fn bcs_relative(regs: &mut CPURegisters, mem: &RamController) -> i32
{
    let relative_address = mode::relative(regs, mem);
    if regs.flag(CPUFlags::Carry) {
        if regs.offset_pc(relative_address) {
            return 4;
        }
        return 3;
    }

    2
}

pub(crate) fn bne_relative(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let relative_address = mode::relative(regs, mem);
    if !regs.flag(CPUFlags::Zero) {
        if regs.offset_pc(relative_address) {
            return 4;
        }
        return 3;
    }

    2
}

pub(crate) fn cld_implied(regs: &mut CPURegisters) -> i32 {
    regs.clear_flag(CPUFlags::ClearDecimalMode);

    2
}

pub(crate) fn ldy_immediate(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::immediate(regs, mem);
    regs.set_y(mem.read8(address));

    2
}

pub(crate) fn ldy_zero_page(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    regs.set_y(mem.read8(address));

    3
}

pub(crate) fn ldy_zero_page_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::zero_page_x(regs, mem);
    regs.set_y(mem.read8(address));

    4
}

pub(crate) fn ldy_absolute(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    regs.set_y(mem.read8(address));

    4
}

pub(crate) fn ldy_absolute_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let result = mode::absolute_indexed(regs, mem, regs.x());
    regs.set_y(mem.read8(result.address));

    if result.page_boundary_crossed { 5 } else { 4 }
}

pub(crate) fn ldx_immediate(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::immediate(regs, mem);
    regs.set_x(mem.read8(address));

    2
}

pub(crate) fn ldx_zero_page(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    regs.set_x(mem.read8(address));

    3
}

pub(crate) fn ldx_zero_page_y(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::zero_page_y(regs, mem);
    regs.set_x(mem.read8(address));

    4
}

pub(crate) fn ldx_absolute(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    regs.set_x(mem.read8(address));

    4
}

pub(crate) fn ldx_absolute_y(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let result = mode::absolute_indexed(regs, mem, regs.y());
    regs.set_x(mem.read8(result.address));

    if result.page_boundary_crossed { 5 } else { 4 }
}

pub(crate) fn sta_indirect_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    mem.write8(mode::indexed_indirect(regs, mem), regs.accumulator());

    6
}

pub(crate) fn sta_indirect_y(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    mem.write8(mode::indirect_indexed(regs, mem).address, regs.accumulator());

    6
}

pub(crate) fn sta_zero_page(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    mem.write8(mode::zero_page(regs, mem), regs.accumulator());

    3
}

pub(crate) fn sta_zero_page_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    mem.write8(mode::zero_page_x(regs, mem), regs.accumulator());

    4
}

pub(crate) fn sta_absolute(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    mem.write8(mode::absolute(regs, mem), regs.accumulator());

    4
}

pub(crate) fn sta_absolute_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    mem.write8(mode::absolute_indexed(regs, mem, regs.x()).address, regs.accumulator());

    5
}

pub(crate) fn sta_absolute_y(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    mem.write8(mode::absolute_indexed(regs, mem, regs.y()).address, regs.accumulator());

    5
}

pub(crate) fn stx_zero_page(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    mem.write8(mode::zero_page(regs, mem), regs.x());

    3
}

pub(crate) fn stx_zero_page_y(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    mem.write8(mode::zero_page_y(regs, mem), regs.x());

    4
}

pub(crate) fn stx_absolute(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    mem.write8(mode::absolute(regs, mem), regs.x());

    4
}

pub(crate) fn sty_zero_page(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    mem.write8(mode::zero_page(regs, mem), regs.y());

    3
}

pub(crate) fn sty_zero_page_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    mem.write8(mode::zero_page_x(regs, mem), regs.y());

    4
}

pub(crate) fn sty_absolute(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    mem.write8(mode::absolute(regs, mem), regs.y());

    4
}

pub(crate) fn tay_implied(regs: &mut CPURegisters) -> i32 {
    regs.set_y(regs.accumulator());

    2
}

pub(crate) fn tax_implied(regs: &mut CPURegisters) -> i32 {
    regs.set_x(regs.accumulator());

    2
}

pub(crate) fn tsx_implied(regs: &mut CPURegisters) -> i32 {
    regs.set_x(regs.stack() as u8);

    2
}

pub(crate) fn txs_implied(regs: &mut CPURegisters) -> i32 {
    regs.set_stack(regs.x().into());

    2
}

pub(crate) fn txa_implied(regs: &mut CPURegisters) -> i32 {
    regs.set_accumulator(regs.x());

    2
}

pub(crate) fn tya_implied(regs: &mut CPURegisters) -> i32 {
    regs.set_accumulator(regs.y());

    2
}

pub(crate) fn bcc_relative(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let relative_address = mode::relative(regs, mem);
    if !regs.flag(CPUFlags::Carry) {
        if regs.offset_pc(relative_address) {
            return 4;
        }
        return 3;
    }

    2
}

fn lax(regs: &mut CPURegisters, value: u8) {
    regs.set_accumulator(value);
    regs.set_x(regs.accumulator());
}

pub(crate) fn lax_indirect_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::indexed_indirect(regs, mem);
    lax(regs, mem.read8(address));

    6
}

pub(crate) fn lax_indirect_y(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let addressing = mode::indirect_indexed(regs, mem);
    lax(regs, mem.read8(addressing.address));

    if addressing.page_boundary_crossed { 6 } else { 5 }
}

pub(crate) fn lax_zero_page(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    lax(regs, mem.read8(address));

    3
}

pub(crate) fn lax_zero_page_y(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::zero_page_y(regs, mem);
    lax(regs, mem.read8(address));

    4
}

pub(crate) fn lax_absolute(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    lax(regs, mem.read8(address));

    4
}

pub(crate) fn lax_absolute_y(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.y());
    lax(regs, mem.read8(addressing.address));

    if addressing.page_boundary_crossed { 5 } else { 4 }
}

fn sax(regs: &CPURegisters, mem: &mut RamController, address: u16) {
    mem.write8(address, regs.accumulator() & regs.x());
}

pub(crate) fn sax_indirect_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::indexed_indirect(regs, mem);
    sax(regs, mem, address);

    6
}

pub(crate) fn sax_zero_page(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    sax(regs, mem, address);

    6
}

pub(crate) fn sax_zero_page_y(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page_y(regs, mem);
    sax(regs, mem, address);

    //TODO How can this be 4 cycles when the vanilla sax zero page is 6 cycles?
    4
}

pub(crate) fn sax_absolute(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    sax(regs, mem, address);

    4
}

pub(crate) fn lda_indirect_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::indexed_indirect(regs, mem);
    regs.set_accumulator(mem.read8(address));

    6
}

pub(crate) fn lda_indirect_y(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let addressing = mode::indirect_indexed(regs, mem);
    regs.set_accumulator(mem.read8(addressing.address));

    if addressing.page_boundary_crossed { 6 } else { 5 }
}

pub(crate) fn lda_immediate(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::immediate(regs, mem);
    regs.set_accumulator(mem.read8(address));

    2
}

pub(crate) fn lda_zero_page(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    regs.set_accumulator(mem.read8(address));

    3
}

pub(crate) fn lda_zero_page_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::zero_page_x(regs, mem);
    regs.set_accumulator(mem.read8(address));

    4
}

pub(crate) fn lda_absolute(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    regs.set_accumulator(mem.read8(address));

    4
}

pub(crate) fn lda_absolute_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.x());
    regs.set_accumulator(mem.read8(addressing.address));

    if addressing.page_boundary_crossed { 5 } else { 4 }
}

pub(crate) fn lda_absolute_y(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.y());
    regs.set_accumulator(mem.read8(addressing.address));

    if addressing.page_boundary_crossed { 5 } else { 4 }
}

pub(crate) fn bpl_relative(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let relative_address = mode::relative(regs, mem);
    if !regs.flag(CPUFlags::Sign) {
        if regs.offset_pc(relative_address) {
            return 4;
        }
        return 3;
    }

    2
}

pub(crate) fn clc_implied(regs: &mut CPURegisters) -> i32 {
    regs.clear_flag(CPUFlags::Carry);

    2
}

pub(crate) fn jsr_absolute(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let routine_address = mode::absolute(regs, mem);
    let return_address = regs.pc() - 1;

    // I think high byte should be pushed first
    push_stack(regs, mem, (return_address >> 8) as u8);
    push_stack(regs, mem, return_address as u8);

    regs.set_pc(routine_address);

    6
}

fn bit(regs: &mut CPURegisters, value: u8) {
    regs.set_flag_if(CPUFlags::Zero, (value & regs.accumulator()) == 0);
    regs.set_flag_if(CPUFlags::Overflow, (value & CPUFlags::Overflow as u8) != 0);
    regs.set_flag_if(CPUFlags::Sign, (value & CPUFlags::Sign as u8) != 0);

    // TODO Verify that this still works. Not exactly as the original C++ version
}

pub(crate) fn bit_zero_page(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    bit(regs, mem.read8(address));

    3
}

pub(crate) fn bit_absolute(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    bit(regs, mem.read8(address));

    4
}

pub(crate) fn rol_accumulator(regs: &mut CPURegisters) -> i32 {
    let mut result = regs.accumulator() << 1;
    if regs.flag(CPUFlags::Carry) {
        result |= 1;
    }

    regs.set_flag_if(CPUFlags::Carry, (regs.accumulator() & 0x80) == 0x80);

    regs.set_accumulator(result);

    2
}

fn rol(regs: &mut CPURegisters, mem: &mut RamController, address: u16) {
    let old_value = mem.read8(address);
    let mut new_value = old_value << 1;
    if regs.flag(CPUFlags::Carry) {
        new_value |= 1;
    }

    regs.set_flag_if(CPUFlags::Carry, (old_value & 0x80) == 0x80);
    regs.set_flag_if(CPUFlags::Sign, (new_value & 0x80) == 0x80);

    mem.write8(address, new_value);
}

pub(crate) fn rol_zero_page(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    rol(regs, mem, address);

    5
}

pub(crate) fn rol_zero_page_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page_x(regs, mem);
    rol(regs, mem, address);

    6
}

pub(crate) fn rol_absolute(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    rol(regs, mem, address);

    6
}

pub(crate) fn rol_absolute_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.x());
    rol(regs, mem, addressing.address);

    7
}

pub(crate) fn ror_accumulator(regs: &mut CPURegisters) -> i32 {
    let mut result = regs.accumulator() >> 1;

    if regs.flag(CPUFlags::Carry) {
        result |= 0x80;
    }

    regs.set_flag_if(CPUFlags::Carry, (regs.accumulator() & 1) == 1);

    regs.set_accumulator(result);

    2
}

fn ror(regs: &mut CPURegisters, mem: &mut RamController, address: u16) {
    let old_value = mem.read8(address);
    let mut new_value = old_value >> 1;

    if regs.flag(CPUFlags::Carry) {
        new_value |= 0x80;
    }

    regs.set_flag_if(CPUFlags::Carry, (old_value & 1) == 1);
    regs.set_flag_if(CPUFlags::Sign, (new_value & 0x80) == 0x80);

    mem.write8(address, new_value);
}

pub(crate) fn ror_zero_page(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    ror(regs, mem, address);

    5
}

pub(crate) fn ror_zero_page_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page_x(regs, mem);
    ror(regs, mem, address);

    6
}

pub(crate) fn ror_absolute(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    ror(regs, mem, address);

    6
}

pub(crate) fn ror_absolute_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.x());
    ror(regs, mem, addressing.address);

    7
}

fn and(regs: &mut CPURegisters, mem: &RamController, address: u16) {
    let value = mem.read8(address);
    regs.set_accumulator(regs.accumulator() & value);
}

pub(crate) fn and_zero_page(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    and(regs, mem, address);

    3
}

pub(crate) fn and_indirect_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::indexed_indirect(regs, mem);
    and(regs, mem, address);

    6
}

pub(crate) fn and_indirect_y(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let addressing = mode::indirect_indexed(regs, mem);
    and(regs, mem, addressing.address);

    if addressing.page_boundary_crossed { 6 } else { 5 }
}

pub(crate) fn and_immediate(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::immediate(regs, mem);
    and(regs, mem, address);

    2
}

pub(crate) fn and_absolute(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    and(regs, mem, address);

    4
}

pub(crate) fn and_zero_page_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::zero_page_x(regs, mem);
    and(regs, mem, address);

    4
}

pub(crate) fn and_absolute_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.x());
    and(regs, mem, addressing.address);

    if addressing.page_boundary_crossed { 5 } else { 4 }
}

pub(crate) fn and_absolute_y(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.y());
    and(regs, mem, addressing.address);

    if addressing.page_boundary_crossed { 5 } else { 4 }
}

pub(crate) fn bmi_relative(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let relative_address = mode::relative(regs, mem);
    if regs.flag(CPUFlags::Sign) {
        if regs.offset_pc(relative_address) {
            return 4;
        }
        return 3;
    }

    2
}

pub(crate) fn sec_implied(regs: &mut CPURegisters) -> i32 {
    regs.set_flag(CPUFlags::Carry);

    2
}

pub(crate) fn pha_implied(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    push_stack(regs, mem, regs.accumulator());

    3
}

pub(crate) fn lsr_accumulator(regs: &mut CPURegisters) -> i32 {
    let old_value = regs.accumulator();
    regs.set_flag_if(CPUFlags::Carry, (old_value & 1) == 1);
    regs.set_accumulator(old_value >> 1);

    2
}

fn lsr(regs: &mut CPURegisters, mem: &mut RamController, address: u16) {
    let old_value = mem.read8(address);
    let new_value = old_value >> 1;
    regs.set_flag_if(CPUFlags::Carry, (old_value & 1) == 1);
    regs.set_flag_if(CPUFlags::Zero, new_value == 0);
    regs.set_flag_if(CPUFlags::Sign, (new_value & 0x80) == 0x80);

    mem.write8(address, old_value >> 1);
}

pub(crate) fn lsr_zero_page(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    lsr(regs, mem, address);

    5
}

pub(crate) fn lsr_zero_page_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page_x(regs, mem);
    lsr(regs, mem, address);

    6
}

pub(crate) fn lsr_absolute(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    lsr(regs, mem, address);

    6
}

pub(crate) fn lsr_absolute_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.x());
    lsr(regs, mem, addressing.address);

    7
}

pub(crate) fn jmp_absolute(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    regs.set_pc(address);

    3
}

pub(crate) fn jmp_indirect(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::indirect(regs, mem);
    regs.set_pc(address);

    5
}

pub(crate) fn bvc_relative(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let relative_address = mode::relative(regs, mem);

    if !regs.flag(CPUFlags::Overflow) {
        if regs.offset_pc(relative_address) {
            return 4;
        }
        return 3;
    }

    2
}

pub(crate) fn bvs_relative(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let relative_address = mode::relative(regs, mem);

    if regs.flag(CPUFlags::Overflow) {
        if regs.offset_pc(relative_address) {
            return 4;
        }
        return 3;
    }

    2
}

pub(crate) fn cli_implied(regs: &mut CPURegisters) -> i32 {
    regs.clear_flag(CPUFlags::InterruptDisable);

    2
}

pub(crate) fn clv_implied(regs: &mut CPURegisters) -> i32 {
    regs.clear_flag(CPUFlags::Overflow);

    2
}

fn cmp(regs: &mut CPURegisters, register_value: u8, value: u8) {
    regs.set_flag_if(CPUFlags::Carry, register_value >= value);
    regs.set_flag_if(CPUFlags::Zero, register_value == value);

    let result = register_value - value;
    regs.set_flag_if(CPUFlags::Sign, (result & CPUFlags::Sign as u8) == CPUFlags::Sign as u8);
}

pub(crate) fn cmp_immediate(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::immediate(regs, mem);
    cmp(regs, regs.accumulator(), mem.read8(address));

    2
}

pub(crate) fn cmp_zero_page(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    cmp(regs, regs.accumulator(), mem.read8(address));

    3
}

pub(crate) fn cmp_zero_page_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::zero_page_x(regs, mem);
    cmp(regs, regs.accumulator(), mem.read8(address));

    3
}

pub(crate) fn cmp_absolute(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    cmp(regs, regs.accumulator(), mem.read8(address));

    4
}

pub(crate) fn cmp_absolute_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.x());
    cmp(regs, regs.accumulator(), mem.read8(addressing.address));

    if addressing.page_boundary_crossed { 5 } else { 4 }
}

pub(crate) fn cmp_absolute_y(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.y());
    cmp(regs, regs.accumulator(), mem.read8(addressing.address));

    if addressing.page_boundary_crossed { 5 } else { 4 }
}

pub(crate) fn cmp_indirect_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::indexed_indirect(regs, mem);
    cmp(regs, regs.accumulator(), mem.read8(address));

    6
}

pub(crate) fn cmp_indirect_y(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let addressing = mode::indirect_indexed(regs, mem);
    cmp(regs, regs.accumulator(), mem.read8(addressing.address));

    if addressing.page_boundary_crossed { 6 } else { 5 }
}

pub(crate) fn cpx_immediate(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address= mode::immediate(regs, mem);
    cmp(regs, regs.x(), mem.read8(address));

    2
}

pub(crate) fn cpx_zero_page(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address= mode::zero_page(regs, mem);
    cmp(regs, regs.x(), mem.read8(address));

    3
}

pub(crate) fn cpx_absolute(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address= mode::absolute(regs, mem);
    cmp(regs, regs.x(), mem.read8(address));

    4
}

pub(crate) fn cpy_immediate(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address= mode::immediate(regs, mem);
    cmp(regs, regs.y(), mem.read8(address));

    2
}

pub(crate) fn cpy_zero_page(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address= mode::zero_page(regs, mem);
    cmp(regs, regs.y(), mem.read8(address));

    3
}

pub(crate) fn cpy_absolute(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address= mode::absolute(regs, mem);
    cmp(regs, regs.y(), mem.read8(address));

    4
}

fn dec(regs: &mut CPURegisters, mem: &mut RamController, address: u16) {
    let value = mem.read8(address) - 1;
    mem.write8(address, value);

    regs.set_flag_if(CPUFlags::Zero, value == 0);
    regs.set_flag_if(CPUFlags::Sign, (value & CPUFlags::Sign as u8) == CPUFlags::Sign as u8);
}

pub(crate) fn dec_zero_page(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    dec(regs, mem, address);

    5
}

pub(crate) fn dec_zero_page_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page_x(regs, mem);
    dec(regs, mem, address);

    6
}

pub(crate) fn dec_absolute(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    dec(regs, mem, address);

    6
}

pub(crate) fn dec_absolute_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.x());
    dec(regs, mem, addressing.address);

    7
}

pub(crate) fn dex_implied(regs: &mut CPURegisters) -> i32 {
    regs.set_x(regs.x() - 1 );

    2
}

pub(crate) fn dey_implied(regs: &mut CPURegisters) -> i32 {
    regs.set_y(regs.y() - 1 );

    2
}

pub(crate) fn eor(regs: &mut CPURegisters, value: u8) {
    regs.set_accumulator(regs.accumulator() ^ value);
}

pub(crate) fn eor_immediate(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::immediate(regs, mem);
    eor(regs, mem.read8(address));

    2
}

pub(crate) fn eor_zero_page(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    eor(regs, mem.read8(address));

    3
}

pub(crate) fn eor_zero_page_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::zero_page_x(regs, mem);
    eor(regs, mem.read8(address));

    4
}

pub(crate) fn eor_absolute(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    eor(regs, mem.read8(address));

    4
}

pub(crate) fn eor_absolute_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.x());
    eor(regs, mem.read8(addressing.address));

    if addressing.page_boundary_crossed { 5 } else { 4 }
}

pub(crate) fn eor_absolute_y(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.y());
    eor(regs, mem.read8(addressing.address));

    if addressing.page_boundary_crossed { 5 } else { 4 }
}

pub(crate) fn eor_indirect_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::indexed_indirect(regs, mem);
    eor(regs, mem.read8(address));

    6
}

pub(crate) fn eor_indirect_y(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let addressing = mode::indirect_indexed(regs, mem);
    eor(regs, mem.read8(addressing.address));

    if addressing.page_boundary_crossed { 6 } else { 5 }
}

pub(crate) fn rti_implied(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let new_status = pop_stack(regs, mem) | CPUFlags::Unused as u8;
    regs.set_status(new_status);

    let new_pc = pop_stack(regs, mem) as u16 | ((pop_stack(regs, mem) as u16) << 8);
    regs.set_pc(new_pc);

    3
}

fn inc(regs: &mut CPURegisters, mem: &mut RamController, address: u16) {
    let value = mem.read8(address) + 1;
    mem.write8(address, value);

    regs.set_flag_if(CPUFlags::Zero, value == 0);
    regs.set_flag_if(CPUFlags::Sign, (value & CPUFlags::Sign as u8) == CPUFlags::Sign as u8);
}

pub(crate) fn inc_zero_page(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    inc(regs, mem, address);

    5
}

pub(crate) fn inc_zero_page_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page_x(regs, mem);
    inc(regs, mem, address);

    5
}

pub(crate) fn inc_absolute(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    inc(regs, mem, address);

    6
}

pub(crate) fn inc_absolute_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.x());
    inc(regs, mem, addressing.address);

    7
}

pub(crate) fn inx_implied(regs: &mut CPURegisters) -> i32 {
    regs.set_x(regs.x() + 1);

    2
}

pub(crate) fn iny_implied(regs: &mut CPURegisters) -> i32 {
    regs.set_y(regs.y() + 1);

    2
}

fn sbc(regs: &mut CPURegisters, value: u8) {
    adc(regs, !value);
}

pub(crate) fn sbc_immediate(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::immediate(regs, mem);
    sbc(regs, mem.read8(address));
    2
}

pub(crate) fn sbc_zero_page(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    sbc(regs, mem.read8(address));
    3
}

pub(crate) fn sbc_zero_page_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::zero_page_x(regs, mem);
    sbc(regs, mem.read8(address));
    4
}

pub(crate) fn sbc_absolute(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    sbc(regs, mem.read8(address));
    4
}

pub(crate) fn sbc_absolute_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.x());
    sbc(regs, mem.read8(addressing.address));

    if addressing.page_boundary_crossed { 5 } else { 4 }
}

pub(crate) fn sbc_absolute_y(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.y());
    sbc(regs, mem.read8(addressing.address));

    if addressing.page_boundary_crossed { 5 } else { 4 }
}

pub(crate) fn sbc_indirect_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let address = mode::indexed_indirect(regs, mem);
    sbc(regs, mem.read8(address));
    6
}

pub(crate) fn sbc_indirect_y(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let addressing = mode::indirect_indexed(regs, mem);
    sbc(regs, mem.read8(addressing.address));

    if addressing.page_boundary_crossed { 6 } else { 5 }
}

pub(crate) fn sed_implied(regs: &mut CPURegisters) -> i32 {
    regs.set_flag(CPUFlags::ClearDecimalMode);

    2
}

pub(crate) fn nop_implied() -> i32 {
    2
}

pub(crate) fn nop_immediate(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    //TODO All the comments below about reads having no side effects are wrong. I dont know why i thought so a year ago. They all increment the program counter.

    // TODO: This is just here for the output log, can remove later as immediate reads have no side effects
    let address = mode::immediate(regs, mem);
    mem.read8(address);

    2
}

pub(crate) fn nop_zero_page(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    // TODO: This is just here for the output log, can remove later as zero page reads have no side effects
    let address = mode::zero_page(regs, mem);
    mem.read8(address);

    3
}

pub(crate) fn nop_zero_page_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    // TODO: This is just here for the output log, can remove later as zero page reads have no side effects
    let address = mode::zero_page_x(regs, mem);
    mem.read8(address);

    4
}

pub(crate) fn nop_absolute(regs: &mut CPURegisters, mem: &RamController) -> i32 {

    let address = mode::absolute(regs, mem);
    mem.read8(address);

    4
}

pub(crate) fn nop_absolute_x(regs: &mut CPURegisters, mem: &RamController) -> i32 {

    let addressing = mode::absolute_indexed(regs, mem, regs.x());
    mem.read8(addressing.address);

    if addressing.page_boundary_crossed { 5 } else { 4 }
}

pub(crate) fn beq_relative(regs: &mut CPURegisters, mem: &RamController) -> i32 {
    let relative_address = mode::relative(regs, mem);
    if regs.flag(CPUFlags::Zero) {
        if regs.offset_pc(relative_address) {
            return 4;
        }
        return 3;
    }

    2
}

pub(crate) fn dcp_indirect_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::indexed_indirect(regs, mem);
    dec(regs, mem, address);
    cmp(regs, regs.accumulator(), mem.read8(address));

    8
}

pub(crate) fn dcp_indirect_y(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let addressing = mode::indirect_indexed(regs, mem);
    dec(regs, mem, addressing.address);
    cmp(regs, regs.accumulator(), mem.read8(addressing.address));

    8
}

pub(crate) fn dcp_zero_page(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    dec(regs, mem, address);
    cmp(regs, regs.accumulator(), mem.read8(address));

    5
}

pub(crate) fn dcp_zero_page_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page_x(regs, mem);
    dec(regs, mem, address);
    cmp(regs, regs.accumulator(), mem.read8(address));

    6
}

pub(crate) fn dcp_absolute(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    dec(regs, mem, address);
    cmp(regs, regs.accumulator(), mem.read8(address));

    6
}

pub(crate) fn dcp_absolute_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.x());
    dec(regs, mem, addressing.address);
    cmp(regs, regs.accumulator(), mem.read8(addressing.address));

    7
}

pub(crate) fn dcp_absolute_y(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.y());
    dec(regs, mem, addressing.address);
    cmp(regs, regs.accumulator(), mem.read8(addressing.address));

    7
}

fn isc(regs: &mut CPURegisters, mem: &mut RamController, address: u16) {
    inc(regs, mem, address);
    sbc(regs, mem.read8(address));
}

pub(crate) fn isc_indirect_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::indexed_indirect(regs, mem);
    isc(regs, mem, address);
    8
}

pub(crate) fn isc_indirect_y(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let addressing = mode::indirect_indexed(regs, mem);
    isc(regs, mem, addressing.address);
    8
}

pub(crate) fn isc_zero_page(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    isc(regs, mem, address);
    5
}

pub(crate) fn isc_zero_page_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page_x(regs, mem);
    isc(regs, mem, address);
    6
}

pub(crate) fn isc_absolute(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    isc(regs, mem, address);
    6
}

pub(crate) fn isc_absolute_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.x());
    isc(regs, mem, addressing.address);
    7
}

pub(crate) fn isc_absolute_y(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.y());
    isc(regs, mem, addressing.address);
    7
}

fn slo(regs: &mut CPURegisters, mem: &mut RamController, address: u16) {
    asl(regs, mem, address);
    ora(regs, mem.read8(address));
}

pub(crate) fn slo_indirect_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::indexed_indirect(regs, mem);
    slo(regs, mem, address);
    8
}

pub(crate) fn slo_indirect_y(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let addressing = mode::indirect_indexed(regs, mem);
    slo(regs, mem, addressing.address);
    8
}

pub(crate) fn slo_zero_page(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    slo(regs, mem, address);
    5
}

pub(crate) fn slo_zero_page_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page_x(regs, mem);
    slo(regs, mem, address);
    6
}

pub(crate) fn slo_absolute(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    slo(regs, mem, address);
    6
}

pub(crate) fn slo_absolute_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.x());
    slo(regs, mem, addressing.address);
    7
}

pub(crate) fn slo_absolute_y(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.y());
    slo(regs, mem, addressing.address);
    7
}

pub(crate) fn rla(regs: &mut CPURegisters, mem: &mut RamController, address: u16) {
    rol(regs, mem, address);
    and(regs, mem, address);
}

pub(crate) fn rla_indirect_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::indexed_indirect(regs, mem);
    rla(regs, mem, address);
    8
}

pub(crate) fn rla_indirect_y(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let addressing = mode::indirect_indexed(regs, mem);
    rla(regs, mem, addressing.address);
    8
}

pub(crate) fn rla_zero_page(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    rla(regs, mem, address);
    5
}

pub(crate) fn rla_zero_page_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page_x(regs, mem);
    rla(regs, mem, address);
    6
}

pub(crate) fn rla_absolute(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    rla(regs, mem, address);
    6
}

pub(crate) fn rla_absolute_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.x());
    rla(regs, mem, addressing.address);
    7
}

pub(crate) fn rla_absolute_y(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.y());
    rla(regs, mem, addressing.address);
    7
}

pub(crate) fn sre(regs: &mut CPURegisters, mem: &mut RamController, address: u16) {
    lsr(regs, mem, address);
    eor(regs, mem.read8(address));
}

pub(crate) fn sre_indirect_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::indexed_indirect(regs, mem);
    sre(regs, mem, address);
    8
}

pub(crate) fn sre_indirect_y(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let addressing = mode::indirect_indexed(regs, mem);
    sre(regs, mem, addressing.address);
    8
}

pub(crate) fn sre_zero_page(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    sre(regs, mem, address);
    5
}

pub(crate) fn sre_zero_page_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page_x(regs, mem);
    sre(regs, mem, address);
    6
}

pub(crate) fn sre_absolute(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    sre(regs, mem, address);
    6
}

pub(crate) fn sre_absolute_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.x());
    sre(regs, mem, addressing.address);
    7
}

pub(crate) fn sre_absolute_y(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.y());
    sre(regs, mem, addressing.address);
    7
}

pub(crate) fn rra(regs: &mut CPURegisters, mem: &mut RamController, address: u16) {
    ror(regs, mem, address);
    adc(regs, mem.read8(address));
}

pub(crate) fn rra_indirect_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::indexed_indirect(regs, mem);
    rra(regs, mem, address);
    8
}

pub(crate) fn rra_indirect_y(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let addressing = mode::indirect_indexed(regs, mem);
    rra(regs, mem, addressing.address);
    8
}

pub(crate) fn rra_zero_page(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page(regs, mem);
    rra(regs, mem, address);
    5
}

pub(crate) fn rra_zero_page_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::zero_page_x(regs, mem);
    rra(regs, mem, address);
    6
}

pub(crate) fn rra_absolute(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let address = mode::absolute(regs, mem);
    rra(regs, mem, address);
    6
}

pub(crate) fn rra_absolute_x(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.x());
    rra(regs, mem, addressing.address);
    7
}

pub(crate) fn rra_absolute_y(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    let addressing = mode::absolute_indexed(regs, mem, regs.y());
    rra(regs, mem, addressing.address);
    7
}

pub(crate) fn php_implied(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    // From the nesdev wiki:
    // In the byte pushed, bit 5 is always set to 1, and bit 4 is 1 if from an
    // instruction (PHP or BRK) or 0 if from an interrupt line being pulled low
    // (/IRQ or /NMI)
    push_stack(regs, mem, regs.status() | CPUFlags::Unused as u8 | CPUFlags::BreakCommand as u8);

    3
}

pub(crate) fn plp_implied(regs: &mut CPURegisters, mem: &mut RamController) -> i32 {
    // PLP ignores bit 4 and 5. 5 is unused and should always be 1.
    let status = pop_stack(regs, mem);

    regs.set_status((status | CPUFlags::Unused as u8) & !(CPUFlags::BreakCommand as u8));

    4
}

mod addressing_mode
{
    pub struct AddressingResult {
        pub address: u16,
        pub page_boundary_crossed: bool,
    }


    use crate::cpuregisters::CPURegisters;
    use crate::ram_controller::RamController;

    pub(crate) fn immediate(regs: &mut CPURegisters, mem: &RamController) -> u16 {
        let address = regs.increment_pc();
        mem.read8(address) as u16
    }

    pub(crate) fn absolute(regs: &mut CPURegisters, mem: &RamController) -> u16 {
        let low = mem.read8(regs.increment_pc());
        let high = mem.read8(regs.increment_pc());

        low as u16 | ((high as u16) << 8)
    }

    pub(crate) fn absolute_indexed(regs: &mut CPURegisters, mem: &RamController, index: u8) -> AddressingResult {
        let mut page_boundary_crossed = false;
        let mut low = mem.read8(regs.increment_pc());
        let mut high = mem.read8(regs.increment_pc());

        low = low + index;

        if low < index {
            page_boundary_crossed = true;
            high = high + 1;
        }

        AddressingResult {
            address: low as u16 | ((high as u16) << 8),
            page_boundary_crossed,
        }
    }

    pub(crate) fn zero_page(regs: &mut CPURegisters, mem: &RamController) -> u16 {
        mem.read8(regs.increment_pc()) as u16
    }

    pub(crate) fn zero_page_x(regs: &mut CPURegisters, mem: &RamController) -> u16 {
        (mem.read8(regs.increment_pc()) + regs.x()) as u16
    }

    pub(crate) fn zero_page_y(regs: &mut CPURegisters, mem: &RamController) -> u16 {
        (mem.read8(regs.increment_pc()) + regs.y()) as u16
    }

    pub(crate) fn relative(regs: &mut CPURegisters, mem: &RamController) -> i8 {
        mem.read8(regs.increment_pc()) as i8
    }

    pub(crate) fn indirect(regs: &mut CPURegisters, mem: &RamController) -> u16 {
        let low = mem.read8(regs.increment_pc());
        let high = mem.read8(regs.increment_pc());

        let address = low as u16 | ((high as u16) << 8);

        // from documentation at obelisk.me.uk:
        // NB:
        // An original 6502 has does not correctly fetch the target address if the
        // indirect vector falls on a page boundary (e.g. $xxFF where xx is any value
        // from $00 to $FF). In this case fetches the LSB from $xxFF as expected but
        // takes the MSB from $xx00. This is fixed in some later chips like the 65SC02
        // so for compatibility always ensure the indirect vector is not at the end of
        // the page.

        let address_high = if low == 0xFF { address & 0xFF00 } else { address + 1 };

        mem.read8(address) as u16 | ((mem.read8(address_high) as u16) << 8)
    }

    pub(crate) fn indexed_indirect(regs: &mut CPURegisters, mem: &RamController) -> u16 {
        let low = mem.read8(regs.increment_pc());
        let zero_page_address = (low + regs.x()) as u16;

        mem.read8(zero_page_address) as u16 | ((mem.read8(zero_page_address + 1) as u16) << 8)
    }

    pub(crate) fn indirect_indexed(regs: &mut CPURegisters, mem: &RamController) -> AddressingResult {
        let zero_page_address = mem.read8(regs.increment_pc());

        let mut low = mem.read8(zero_page_address as u16);
        let mut high = mem.read8((zero_page_address + 1) as u16);

        let mut page_boundary_crossed = false;
        low = low + regs.y();
        if low < regs.y() {
            page_boundary_crossed = true;
            high = high + 1;
        }

        AddressingResult {
            page_boundary_crossed,
            address: low as u16 | ((high as u16) << 8),
        }
    }
}