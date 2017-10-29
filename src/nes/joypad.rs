#[derive(Debug)]
pub struct Joypad {
    register :u8,
    state    :u8,
    counter  :u8,
}

#[allow(dead_code)] pub const BUTTON_A      :u8 = 0x01;
#[allow(dead_code)] pub const BUTTON_B      :u8 = 0x02;
#[allow(dead_code)] pub const BUTTON_SELECT :u8 = 0x04;
#[allow(dead_code)] pub const BUTTON_START  :u8 = 0x08;
#[allow(dead_code)] pub const BUTTON_UP     :u8 = 0x10;
#[allow(dead_code)] pub const BUTTON_DOWN   :u8 = 0x20;
#[allow(dead_code)] pub const BUTTON_LEFT   :u8 = 0x40;
#[allow(dead_code)] pub const BUTTON_RIGHT  :u8 = 0x80;

impl Joypad {
    pub fn new() -> Self {
        Joypad{register: 0u8, state: 0u8, counter: 0u8}
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x4016 => self.read_state(),
            0x4017 => 0x00, // TODO:2 player
            _ => panic!("Joypad read error:#{:x}", addr)
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x4016 => {
                self.register = data;
            },
            0x4017 => {
                // TODO: player 2
            },
            _ => panic!("Joypad write error:#{:x}, {:x}", addr, data)
        }
    }

    pub fn set_button_state(&mut self, state: u8) {
        // info!("set_button_state({:x}) => {:?}", state, self);
        if (self.register & 0x01) == 0x01 {
            self.reset();
            self.state = state;
        }
    }

    pub fn reset(&mut self) {
        self.counter = 0u8;
    }

    pub fn read_state(&mut self) -> u8 {
        // info!("joypad: register:{:x}, state:{:x}, counter:{:x}",
        //          self.register,
        //          self.state,
        //          self.counter);
        let result = (self.state >> self.counter) & 0x01;
        if self.counter < 0x07 {
            self.counter += 1;
        }
        return result
    }
}
