use nes::rom::Rom;

pub struct Mbc<'a> {
    rom: Option<&'a Rom>,
    // vrom: &u8,
    ram: [u8; 0x2000],
    // sram: &u8,
    // vram: &u8,
}

impl<'a> Mbc<'a> {
    pub fn new(rom: Option<&'a Rom>) -> Self {
        Mbc{rom: rom, ram: [0u8; 0x2000]}
    }

    pub fn set_rom(&self, rom: &'a Rom){
        self.rom = Some(rom);
    }

    pub fn read(&self, addr: &u16) -> u8 {
        let x = match *addr {
            0x0000u16...0x1FFFu16 => self.ram[*addr as usize],
            // 0x2000u16...0x3FFFu16 => self.io[],
            // 0x4000u16...0x5FFFu16 => self.io[],
            // 0x6000u16...0x7FFFu16 => self.sram[],
             0x8000u16...0xFFFFu16 => self.rom.unwrap().read(addr),
             _ => panic!("mbc read error:#{}", *addr)

        };
        x as u8
        // self.rom.unwrap()[addr]
    }

    pub fn write(&mut self, addr: &u16, value: u8) {
        match *addr {
            0x0000u16...0x1FFFu16 => self.ram[(*addr & 0x7FF) as usize] = value,
            // 0x2000u16...0x3FFFu16 => self.io[],
            // 0x4000u16...0x5FFFu16 => self.io[],
            // 0x6000u16...0x7FFFu16 => self.sram[],
            0x8000u16...0xFFFFu16 => panic!("cant write to ROM:{}", *addr),
             _ => panic!("mbc write error:#{}", *addr)
        };
    }
}
