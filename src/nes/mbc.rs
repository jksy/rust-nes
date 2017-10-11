use nes::rom::Rom;
use nes::ppu::Ppu;
use nes::mapper::Mapper;
use std::mem;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct Mbc {
    mapper: Rc<RefCell<Box<Mapper>>>,
    // vrom: &u8,
    ram: Box<[u8]>,
    ppu: Rc<RefCell<Box<Ppu>>>,
    // sram: &u8,
    // vram: &u8,
}

impl Mbc {
    pub fn new(mapper: Rc<RefCell<Box<Mapper>>>, ppu: Rc<RefCell<Box<Ppu>>>) -> Self {
        Mbc{mapper: mapper,
            ppu: ppu,
            ram: Box::new([0u8; 0x2000])}
    }

    pub fn set_rom(&mut self, rom: Box<Rom>){
        self.mapper.borrow_mut().set_rom(rom)
    }

    pub fn read(&self, addr: &mut u16) -> u8 {
        // print!("Mbc::read({:x})", *addr);
        let x = match *addr {
            0x0000u16...0x1FFFu16 => self.ram[*addr as usize],
            0x2000u16...0x2007u16 => self.ppu.borrow().read(addr),
            // 0x4000u16...0x5FFFu16 => self.io[],
            0x6000u16...0x7FFFu16 => { // self.sram[],
                0x00u8
            },
            0x8000u16...0xFFFFu16 => {
                let mut r = *addr & 0x7FFFu16;
                self.mapper.borrow().read(&r)
            },
            _ => panic!("mbc read error:#{:x}", *addr)

        };
        // println!("-> {:x}", x);
        *addr += 1;
        x as u8
    }

    pub fn read16(&self, addr: &mut u16) -> u16 {
        (self.read(addr) as u16 | (self.read(addr) as u16) << 8) as u16
    }


    pub fn write(&mut self, addr: &u16, value: &u8) {
        // println!("Mbc::write({:x},{:x})", *addr, *value);
        match *addr {
            0x0000u16...0x1FFFu16 => {
                let prev = self.ram[*(addr) as usize];
                // println!("({:x} -> {:x})", prev, *value);
                self.ram[(*addr) as usize] = *value
            },
            0x2000u16...0x2007u16 => {
                self.ppu.borrow_mut().write(addr, value)
            },
            // 0x2000u16...0x3FFFu16 => self.io[], // dont use
            0x4000u16...0x401Fu16 => {}, // ignore(APU, etc)
            // 0x4020u16...0x5FFFu16 => self.io[], // extend ram
            // 0x6000u16...0x7FFFu16 => self.sram[],
            0x8000u16...0xFFFFu16 => panic!("cant write to ROM:{:x}", *addr),
             _ => panic!("mbc write error:#{:x}", *addr)
        };
    }
}
