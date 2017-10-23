pub struct Joypad {
}

#[allow(dead_code)] const BUTTON_A      :u8 = 0x80;
#[allow(dead_code)] const BUTTON_B      :u8 = 0x40;
#[allow(dead_code)] const BUTTON_SELECT :u8 = 0x20;
#[allow(dead_code)] const BUTTON_START  :u8 = 0x10;
#[allow(dead_code)] const BUTTON_UP     :u8 = 0x08;
#[allow(dead_code)] const BUTTON_DOWN   :u8 = 0x04;
#[allow(dead_code)] const BUTTON_LEFT   :u8 = 0x02;
#[allow(dead_code)] const BUTTON_RIGHT  :u8 = 0x01;

impl Joypad {
    pub fn new() -> Self {
        Joypad{}
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x4016 => 0x00,
            0x4017 => 0x00,
            _ => panic!("Joypad read error:#{:x}", addr)
        }
    }

    pub fn write(&self, addr: u16, data: u8) {
        panic!("Joypad write error:#{:x},{:x}", addr, data)
    }
}
