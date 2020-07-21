use crate::cpu::CPU;
use crate::cpuregisters::CPURegisters;
use crate::ram_controller::RamController;
use crate::cartridge::Cartridge;
use crate::ppu_registers::PPURegisters;
use crate::vram_controller::VRAMController;
use crate::ppu::PPU;

mod cpu;
mod cpuregisters;
mod ram_controller;
mod opcodes;
mod cartridge;
mod ppu_registers;
mod vram_controller;
mod ppu;

fn main()
{
    let mut vram = VRAMController::new();
    let mut ppuregs = PPURegisters::new();
    let mut ppu = PPU::new(&mut vram, &mut ppuregs);
    let mut memory = RamController::new(&mut ppu);
    // let c = Cartridge::load("./roms/nestest.nes");
    let c = Cartridge::load("../../roms/nestest.nes");

    if c.prg_rom_banks().len() > 1 {
        memory.load_prg_bank1(&c.prg_rom_banks()[0]);
        memory.load_prg_bank1(&c.prg_rom_banks()[1]);
    } else {
        memory.load_prg_bank1(&c.prg_rom_banks()[0]);
        memory.load_prg_bank2(&c.prg_rom_banks()[0]);
    }


    let mut cpu = CPU::new(&mut memory);
    cpu.reset();

    let mut total_cycles = 7;
    let mut should_break = false;
    loop {
        if cpu.registers.pc() == 0xC66E {
            should_break = true;
        }
        print!("{:04X}  ", cpu.registers.pc());

        if cpu.registers.pc() == 0xDCC4 {
            let _fdfd = 5;
        }

        let regs_copy = cpu.registers.clone();

        let cycles = cpu.process_instruction();
        print!("A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
               regs_copy.accumulator(), regs_copy.x(), regs_copy.y(), regs_copy.status(),
               regs_copy.stack() & 0xFF, total_cycles);

        total_cycles += cycles;
        println!();

        if should_break
        {
            break;
        }
    }
}
