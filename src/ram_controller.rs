pub struct RamController {
    memory: [u8; 0xFFFF],
}

impl RamController {
    pub fn new() -> RamController {
        RamController {
            memory: [0; 0xFFFF]
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
}