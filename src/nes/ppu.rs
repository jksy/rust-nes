#![feature(generators, generator_traint)]

use std::cell::RefCell;
use std::rc::Rc;
use nes::mapper::Mapper;

#[derive(Clone)]
pub struct Ppu {
    // PPU register
    control: u8,     // $2000(w)
    mask: u8,        // $2001(w)
    status: u8,      // $2002(r)
    oam_address: u8, // $2003(w)
    oam_data: u8,    // $2004(r/w)
    scroll: u8,      // $2005(w*2)
    vram_addr: u8,   // $2006(w*2)
    vram_data: u8,   // $2007(r/w)
    oam_dma: u8,     // $4014(w)
    vram: Vec<u8>,
    mapper: Rc<RefCell<Box<Mapper>>>,
    //
    tick: u64,
    current_line: u16,
    current_cycle: u16,
}

const SCANLINE: i32 = 261;
const CYCLE_PER_LINE: i32 = 341;

const STATUS_OVERFLOW: u8 = 0x40u8; // sprite over flow
const STATUS_ZERO: u8     = 0x40u8; // sprite zero hit
const STATUS_VBLANK: u8   = 0x80u8;

impl Ppu {
    pub fn new(mapper: Rc<RefCell<Box<Mapper>>>) -> Self {
        Ppu{
            control:       0u8,
            mask:          0u8,
            status:        0u8,
            oam_address:   0u8,
            oam_data:      0u8,
            scroll:        0u8,
            vram_addr:     0u8,
            vram_data:     0u8,
            oam_dma:       0u8,
            mapper:        mapper,
            vram:          vec![0x00u8; 256*240*3],
            tick:          0u64,
            current_line: 0,
            current_cycle: 0,
        }
    }

    pub fn tick(&mut self) {
        println!("=====PPU Tick:{}", self.tick);
        println!("self.current_line = {}, self.current_cycle = {}", self.current_line, self.current_cycle);
        self.tick = self.tick.overflowing_add(1).0;
        self.process_cycle();
    }

    fn process_cycle(&mut self) {
        if self.current_line < 240 {
            self.process_pixel();
        }

        if self.current_cycle == 1 {
            if self.current_line == 241 {
                self.status |= STATUS_VBLANK; // on vblank flag
            }
            if self.current_line == 261 {
                self.status &= !STATUS_VBLANK; // clar vblank flag
            }
        }

        self.current_cycle += 1;
        if self.current_cycle == 261 {
            self.current_cycle = 0;
            self.current_line += 1;
            if self.current_line == 341 {
                self.current_line = 0;
            }
        }

    }

    fn process_pixel(&mut self) {

    }

    pub fn read(&self, addr: &u16) -> u8 {
        match *addr {
            0x2002 => self.status,
            0x2004 => self.oam_data,
            0x2007 => self.vram_data,
            _ => panic!("PPU read error:#{:x}", *addr)
        }
    }

    pub fn write(&mut self, addr: &u16, data: &u8) {
        match *addr {
            0x2000 => {
                self.control = *data
            },
            0x2001 => self.mask = *data,
            0x2003 => self.oam_address = *data,
            0x2004 => self.oam_data = *data,
            0x2005 => self.scroll = *data,
            0x2006 => self.vram_addr = *data,
            0x2007 => self.vram_data = *data,
            _ => panic!("PPU write error:#{:x},#{:x}", *addr, *data)
        }
    }
}