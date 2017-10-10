use nes::rom::Rom;

#[derive(Clone)]
pub struct Mapper {
    rom: Box<Rom>,
}

impl Mapper {
    pub fn new() -> Self {
        Mapper{rom: Rom::empty()}
    }

    pub fn set_rom(&mut self, rom: Box<Rom>) {
        self.rom = rom
    }

    pub fn read(&self, addr: &u16) -> u8 {
        self.rom.read(addr)
    }
}

