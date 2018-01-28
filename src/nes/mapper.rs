use nes::rom::Rom;

pub struct Mapper {
    rom: Box<Rom>,
}

impl Mapper {
    pub fn new() -> Self {
        Mapper { rom: Rom::empty() }
    }

    pub fn set_rom(&mut self, rom: Box<Rom>) {
        self.rom = rom
    }

    pub fn read_prg(&self, addr: u16) -> u8 {
        self.rom.read_prg(addr)
    }

    pub fn chr_rom(&self) -> &[u8] {
        self.rom.chr()
    }

    pub fn is_horizontal(&self) -> bool {
        self.rom.is_horizontal()
    }

    pub fn initial_pc(&self) -> u16 {
        self.rom.initial_pc()
    }
}
