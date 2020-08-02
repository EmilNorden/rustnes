use crate::cpuregisters::CPURegisters;
use crate::ram_controller::RamController;

pub fn push(regs: &mut CPURegisters, mem: &mut RamController, value: u8) {
    mem.write8(regs.stack(), value);
    regs.decrement_stack();
}

pub(crate) fn pop(regs: &mut CPURegisters, mem: &RamController) -> u8 {
    regs.increment_stack();
    mem.read8(regs.stack())
}