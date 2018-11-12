extern crate bmp;

mod cpu;
mod mapper;
mod mbc;
mod ppu;
mod apu;
pub mod joypad;
pub mod rom;

use std::cell::RefCell;
use std::rc::Rc;
use nes::cpu::Cpu;
use nes::mbc::Mbc;
use nes::joypad::Joypad;
use nes::ppu::Ppu;
use nes::apu::Apu;
use nes::mapper::Mapper;
use nes::bmp::Image;

pub struct Nes {
    cpu: Cpu,
    mbc: Rc<RefCell<Box<Mbc>>>,
    ppu: Rc<RefCell<Box<Ppu>>>,
    apu: Rc<RefCell<Box<Apu>>>,
    joypad: Rc<RefCell<Box<Joypad>>>,
    // tick: u32,
}

macro_rules !wrap_rc {
    ($value: expr) => {
        Rc::new(RefCell::new(Box::new($value)));
    }
}

impl Nes {
    pub fn new() -> Self {
        let mapper = wrap_rc!(Mapper::new());
        let ppu = wrap_rc!(Ppu::new(mapper.clone()));
        let apu = wrap_rc!(Apu::new());
        let joypad = wrap_rc!(Joypad::new());
        let mbc = wrap_rc!(Mbc::new(mapper.clone(), ppu.clone(), apu.clone(), joypad.clone()));

        ppu.borrow_mut().set_mbc(Rc::downgrade(&mbc));
        let cpu = Cpu::new(mbc.clone());

        Nes {
            cpu: cpu,
            mbc: mbc,
            ppu: ppu,
            apu: apu,
            joypad: joypad,
        }
    }

    pub fn run(&mut self) {
        loop {
            self.tick();
        }
    }

    pub fn dump(&self) {
        self.mbc.borrow_mut().dump_ram();
        self.ppu.borrow_mut().dump();
    }

    pub fn screen_rendered(&self) -> bool {
        self.ppu.borrow_mut().screen_rendered()
    }

    pub fn reset_screen_rendered(&self) -> bool {
        self.ppu.borrow_mut().screen_rendered()
    }

    #[inline(never)]
    pub fn tick(&mut self) {
        let cpu_cycle = self.cpu.cycle();
        let ppu_cycle = { self.ppu.borrow_mut().cycle() };

        if cpu_cycle * 3 > ppu_cycle {
            self.ppu.borrow_mut().tick();
        } else {
            self.cpu.tick();
        }
    }

    pub fn set_rom(&mut self, rom: Box<rom::Rom>) {
        self.mbc.borrow_mut().set_rom(rom);
        self.cpu.setup();
        self.ppu.borrow_mut().setup();
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
    }

    pub fn screen_size(&self) -> (u32, u32) {
        (ppu::SCREEN_WIDTH as u32, ppu::SCREEN_HEIGHT as u32)
    }

    pub fn render_image(&self, img: &mut Vec<u8>) {
        self.ppu.borrow().render_image(img)
    }

    pub fn set_joypad_button_state(&self, state: u8) {
        self.joypad.borrow_mut().set_button_state(state);
    }

    pub fn render_sound_buffer(&self, data: &mut [u8]) {
        self.apu.borrow_mut().render_sound_buffer(data);
        // // 22050
        // let hz = 440.0;
        // let sample_freq = 22050 as f32;
        // let byte_per_period = sample_freq / self.hz;
        // let position = 0;

        // for dst in data.iter_mut() {
        //     let current = (self.position as f32 * 6.28 / byte_per_period).sin();
        //     *dst = (current * 128.0 + 128.0) as u8;
        //     info!("dst = {:x}", *dst);
        //     self.position += 1;
        //     self.position %= byte_per_period as u32;
        // }
    }
}
