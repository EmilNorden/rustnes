use crate::vram_controller::VRAMController;
use crate::ppu_registers::PPURegisters;

#[derive(PartialEq)]
enum FetchState {
    NameTable,
    AttributeTable,
    PatternTableTileLow,
    PatternTableTileHigh,
}

pub struct PPU<'a> {
    scanline_cycle: i32,
    scanline: i32,
    cycle_remainders: i32,
    idle_cycle: bool,

    name_table: u8,
    attribute_table: u8,
    pattern_table_tile_low: u8,
    pattern_table_tile_high: u8,
    fetch_state: FetchState,
    vram: &'a mut VRAMController,
    ppu_regs: &'a mut PPURegisters,
}

impl<'a> PPU<'_> {
    pub fn new(vram: &'a mut VRAMController, ppu_regs: &'a mut PPURegisters) -> PPU<'a> {
        PPU {
            scanline_cycle: 0,
            scanline: -1,
            cycle_remainders: 0,
            idle_cycle: true,

            name_table: 0,
            attribute_table: 0,
            pattern_table_tile_low: 0,
            pattern_table_tile_high: 0,
            fetch_state: FetchState::NameTable,
            vram,
            ppu_regs,
        }
    }

    pub fn process(&mut self, ppu_cycles: i32) {
        self.cycle_remainders += ppu_cycles;

        if self.idle_cycle { // First cycle of each scanline is an idle cycle. Just spend it.
            self.cycle_remainders -= 1;
            self.idle_cycle = false;
        }

        if self.fetch_state == FetchState::NameTable && self.spend_cycles(2) {
            self.name_table = 0;
            self.fetch_state = FetchState::AttributeTable;
        }

        if self.fetch_state == FetchState::AttributeTable && self.spend_cycles(2) {
            self.attribute_table = 0;
            self.fetch_state = FetchState::PatternTableTileLow;
        }

        if self.fetch_state == FetchState::PatternTableTileLow && self.spend_cycles(2) {
            self.pattern_table_tile_low = 0;
            self.fetch_state = FetchState::PatternTableTileHigh;
        }

        if self.fetch_state == FetchState::PatternTableTileHigh && self.spend_cycles(2) {
            self.pattern_table_tile_high = 0;
            self.fetch_state = FetchState::NameTable;
        }

        if self.scanline_cycle > 341 {
            self.scanline_cycle -= 341;
            self.scanline += 1;
        }
        
    }

    fn spend_cycles(&mut self, cycles: i32) -> bool {
        if self.cycle_remainders >= cycles {
            self.cycle_remainders -= cycles;
            self.scanline_cycle += cycles;
            return true;
        }

        false
    }

    pub fn ppu_data_write(&mut self, value: u8) {
        self.vram.write8(self.ppu_regs.ppuaddr(), value);
        self.ppu_regs.increment_ppuaddr();
    }

    pub fn set_ppuctrl(&mut self, value: u8) {
        self.ppu_regs.set_ppuctrl(value);
    }

    pub fn set_ppumask(&mut self, value: u8) {
        self.ppu_regs.set_ppumask(value);
    }

    pub fn set_oamaddr(&mut self, value: u8) {
        self.ppu_regs.set_oamaddr(value);
    }

    pub fn set_oamdata(&mut self, value: u8) {
        self.ppu_regs.set_oamdata(value);
    }

    pub fn set_ppuscroll(&mut self, value: u8) {
        self.ppu_regs.set_ppuscroll(value);
    }

    pub fn set_ppuaddr(&mut self, value: u8) {
        self.ppu_regs.set_ppuaddr(value);
    }
}