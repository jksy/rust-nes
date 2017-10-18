use std::cell::RefCell;
use std::rc::Rc;
use nes::mapper::Mapper;
use nes::bmp::Image;

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
    vram: Vec<u8>,   // 0x0000-0x1FFF:Pattern
                     // 0x3F00-0x3F1F:Pallete
    mapper: Rc<RefCell<Box<Mapper>>>,
    tick: u64,
    current_line: u16,
    current_cycle: u16,
    vram_write_addr: Vec<u8>,
    scroll_position: Vec<u8>,
    is_raise_nmi:    bool, // true:when raise interruput
}

const CONTROL_MASK_ENABLE_NMI      :u8 = 0x80;  // VBlank時にNMIを発生
const CONTROL_MASK_MASTER_SLAVE    :u8 = 0x40;  // always true
const CONTROL_MASK_SPRITE_SIZE     :u8 = 0x20;  // 0:$0000, 1:$1000
const CONTROL_MASK_BG_ADDRESS      :u8 = 0x10;  // 0:$0000, 1:$1000
const CONTROL_MASK_SPRITE_ADDRESS  :u8 = 0x08;  // 0:$0000, 1:$1000
const CONTROL_MASK_ADDR_INCREMENT  :u8 = 0x04;  // 0: +=1 1: +=32
const CONTROL_MASK_NAME_TABLE_ADDR :u8 = 0x03;  // 00:$2000, 01:$2400, 10:$2800, 11:$2C00

const SCANLINE: i32 = 261;
const CYCLE_PER_LINE: i32 = 341;

const STATUS_OVERFLOW : u8 = 0x20u8; // sprite over flow
const STATUS_SPRITE   : u8 = 0x40u8; // sprite zero hit
const STATUS_VBLANK   : u8 = 0x80u8;

const MASK_GRAY                     :u8 = 0x01u8;
const MASK_SHOW_BACKGROUND_LEFTMOST :u8 = 0x02u8;
const MASK_SHOW_SPRITE_LEFTMOST     :u8 = 0x04u8;
const MASK_SHOW_BACKGROUND          :u8 = 0x08u8;
const MASK_SHOW_SPRITE              :u8 = 0x10u8;
const MASK_EMP_RED                  :u8 = 0x20u8;
const MASK_EMP_GREEN                :u8 = 0x40u8;
const MASK_EMP_BLUE                 :u8 = 0x80u8;

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
            vram:          vec![0x00u8; 0xFFFF],
            tick:          0u64,
            current_line: 0,
            current_cycle: 0,
            vram_write_addr:          vec![0,0],
            scroll_position:          vec![0,0],
            is_raise_nmi   : false,
        }
    }

    pub fn tick(&mut self) {
        // println!("=====PPU Tick:{}", self.tick);
        // println!("self.current_line = {}, self.current_cycle = {}", self.current_line, self.current_cycle);
        self.tick = self.tick.overflowing_add(1).0;
        self.process_cycle();

        if self.current_line == 0 && self.current_cycle == 0 {
            self.print_bg_name_table();
        }
    }

    fn print_bg_name_table(&self) {
        let addr = self.name_table_addr();
        println!("======== BG NAME TABLE({:04x}) =====", addr);
        for y in 0..30 {
            let start = (addr + y * 32) as usize;
            let end = start + 32;
            let s = String::from_utf8_lossy(&self.vram[start..end]);
            println!("{:02}:{}", y, s);
        }

        // to bmp
        let mut img = Image::new(256, 240);
        let mapper = self.mapper.borrow();
        for y in 0..30 {
            for x in 0..32 {
                let pat_index = self.vram[x + y*32];
                let chr = mapper.chr_rom(pat_index as u16);
            }
        }
    }

    fn process_cycle(&mut self) {
        if self.current_line < 240 {
            self.process_pixel();
        }

        if self.current_cycle == 1 {
            if self.current_line == 241 {
                self.status |= STATUS_VBLANK; // on vblank flag
                self.is_raise_nmi = true;
            }
            if self.current_line == 261 {
                self.status &= !STATUS_VBLANK; // clar vblank flag
                self.is_raise_nmi = false;
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
        // self.name_table_addr()
        // self.current_line;  // y
        // self.current_cycle; // x
    }

    fn name_table_addr(&self) -> u16 {
        match self.control & CONTROL_MASK_NAME_TABLE_ADDR {
            0 => 0x2000u16,
            1 => 0x2400u16,
            2 => 0x2800u16,
            3 => 0x2C00u16,
            _ => {unreachable!()}
        }
    }

    fn nametable_increment_value(&self) -> u16 {
        match self.control & CONTROL_MASK_ADDR_INCREMENT {
            0 => 1,
            _ => 32,
        }
    }

    fn raise_nmi() {
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x2002 => self.status,
            0x2004 => self.oam_data,
            0x2007 => self.vram_data,
            _ => panic!("PPU read error:#{:x}", addr)
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x2000 => {
                self.control = data;
                self.vram_write_addr.clear();
                self.vram_write_addr.insert(0, 0);
                self.vram_write_addr.insert(0, 0);
            },
            0x2001 => self.mask = data,
            // 0x2003 => self.oam_address = data,
            // 0x2004 => self.oam_data = data,
            0x2005 => {
                self.scroll_position.insert(0, data);
                self.scroll_position.truncate(2);
                println!("PPU scroll position:{:x},{:x}",
                         self.scroll_position[0],
                         self.scroll_position[1],
                         );
            },
            0x2006 => {
                self.vram_write_addr.insert(0, data);
                self.vram_write_addr.truncate(2);
                println!("PPU vram write addr:{:x},{:x}",
                         self.vram_write_addr[0],
                         self.vram_write_addr[1],
                         );
            },
            0x2007 => {
                let mut address = self.vram_write_addr[0] as u16;
                address |= (self.vram_write_addr[1] as u16) << 8;
                println!("write PPU:vram[{:x}] = {:x}", address, data);
                self.vram[address as usize] = data;

                address += self.nametable_increment_value();
                self.vram_write_addr[0] = (address & 0xFF) as u8;
                self.vram_write_addr[1] = (address >> 8) as u8;
            },
            _ => panic!("PPU write error:#{:x},#{:x}", addr, data)
        }
    }

    pub fn is_enable_nmi(&self) -> bool {
        (self.control & CONTROL_MASK_ENABLE_NMI) != 0
    }

    pub fn is_raise_nmi(&mut self) -> bool {
        let result = self.is_raise_nmi;
        self.is_raise_nmi = false;
        result
    }
}
