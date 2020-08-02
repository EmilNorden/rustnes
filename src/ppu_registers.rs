
#[derive(Clone, Copy, Debug)]
pub struct PPURegisters {
    ppuaddr: u16,
    ppuctrl: u8,
    ppumask: u8,
    oamaddr: u8,
    ppuscroll_x: u8,
    ppuscroll_y: u8,
    ppustatus: u8,
    ppuscroll_toggle: bool,
    ppuaddr_toggle: bool
}

impl PPURegisters {
    pub fn new() -> PPURegisters {
        PPURegisters {
            ppuaddr: 0,
            ppuctrl: 0,
            ppumask: 0,
            oamaddr: 0,
            ppuscroll_x: 0,
            ppuscroll_y: 0,
            ppustatus: 0,
            ppuscroll_toggle: false,
            ppuaddr_toggle: false
        }
    }

    pub fn set_ppuctrl(&mut self, value: u8) {
        self.ppuctrl = value;
    }

    pub fn set_ppumask(&mut self, value: u8) {
        self.ppumask = value;
    }

    pub fn set_oamaddr(&mut self, value: u8) {
        self.oamaddr = value;
    }

    pub fn set_ppuscroll(&mut self, value: u8) {
        if self.ppuscroll_toggle {
            self.ppuscroll_y = value;
        }
        else {
            self.ppuscroll_x = value;
        }

        self.ppuscroll_toggle = !self.ppuscroll_toggle;
    }

    pub fn set_ppuaddr(&mut self, value: u8) {
        if self.ppuaddr_toggle {
            self.ppuaddr = (self.ppuaddr & 0xFF00) | value as u16;
        }
        else {
            self.ppuaddr = (self.ppuaddr & 0x00FF) | ((value as u16) << 8);
        }

        self.ppuaddr_toggle = !self.ppuaddr_toggle;
    }

    pub fn oamaddr(&self) -> u8 {
        self.oamaddr
    }

    pub fn ppuaddr(&self) -> u16 {
        self.ppuaddr
    }

    pub fn increment_ppuaddr(&mut self) {
        if (self.ppuctrl & 0x4) == 0x4 {
            self.ppuaddr += 0x20;
        }
        else {
            self.ppuaddr += 1;
        }
    }

    pub fn set_vblank(&mut self) {
        self.ppustatus |= 0b10000000;
    }

    fn clear_vblank(&mut self) {
        self.ppustatus &= !0b10000000;
    }

    pub fn status(&mut self) -> u8 {
        let result = self.ppustatus;
        self.clear_vblank();
        self.ppuscroll_toggle = false;
        self.ppuaddr_toggle = false;

        result
    }

    pub fn should_generate_nmi(&self) -> bool {
        (self.ppuctrl & 0b10000000) == 0b10000000
    }
}