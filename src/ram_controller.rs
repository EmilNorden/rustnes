use crate::cartridge::PrgRomBank;
use crate::ppu::PPU;

pub struct RamController<'a> {
    ppu: &'a mut PPU<'a>,
    memory: [u8; 0x10000],
}

impl RamController<'_> {
    const PRG_BANK1_LOCATION: usize = 0x8000;
    const PRG_BANK2_LOCATION: usize = 0xC000;
    const PRG_BANK_SIZE: usize = 0x4000;

    pub fn new<'a>(ppu: &'a mut PPU<'a>) -> RamController<'a> {
        RamController {
            ppu,
            memory: [0; 0x10000]
        }
    }
    pub fn read8(&self, address: u16) -> u8 {
        self.memory[self.translate_address(address)]
    }

    pub fn read16(&self, address: u16) -> u16 {
        // I will assume that we will never attempt to read 16bit that crosses the
        // border of two ranges i.e the range that (address) occupies is not the
        // same as (address+1). Otherwise I would have to do
        // translate_address(address) AND translate_address(address+1)

        let real_address = self.translate_address(address);

        self.memory[real_address] as u16 | ((self.memory[real_address + 1] as u16) << 8)
    }

    pub fn write8(&mut self, address: u16, value: u8) {
        self.memory[self.translate_address(address)] = value;

        self.handle_ppu_registers(address, value);
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

    fn handle_ppu_registers(&mut self, address: u16, value: u8) {
        match address {
            0x2000 => self.ppu.set_ppuctrl(value),
            0x2001 => self.ppu.set_ppumask(value),
            0x2003 => self.ppu.set_oamaddr(value),
            0x2004 => self.ppu.set_oamdata(value),
            0x2005 => self.ppu.set_ppuscroll(value),
            0x2006 => self.ppu.set_ppuaddr(value),
            0x2007 => self.ppu.ppu_data_write(value),
            0x4014 => println!("OAM DMA!"),
            _ => ()
        }
    }
}