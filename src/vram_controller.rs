pub struct VRAMController {
    memory: [u8; 0x4000],
    oam: [u8; 0xFF]
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
        let bytes_to_copy = 0xFF - oamaddr;
        for i in 0..bytes_to_copy {
            self.oam[oamaddr + i] = data[i];
        }
    }
}