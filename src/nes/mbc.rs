use nes::rom::Rom;
use nes::ppu::Ppu;
use std::mem;

#[derive(Clone)]
pub struct Mbc {
    rom: Box<Rom>,
    ppu: Box<Ppu>,
    // vrom: &u8,
    ram: Box<[u8]>,
    // sram: &u8,
    // vram: &u8,
}

impl Mbc {
    pub fn new(rom: Box<Rom>) -> Self {
        Mbc{rom: rom,
            ppu: Box::new(Ppu::new()),
            ram: Box::new([0u8; 0x2000])}
    }

    pub fn set_rom(&mut self, rom: Box<Rom>){
        self.rom = rom
    }

    pub fn read(&self, addr: &mut u16) -> u8 {
        print!("Mbc::read({:x})", *addr);
        let x = match *addr {
            0x0000u16...0x1FFFu16 => self.ram[*addr as usize],
            0x2000u16...0x2007u16 => self.ppu.read(addr),
            // 0x4000u16...0x5FFFu16 => self.io[],
            0x6000u16...0x7FFFu16 => { // self.sram[],
                0x00u8
            },
            0x8000u16...0xFFFFu16 => {
                let mut r = *addr & 0x7FFFu16;
                self.rom.read(&r)},
            _ => panic!("mbc read error:#{:x}", *addr)

        };
        println!("-> {:x}", x);
        *addr += 1;
        x as u8
        // self.rom.unwrap()[addr]
    }

    pub fn read16(&self, addr: &mut u16) -> u16 {
        (self.read(addr) as u16 | (self.read(addr) as u16) << 8) as u16
    }

    pub fn vector(&self, name: &str) -> u16 {
        let mut addr = match name {
            "nmi"   => {0xFFFAu16}
            "reset" => {0xFFFCu16}
            "irq"   => {0xFFFEu16}
            _       => {panic!("invalid vector name:{}", name)}
        };
        self.read16(&mut addr)
    }

    pub fn prg_len(&self) -> u16 {
        self.rom.prg_len()
    }

    pub fn write(&mut self, addr: &u16, value: &u8) {
        println!("Mbc::write({:x},{:x})", *addr, *value);
        match *addr {
            0x0000u16...0x1FFFu16 => {
                let prev = self.ram[*(addr) as usize];
                println!("({:x} -> {:x})", prev, *value);
                self.ram[(*addr) as usize] = *value
            },
            0x2000u16...0x2007u16 => {
                self.ppu.write(addr, value)
            },
            // 0x2000u16...0x3FFFu16 => self.io[],
            // 0x4000u16...0x5FFFu16 => self.io[],
            // 0x6000u16...0x7FFFu16 => self.sram[],
            0x8000u16...0xFFFFu16 => panic!("cant write to ROM:{:x}", *addr),
             _ => panic!("mbc write error:#{}", *addr)
        };
    }
}
