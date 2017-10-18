pub struct Joypad {
}

const BUTTON_A      :u8 = 0x80;
const BUTTON_B      :u8 = 0x40;
const BUTTON_SELECT :u8 = 0x20;
const BUTTON_START  :u8 = 0x10;
const BUTTON_UP     :u8 = 0x08;
const BUTTON_DOWN   :u8 = 0x04;
const BUTTON_LEFT   :u8 = 0x02;
const BUTTON_RIGHT  :u8 = 0x01;

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
