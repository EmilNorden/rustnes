use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::error::Error;

pub(crate) struct Cartridge {
    prg_rom: Vec<PrgRomBank>
}

pub(crate) struct PrgRomBank {
    data: [u8; 0x4000]
}

impl Cartridge {
    pub(crate) fn load(path: &str) -> Cartridge {
        let mut file = match File::open(path) {
            Err(why) => panic!("Couldn't open rom file: {}", why),
            Ok(file) => file
        };

        let mut header = [0u8; 16];
        file.read_exact(&mut header).unwrap();

        //let s = String::from_raw_parts(header.as_mut_ptr(), 3, 3);
        let s = std::str::from_utf8(&header[0..3]).unwrap();

        if !s.eq("NES") {
            panic!("Invalid header in ROM");
        }

        let prg_rom_bank_count = header[4];
        let chr_rom_size = header[5] as i32 * 0x2000;
        let flags6 = header[6];

        let has_trainer_mask = 0b00000100;

        if (flags6 & has_trainer_mask) == has_trainer_mask {
            // There are 512 bytes of trainer data before the prg rom, so we skip past it for now
            file.seek(SeekFrom::Current(512));
        }

        let mut prg_rom_banks = Vec::new();
        for i in 0..prg_rom_bank_count {
            let mut buffer = [0u8; 0x4000];
            file.read_exact(&mut buffer).unwrap();

            prg_rom_banks.push(PrgRomBank::new(buffer));
        }

        Cartridge {
            prg_rom: prg_rom_banks
        }
    }
}

impl PrgRomBank {
    pub(crate) fn new(data: [u8; 0x4000]) -> PrgRomBank {
        PrgRomBank {
            data
        }
    }

    pub(crate) fn get_data(&self) -> &[u8; 0x4000] {
        &self.data
    }
}