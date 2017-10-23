extern crate timer;
extern crate chrono;
extern crate bmp;

mod cpu;
pub mod rom;
mod mbc;
mod ppu;
mod joypad;
mod mapper;
mod addressing_mode;

use std::cell::RefCell;
use std::rc::Rc;
use nes::cpu::Cpu;
use nes::mbc::Mbc;
use nes::joypad::Joypad;
use nes::ppu::Ppu;
use nes::mapper::Mapper;
use std::sync::mpsc::channel;
use std::thread;
use std::time;

pub struct Nes {
    cpu: Cpu,
    mbc: Rc<RefCell<Box<Mbc>>>,
    ppu: Rc<RefCell<Box<Ppu>>>,
    // mapper: Rc<RefCell<Box<Mapper>>>,
    // tick: u32,
}

macro_rules !wrap_with_rc {
    ($value: expr) => {
        Rc::new(RefCell::new(Box::new($value)));
    }
}

impl Nes {
    pub fn new() -> Self {
        let mapper = wrap_with_rc!(Mapper::new());
        let ppu    = wrap_with_rc!(Ppu::new(mapper.clone()));
        let joypad = wrap_with_rc!(Joypad::new());
        let mbc    = wrap_with_rc!(Mbc::new(mapper.clone(), ppu.clone(), joypad.clone()));
        let cpu = Cpu::new(mbc.clone());

        Nes{
            cpu: cpu,
            mbc: mbc,
            ppu: ppu,
            // mapper: mapper,
        }
    }

    pub fn run(&mut self) {
        let (sender, receiver) = channel();

        println!("==============================");
        thread::spawn(
            move || {
                loop {
                    thread::sleep(time::Duration::new(0, 50));
                    let _ = sender.send(0).unwrap();
                }
            });

        loop {
            let _ = receiver.recv().unwrap();
            self.tick();
        }
    }

    fn tick(&mut self) {
        {
            // println!("ppu.tick()");
            let mut ppu = self.ppu.borrow_mut();
            ppu.tick();
            ppu.tick();
            ppu.tick();
        }
        self.cpu.tick();
    }

    pub fn set_rom(&mut self, rom: Box<rom::Rom>) {
        self.mbc.borrow_mut().set_rom(rom);
        self.cpu.setup();
    }


    pub fn reset(&mut self) {
        self.cpu.reset();
    }
}
