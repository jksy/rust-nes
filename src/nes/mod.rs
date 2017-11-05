extern crate timer;
extern crate chrono;
extern crate bmp;

mod cpu;
pub mod rom;
mod mbc;
mod ppu;
pub mod joypad;
mod mapper;
mod addressing_mode;

use std::cell::RefCell;
use std::rc::Rc;
use nes::cpu::Cpu;
use nes::mbc::Mbc;
use nes::joypad::Joypad;
use nes::ppu::Ppu;
use nes::mapper::Mapper;
use nes::bmp::Image;

pub struct Nes {
    cpu: Cpu,
    mbc: Rc<RefCell<Box<Mbc>>>,
    ppu: Rc<RefCell<Box<Ppu>>>,
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
        let ppu    = wrap_rc!(Ppu::new(mapper.clone()));
        let joypad = wrap_rc!(Joypad::new());
        let mbc    = wrap_rc!(Mbc::new(mapper.clone(), ppu.clone(), joypad.clone()));

        ppu.borrow_mut().set_mbc(Rc::downgrade(&mbc));
        let cpu = Cpu::new(mbc.clone());

        Nes{
            cpu: cpu,
            mbc: mbc,
            ppu: ppu,
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

    pub fn tick(&mut self) {
        // {
        //     // info!("ppu.tick()");
        //     let mut ppu = self.ppu.borrow_mut();
        //     ppu.tick();
        //     ppu.tick();
        //     ppu.tick();
        // }
        // self.cpu.tick();
        let cpu_cycle = self.cpu.cycle();
        let ppu_cycle = {
            self.ppu.borrow_mut().cycle()
        };

        if cpu_cycle > ppu_cycle * 3 {
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

    pub fn render_image(&self, img: &mut Image) {
        self.ppu.borrow().render_image(img)
    }

    pub fn is_display_changed(&self) -> bool {
        self.ppu.borrow().is_display_changed()
    }

    pub fn clear_display_changed(&self) {
        self.ppu.borrow_mut().clear_display_changed()
    }

    pub fn set_joypad_button_state(&self, state: u8) {
        self.joypad.borrow_mut().set_button_state(state);
    }

}
