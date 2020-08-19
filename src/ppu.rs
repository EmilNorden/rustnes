use crate::vram_controller::VRAMController;
use crate::ppu_registers::PPURegisters;
use std::cell::{RefCell, Cell};

#[derive(PartialEq)]
pub enum PPUResult {
    Ok,
    VBlankNMI
}

#[derive(PartialEq)]
enum FetchState {
    NameTable,
    AttributeTable,
    PatternTableTileLow,
    PatternTableTileHigh,
}

pub struct PPU<'a> {
    frame_cycle: i32,
    scanline_cycle: i32,
    scanline: i32,
    cycle_remainders: i32,
    idle_cycle: bool,
    cycle_deduction: i32,

    name_table: u8,
    attribute_table: u8,
    pattern_table_tile_low: u8,
    pattern_table_tile_high: u8,
    fetch_state: FetchState,
    vram: &'a RefCell<VRAMController>,
    ppu_regs: &'a Cell<PPURegisters>,
}

impl<'a> PPU<'_> {

    // Just for debugging
    pub fn pixel(&self) -> i32 {
        self.scanline_cycle
    }

    pub fn scanline(&self) -> i32 {
        self.scanline
    }


    pub fn new(vram: &'a RefCell<VRAMController>, ppu_regs: &'a Cell<PPURegisters>) -> PPU<'a> {
        PPU {
            frame_cycle: 0,
            scanline_cycle: 0,
            scanline: 0,
            cycle_remainders: 0,
            idle_cycle: true,
            cycle_deduction: 0,

            name_table: 0,
            attribute_table: 0,
            pattern_table_tile_low: 0,
            pattern_table_tile_high: 0,
            fetch_state: FetchState::NameTable,
            vram,
            ppu_regs
        }
    }

    pub fn render_visible_scanlines(&mut self) {
        if self.idle_cycle && self.spend_cycles(1) { // First cycle of each scanline is an idle cycle. Just spend it.
            self.idle_cycle = false;
        }

        if self.scanline_cycle > 0 && self.scanline_cycle < 257 {
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
        }

        if self.scanline_cycle > 256 && self.scanline_cycle < 321 {
            // Garbage nametable byte read
            if self.fetch_state == FetchState::NameTable && self.spend_cycles(2) {
                self.fetch_state = FetchState::AttributeTable;
            }

            // Garbage attribute table byte read
            if self.fetch_state == FetchState::AttributeTable && self.spend_cycles(2) {
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
        }

        if self.scanline_cycle > 320 && self.scanline_cycle < 337 {
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
        }

        if self.scanline_cycle > 336 && self.scanline_cycle < 341 {
            if self.fetch_state == FetchState::NameTable && self.spend_cycles(2) {
                self.fetch_state = FetchState::AttributeTable;
            }

            if self.fetch_state == FetchState::AttributeTable && self.spend_cycles(2) {
                self.fetch_state = FetchState::NameTable; // Reset back to nametable for next scanline
            }
        }
    }

    fn process_internal(&mut self) -> PPUResult {
        let mut result = PPUResult::Ok;

        if self.scanline_cycle > 330 {
            let _foo = 3;
        }

        if self.scanline < 240 || self.scanline == 261 {
            self.render_visible_scanlines()
        }

        if self.scanline >= 240 && self.scanline < 261 {
            let scanline_cycle_before = self.scanline_cycle;

            // Spend as many cycles as possible, but do not exceed 341
            self.spend_cycles(  self.cycle_remainders.min(341 - self.scanline_cycle));

            if self.scanline == 241 && scanline_cycle_before < 1 && self.scanline_cycle >= 1 {
                let mut regs = self.ppu_regs.get();
                regs.set_vblank();
                self.ppu_regs.set(regs);

                if regs.should_generate_nmi() {
                    result = PPUResult::VBlankNMI;
                }
            }

        }

        if self.scanline_cycle == 341 {
            self.scanline_cycle = 0;
            self.scanline += 1;
            self.idle_cycle = true;

            if self.scanline == 262 {
                self.scanline = 0;
                self.frame_cycle = 0;
            }
        }

        result
    }

    pub fn process(&mut self, ppu_cycles: i32) -> PPUResult {
        self.cycle_remainders += ppu_cycles;
        let mut result = PPUResult::Ok;

        loop {
            let unspent_cycles = self.cycle_remainders;
            if self.process_internal() == PPUResult::VBlankNMI {
                result = PPUResult::VBlankNMI;
            }

            if unspent_cycles == self.cycle_remainders {
                break;
            }
        }

        result
    }

    fn spend_cycles(&mut self, mut cycles: i32) -> bool {
        if self.cycle_remainders == 0 {
            return false;
        }

        cycles -= self.cycle_deduction;

        if self.cycle_remainders >= cycles {
            self.cycle_remainders -= cycles;
            self.scanline_cycle += cycles;
            self.frame_cycle += cycles;
            self.cycle_deduction = 0;
            true
        } else {
            self.scanline_cycle += self.cycle_remainders;
            self.frame_cycle += self.cycle_remainders;
            self.cycle_deduction = cycles - self.cycle_remainders;
            self.cycle_remainders = 0;

            false
        }
    }

    /*fn spend_cycles(&mut self, cycles: i32) -> bool {
        if self.cycle_remainders > 0 {
            self.cycle_remainders -= cycles;
            self.scanline_cycle += cycles;
            self.frame_cycle += cycles;
            return true;
        }

        false
    }*/

}