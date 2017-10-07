use nes::rom::Rom;
use std::mem;

#[derive(Clone)]
pub struct Mbc {
    rom: Box<Rom>,
    // vrom: &u8,
    ram: Box<[u8]>,
    // sram: &u8,
    // vram: &u8,
}

impl Mbc {
    pub fn new(rom: Box<Rom>) -> Self {
        Mbc{rom: rom, ram: Box::new([0u8; 0x2000])}
    }

    pub fn set_rom(&mut self, rom: Box<Rom>){
        self.rom = rom
    }

    pub fn read(&self, addr: &u16) -> u8 {
        println!("Mbc::read({:x})", *addr);
        let x = match *addr {
            0x0000u16...0x1FFFu16 => self.ram[*addr as usize],
            // 0x2000u16...0x3FFFu16 => self.io[],
            // 0x4000u16...0x5FFFu16 => self.io[],
            0x6000u16...0x7FFFu16 => { // self.sram[],
                0x00u8
            },
            0x8000u16...0xFFFFu16 => {
                let r = *addr & 0x7FFFu16;
                self.rom.read(&r)},
            _ => panic!("mbc read error:#{:x}", *addr)

        };
        x as u8
        // self.rom.unwrap()[addr]
    }

    pub fn read16(&self, addr: &u16) -> u16 {
        let (low, high) = (*addr, *addr + 1);
        self.read(&low) as u16 | (self.read(&high) as u16) << 8
    }

    pub fn vector(&self, name: &str) -> u16 {
        let addr = match name {
            "nmi"   => {0xFFFAu16}
            "reset" => {0xFFFCu16}
            "irq"   => {0xFFFEu16}
            _       => {panic!("invalid vector name:{}", name)}
        };
        self.read16(&addr)
    }

    pub fn prg_len(&self) -> u16 {
        self.rom.prg_len()
    }

    pub fn write(&mut self, addr: &u16, value: &u8) {
        match *addr {
            0x0000u16...0x1FFFu16 => self.ram[(*addr & 0x7FF) as usize] = *value,
            // 0x2000u16...0x3FFFu16 => self.io[],
            // 0x4000u16...0x5FFFu16 => self.io[],
            // 0x6000u16...0x7FFFu16 => self.sram[],
            0x8000u16...0xFFFFu16 => panic!("cant write to ROM:{}", *addr),
             _ => panic!("mbc write error:#{}", *addr)
        };
    }
}
