use crate::cpu::CPU;
use crate::cpuregisters::CPURegisters;
use crate::ram_controller::RamController;
use crate::cartridge::Cartridge;

mod cpu;
mod cpuregisters;
mod ram_controller;
mod opcodes;
mod cartridge;

fn main()
{
    let mut cpu = CPU::new();
    let c = Cartridge::load("./roms/nestest.nes");

    loop {
        let cycles = cpu.process_instruction();

    }


    println!("Hello, world!");
}
