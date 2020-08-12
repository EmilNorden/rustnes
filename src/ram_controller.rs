use crate::cartridge::PrgRomBank;
use crate::ppu::PPU;
use crate::ppu_registers::PPURegisters;
use std::cell::{Cell, RefCell};
use crate::vram_controller::VRAMController;
use crate::opcodes::AbsoluteAddress;

pub struct RamController<'data> {
    ppu_regs: &'data Cell<PPURegisters>,
    vram: &'data RefCell<VRAMController>,
    memory: [u8; 0x10000],
    pub     foobar: Vec<u16>,
}

impl RamController<'_> {
    const PRG_BANK1_LOCATION: usize = 0x8000;
    const PRG_BANK2_LOCATION: usize = 0xC000;
    const PRG_BANK_SIZE: usize = 0x4000;

    pub fn new<'a>(ppu_regs: &'a Cell<PPURegisters>, vram: &'a RefCell<VRAMController>) -> RamController<'a> {
        RamController {
            ppu_regs,
            vram,
            memory: [0; 0x10000],
            foobar: vec![]
        }
    }

    pub(crate) fn read8x(&self, address: &AbsoluteAddress) -> u8 {
        0
    }

    pub fn read8(&self, address: u16) -> u8 {
        let translated_address = self.translate_address(address);

        self.read_ppu_registers(address).unwrap_or(self.memory[translated_address])
    }

    pub fn read16(&self, address: u16) -> u16 {
        // I will assume that we will never attempt to read 16bit that crosses the
        // border of two ranges i.e the range that (address) occupies is not the
        // same as (address+1). Otherwise I would have to do
        // translate_address(address) AND translate_address(address+1)

        let real_address = self.translate_address(address);

        self.memory[real_address] as u16 | ((self.memory[real_address + 1] as u16) << 8)
    }

    pub fn write8(&mut self, address: u16, value: u8) -> i32 {
        self.memory[self.translate_address(address)] = value;

        self.write_ppu_registers(address, value)
    }

    pub(crate) fn write8x(&mut self, address: &AbsoluteAddress, value: u8) -> i32 {

        -5000
    }

    pub(crate) fn load_prg_bank1(&mut self, rom: &PrgRomBank) {
        let prg_bank1 = &mut self.memory[RamController::PRG_BANK1_LOCATION..RamController::PRG_BANK1_LOCATION + RamController::PRG_BANK_SIZE];
        prg_bank1.copy_from_slice(rom.get_data());
    }

    pub(crate) fn load_prg_bank2(&mut self, rom: &PrgRomBank) {
        let prg_bank2 = &mut self.memory[RamController::PRG_BANK2_LOCATION..RamController::PRG_BANK2_LOCATION + RamController::PRG_BANK_SIZE];
        prg_bank2.copy_from_slice(rom.get_data());
    }

    fn is_lower_ram_range(&self, address: u16) -> bool {
        (address & 0x1FFF) == address
    }

    fn is_io_mirror_range(&self, address: u16) -> bool {
        (address & 0x3FFF) == address
    }

    fn translate_address(&self, address: u16) -> usize {
        if self.is_lower_ram_range(address) {
            // Address range $0000-$07FF is mirrored 3 times
            return (address & 0x07FF) as usize;
        } else if self.is_io_mirror_range(address) {
            // Address range $2000-$2007 is mirrored multiple times
            return (address & 0x2007) as usize;
        }

        address as usize
    }

    fn read_ppu_registers(&self, address: u16) -> Option<u8> {
        match address {
            0x2002 => {
                let mut ppu_regs = self.ppu_regs.get();
                let status = ppu_regs.status();
                self.ppu_regs.set(ppu_regs);
                Some(status)
            },
            0x2004 => {
                Some(self.vram.borrow().read8(address))
            },
            0x2007 => {
                let mut regs = self.ppu_regs.get();
                let ppuaddr = regs.ppuaddr();
                regs.increment_ppuaddr();
                self.ppu_regs.set(regs);
                Some(self.vram.borrow().read8(ppuaddr))
            }
            _ => None
        }
    }

    fn write_ppu_registers(&mut self, address: u16, value: u8) -> i32 {
        match address {
            0x2000 => {
                let mut regs = self.ppu_regs.get();
                regs.set_ppuctrl(value);
                self.ppu_regs.set(regs);

                0
            }
            0x2001 => {
                let mut regs = self.ppu_regs.get();
                regs.set_ppumask(value);
                self.ppu_regs.set(regs);

                0
            }
            0x2003 => {
                let mut regs = self.ppu_regs.get();
                regs.set_oamaddr(value);
                self.ppu_regs.set(regs);

                0
            },
            0x2004 => {
                let mut regs = self.ppu_regs.get();
                self.vram.borrow_mut().write_oam(regs.oamaddr(), value);
                if value > 0 {
                    println!("test");
                }
                regs.set_oamaddr(regs.oamaddr() + 1);
                self.ppu_regs.set(regs);

                0
            }
            0x2005 => {
                let mut regs = self.ppu_regs.get();
                regs.set_ppuscroll(value);
                self.ppu_regs.set(regs);

                0
            }
            0x2006 => {
                let mut regs = self.ppu_regs.get();
                regs.set_ppuaddr(value);
                self.ppu_regs.set(regs);

                0
            }
            0x2007 => {
                let mut regs = self.ppu_regs.get();
                self.vram.borrow_mut().write8(regs.ppuaddr(), value);
                self.foobar.push(regs.ppuaddr());
                regs.increment_ppuaddr();
                self.ppu_regs.set(regs);

                0
            }
            0x4014 => {
                let cpu_page_address = (value as usize) << 8;
                let cpu_page= &self.memory[cpu_page_address..(cpu_page_address + 0xFF)];

                self.vram.borrow_mut().write_oam_dma(self.ppu_regs.get().oamaddr(), cpu_page);

                513
            }
            _ => 0
        }
    }
}