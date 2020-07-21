pub struct VRAMController {
    memory: [u8; 0x4000],
}

impl VRAMController {
    pub fn new() -> VRAMController {
        VRAMController {
            memory: [0; 0x4000],
        }
    }

    pub fn write8(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }
}