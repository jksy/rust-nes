extern crate timer;
extern crate chrono;

pub mod cpu;
pub mod rom;
pub mod opcode;
pub mod mbc;
pub mod ppu;
pub mod mapper;

use std::cell::RefCell;
use std::rc::Rc;
use nes::cpu::Cpu;
use nes::rom::Rom;
use nes::mbc::Mbc;
use nes::ppu::Ppu;
use nes::mapper::Mapper;
use std::sync::mpsc::channel;
use std::thread;
use std::time;

type RefMapper = Rc<RefCell<Box<Mapper>>>;

pub struct Nes {
    cpu: Cpu,
    mbc: Rc<RefCell<Box<Mbc>>>,
    ppu: Rc<RefCell<Box<Ppu>>>,
    mapper: Rc<RefCell<Box<Mapper>>>,
    // tick: u32,
}

impl Nes {
    pub fn new() -> Self {
        let mut mapper = Rc::new(RefCell::new(Box::new(Mapper::new())));
        let mut ppu = Rc::new(RefCell::new(Box::new(Ppu::new(mapper.clone()))));
        let mut mbc = Rc::new(RefCell::new(Box::new(Mbc::new(mapper.clone(), ppu.clone()))));
        // let mut ref_mbc = Rc::new(RefCell::new(mbc));
        let mut cpu = Cpu::new(mbc.clone());

        Nes{
            cpu: cpu,
            mbc: mbc,
            ppu: ppu,
            mapper: mapper,
        }
    }

    pub fn run(&mut self) {
        let (sender, receiver) = channel();

        println!("==============================");
        thread::spawn(
            move || {
                loop {
                    thread::sleep(time::Duration::new(0, 50_000));
                    let x = sender.send(0).unwrap();
                }
            });

        loop {
            println!("=====recv");
            let r = receiver.recv().unwrap();
            println!("Receiver:{:?}", r);
            self.tick();
        }
    }

    fn tick(&mut self) {
        {
            println!("ppu.tick()");
            let mut ppu = self.ppu.borrow_mut();
            ppu.tick();
            ppu.tick();
            ppu.tick();
        }
        self.cpu.tick();
    }

    pub fn set_rom(&mut self, rom: Box<rom::Rom>) {
        self.mbc.borrow_mut().set_rom(rom);
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
    }
}
