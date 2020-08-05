use crate::cartridge::ChrRomBank;

pub struct VRAMController {
    memory: [u8; 0x4000],
    pub oam: [u8; 0xFF]
}

impl VRAMController {
    pub fn new() -> VRAMController {
        VRAMController {
            memory: [0; 0x4000],
            oam: [0; 0xFF]
        }
    }

    pub fn write8(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }

    pub fn read8(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn write_oam_dma(&mut self, oamaddr: u8, data: &[u8]) {
        let oamaddr_usize = oamaddr as usize;
        let bytes_to_copy = 0xFFusize - oamaddr_usize;
        for i in 0usize..bytes_to_copy {
            if data[i] > 0 {
                println!("test");
            }
            self.oam[oamaddr_usize + i] = data[i];
        }
    }

    pub fn write_oam(&mut self, index: u8, value: u8) {
        self.oam[index as usize] = value;
    }

    pub fn nametable0(&self) -> &[u8] {
        &self.memory[0x2000..0x23FF]
    }

    pub(crate) fn load_chr_rom(&mut self, rom: &ChrRomBank) {
        /*
        let prg_bank1 = &mut self.memory[RamController::PRG_BANK1_LOCATION..RamController::PRG_BANK1_LOCATION + RamController::PRG_BANK_SIZE];
        prg_bank1.copy_from_slice(rom.get_data());
        */

        let pattern_tables = &mut self.memory[0..0x2000];
        pattern_tables.copy_from_slice(rom.get_data());
    }
}