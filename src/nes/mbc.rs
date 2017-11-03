use nes::rom::Rom;
use nes::ppu::Ppu;
use nes::mapper::Mapper;
use nes::joypad::Joypad;
use std::cell::RefCell;
use std::rc::Rc;
use std::fs::File;
use std::io::prelude::*;

pub struct Mbc {
    mapper: Rc<RefCell<Box<Mapper>>>,
    // vrom: &u8,
    ram: Box<[u8]>,
    ppu: Rc<RefCell<Box<Ppu>>>,
    joypad: Rc<RefCell<Box<Joypad>>>,
    // sram: &u8,
    // vram: &u8,
}

impl Mbc {
    pub fn new(mapper: Rc<RefCell<Box<Mapper>>>,
               ppu: Rc<RefCell<Box<Ppu>>>,
               joypad: Rc<RefCell<Box<Joypad>>>,
               ) -> Self {
        Mbc{mapper: mapper,
            ppu: ppu,
            joypad: joypad,
            ram: Box::new([0u8; 0x2000])}
    }

    pub fn set_rom(&mut self, rom: Box<Rom>){
        self.mapper.borrow_mut().set_rom(rom);
    }

    pub fn initial_pc(&self) -> u16 {
        self.mapper.borrow().initial_pc()
    }

    pub fn read(&self, addr: u16) -> u8 {
        let x = match addr {
            0x0000u16...0x1FFFu16 => self.ram[addr as usize],
            0x2000u16...0x3FFFu16 => self.ppu.borrow().read(addr & 0x2007),
            0x4016u16...0x4017u16 => self.joypad.borrow_mut().read(addr),
            0x6000u16...0x7FFFu16 => { // self.sram[],
                0x00u8
            },
            0x8000u16...0xFFFFu16 => {
                let r = addr & 0x7FFFu16;
                self.mapper.borrow().read_prg(r)
            },
            _ => panic!("mbc read error:#{:x}", addr)

        };
        info!("Mbc::read({:04x}) -> {:x}", addr, x);
        x as u8
    }

    pub fn read16(&self, addr: u16) -> u16 {
        let low = self.read(addr) as u16;
        let high = self.read(addr+1) as u16;
        high << 8 | low
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        info!("  Mbc::write({:x},{:x})", addr, value);
        match addr {
            0x0000u16...0x1FFFu16 => {
                self.ram[addr as usize] = value
            },
            0x2000u16...0x3FFFu16 => {
                self.ppu.borrow_mut().write(addr & 0x2007, value)
            },
            // 0x2000u16...0x3FFFu16 => self.io[], // dont use
            0x4000u16...0x4013u16 => {}, // ignore(APU, etc)
            0x4014u16             => {
                self.ppu.borrow_mut().write(addr, value)
            },
            0x4015u16             => {
                // ignore
            },
            0x4016u16...0x4017u16 => {
                self.joypad.borrow_mut().write(addr,value)
            },
            // 0x4020u16...0x5FFFu16 => self.io[], // extend ram
            // 0x6000u16...0x7FFFu16 => self.sram[],
            0x8000u16...0xFFFFu16 => panic!("cant write to ROM:{:x}", addr),
             _ => panic!("mbc write error:#{:x}", addr)
        };
    }

    pub fn mapper(&self) -> Rc<RefCell<Box<Mapper>>> {
        self.mapper.clone()
    }

    pub fn is_enable_nmi(&self) -> bool {
        self.ppu.borrow().is_enable_nmi()
    }

    pub fn is_raise_nmi(&self) -> bool {
        self.ppu.borrow_mut().is_raise_nmi()
    }

    pub fn dump_ram(&self) {
        let mut file = File::create("ram.dmp").unwrap();
        let _ = file.write_all(&self.ram).unwrap();
    }
}
