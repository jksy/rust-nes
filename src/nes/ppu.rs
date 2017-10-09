

#[derive(Clone)]
pub struct Ppu {
    // PPU register
    control1: u8,
    control2: u8,
    status: u8,
    sprite_addr: u8,
    sprite_access: u8,
    scroll: u8,
    vram_addr: u8,
    vram_access: u8,
    tick: u8,
}

impl Ppu {
    pub fn new() -> Self {
        Ppu{
            control1:      0u8,
            control2:      0u8,
            status:        0u8,
            sprite_addr:   0u8,
            sprite_access: 0u8,
            scroll:        0u8,
            vram_addr:     0u8,
            vram_access:   0u8,
            tick:          0u8,
        }
    }

    pub fn read(&self, addr: &u16) -> u8 {
        match *addr {
            0x2000 => self.control1,
            0x2001 => self.control2,
            0x2002 => self.status,
            0x2003 => self.sprite_addr,
            0x2004 => self.sprite_access,
            0x2005 => self.scroll,
            0x2006 => self.vram_addr,
            0x2007 => self.vram_access,
            _ => panic!("PPU read error:#{:x}", *addr)
        }
    }

    pub fn write(&mut self, addr: &u16, data: &u8) {
        match *addr {
            0x2000 => self.control1 = *data,
            0x2001 => self.control2 = *data,
            0x2002 => self.status = *data,
            0x2003 => self.sprite_addr = *data,
            0x2004 => self.sprite_access = *data,
            0x2005 => self.scroll = *data,
            0x2006 => self.vram_addr = *data,
            0x2007 => self.vram_access = *data,
            _ => panic!("PPU write error:#{:x},#{:x}", *addr, *data)
        }
    }
}
