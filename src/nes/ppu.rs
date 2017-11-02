use std::cell::RefCell;
use std::rc::Weak;
use nes::mapper::Mapper;
use nes::mbc::Mbc;
use nes::bmp::Image;
use nes::bmp::Pixel;
use std::fs::File;
use std::io::prelude::*;
use std::rc::Rc;

pub struct Ppu {
    // PPU register
    control: u8,     // $2000(w)
    mask: u8,        // $2001(w)
    status: u8,      // $2002(r)
    oam_address: u8, // $2003(w)
    scroll: u8,      // $2005(w*2)
    vram_addr: u8,   // $2006(w*2)
    vram: Vec<u8>,   // 0x0000-0x0FFF:Pattern table1(mapped by chr rom)
                     // 0x1000-0x1FFF:Pattern table2(mapped by chr rom)
                     // 0x2000-0x23FF:Name table1
                     // 0x2400-0x27FF:Name table2
                     // 0x2800-0x2BFF:Name table3
                     // 0x2C00-0x2FFF:Name table4
                     // 0x3F00-0x3F1F:Pallete
    oam_ram: Vec<u8>, // for sprites
    mbc: Weak<RefCell<Box<Mbc>>>,
    tick: u64,
    current_line: u16,
    current_cycle: u16,
    vram_write_addr: Vec<u8>,
    scroll_position: Vec<u8>,
    is_raise_nmi:    bool, // true:when raise interruput
    is_display_changed: bool,

    tasks: Vec<Box<OamDmaTask>>,
}

const PALETTES: [[u8;3]; 64] = [
    [0x7Cu8, 0x7Cu8, 0x7Cu8],
    [0x00u8, 0x00u8, 0xFCu8],
    [0x00u8, 0x00u8, 0xBCu8],
    [0x44u8, 0x28u8, 0xBCu8],
    [0x94u8, 0x00u8, 0x84u8],
    [0xA8u8, 0x00u8, 0x20u8],
    [0xA8u8, 0x10u8, 0x00u8],
    [0x88u8, 0x14u8, 0x00u8],
    [0x50u8, 0x30u8, 0x00u8],
    [0x00u8, 0x78u8, 0x00u8],
    [0x00u8, 0x68u8, 0x00u8],
    [0x00u8, 0x58u8, 0x00u8],
    [0x00u8, 0x40u8, 0x58u8],
    [0x00u8, 0x00u8, 0x00u8],
    [0x00u8, 0x00u8, 0x00u8],
    [0x00u8, 0x00u8, 0x00u8],
    [0xBCu8, 0xBCu8, 0xBCu8],
    [0x00u8, 0x78u8, 0xF8u8],
    [0x00u8, 0x58u8, 0xF8u8],
    [0x68u8, 0x44u8, 0xFCu8],
    [0xD8u8, 0x00u8, 0xCCu8],
    [0xE4u8, 0x00u8, 0x58u8],
    [0xF8u8, 0x38u8, 0x00u8],
    [0xE4u8, 0x5Cu8, 0x10u8],
    [0xACu8, 0x7Cu8, 0x00u8],
    [0x00u8, 0xB8u8, 0x00u8],
    [0x00u8, 0xA8u8, 0x00u8],
    [0x00u8, 0xA8u8, 0x44u8],
    [0x00u8, 0x88u8, 0x88u8],
    [0x00u8, 0x00u8, 0x00u8],
    [0x00u8, 0x00u8, 0x00u8],
    [0x00u8, 0x00u8, 0x00u8],
    [0xF8u8, 0xF8u8, 0xF8u8],
    [0x3Cu8, 0xBCu8, 0xFCu8],
    [0x68u8, 0x88u8, 0xFCu8],
    [0x98u8, 0x78u8, 0xF8u8],
    [0xF8u8, 0x78u8, 0xF8u8],
    [0xF8u8, 0x58u8, 0x98u8],
    [0xF8u8, 0x78u8, 0x58u8],
    [0xFCu8, 0xA0u8, 0x44u8],
    [0xF8u8, 0xB8u8, 0x00u8],
    [0xB8u8, 0xF8u8, 0x18u8],
    [0x58u8, 0xD8u8, 0x54u8],
    [0x58u8, 0xF8u8, 0x98u8],
    [0x00u8, 0xE8u8, 0xD8u8],
    [0x78u8, 0x78u8, 0x78u8],
    [0x00u8, 0x00u8, 0x00u8],
    [0x00u8, 0x00u8, 0x00u8],
    [0xFCu8, 0xFCu8, 0xFCu8],
    [0xA4u8, 0xE4u8, 0xFCu8],
    [0xB8u8, 0xB8u8, 0xF8u8],
    [0xD8u8, 0xB8u8, 0xF8u8],
    [0xF8u8, 0xB8u8, 0xF8u8],
    [0xF8u8, 0xA4u8, 0xC0u8],
    [0xF0u8, 0xD0u8, 0xB0u8],
    [0xFCu8, 0xE0u8, 0xA8u8],
    [0xF8u8, 0xD8u8, 0x78u8],
    [0xD8u8, 0xF8u8, 0x78u8],
    [0xB8u8, 0xF8u8, 0xB8u8],
    [0xB8u8, 0xF8u8, 0xD8u8],
    [0x00u8, 0xFCu8, 0xFCu8],
    [0xF8u8, 0xD8u8, 0xF8u8],
    [0x00u8, 0x00u8, 0x00u8],
    [0x00u8, 0x00u8, 0x00u8],
];

#[allow(dead_code)] const CONTROL_MASK_ENABLE_NMI      :u8 = 0x80;  // VBlank時にNMIを発生
#[allow(dead_code)] const CONTROL_MASK_MASTER_SLAVE    :u8 = 0x40;  // always true
#[allow(dead_code)] const CONTROL_MASK_SPRITE_SIZE     :u8 = 0x20;  // 0:$0000, 1:$1000
#[allow(dead_code)] const CONTROL_MASK_BG_ADDRESS      :u8 = 0x10;  // 0:$0000, 1:$1000
#[allow(dead_code)] const CONTROL_MASK_SPRITE_ADDRESS  :u8 = 0x08;  // 0:$0000, 1:$1000
#[allow(dead_code)] const CONTROL_MASK_ADDR_INCREMENT  :u8 = 0x04;  // 0: +=1 1: +=32
#[allow(dead_code)] const CONTROL_MASK_NAME_TABLE_ADDR :u8 = 0x03;  // 00:$2000, 01:$2400, 10:$2800, 11:$2C00

#[allow(dead_code)] const SCANLINE: i32 = 261;
#[allow(dead_code)] const CYCLE_PER_LINE: i32 = 341;

#[allow(dead_code)] const STATUS_OVERFLOW : u8 = 0x20u8; // sprite over flow
#[allow(dead_code)] const STATUS_SPRITE   : u8 = 0x40u8; // sprite zero hit
#[allow(dead_code)] const STATUS_VBLANK   : u8 = 0x80u8;

#[allow(dead_code)] const MASK_GRAY                     :u8 = 0x01u8;
#[allow(dead_code)] const MASK_SHOW_BACKGROUND_LEFTMOST :u8 = 0x02u8;
#[allow(dead_code)] const MASK_SHOW_SPRITE_LEFTMOST     :u8 = 0x04u8;
#[allow(dead_code)] const MASK_SHOW_BACKGROUND          :u8 = 0x08u8;
#[allow(dead_code)] const MASK_SHOW_SPRITE              :u8 = 0x10u8;
#[allow(dead_code)] const MASK_EMP_RED                  :u8 = 0x20u8;
#[allow(dead_code)] const MASK_EMP_GREEN                :u8 = 0x40u8;
#[allow(dead_code)] const MASK_EMP_BLUE                 :u8 = 0x80u8;

impl Ppu {
    pub fn new() -> Self {
        Ppu{
            control:       0u8,
            mask:          0u8,
            status:        0u8,
            oam_address:   0u8,
            scroll:        0u8,
            vram_addr:     0u8,
            mbc:           Weak::default(),
            vram:          vec![0x00u8; 0x4000],
            oam_ram:       vec![0x00u8; 0x0100],
            tick:          0u64,
            current_line:  0,
            current_cycle: 0,
            vram_write_addr: vec![0,0],
            scroll_position: vec![0,0],
            is_raise_nmi   : false,
            is_display_changed   : false,
            tasks   : vec![],
        }
    }

    pub fn set_mbc(&mut self, mbc: Weak<RefCell<Box<Mbc>>>) {
        self.mbc = mbc;
    }

    pub fn tick(&mut self) {
        self.tick = self.tick.overflowing_add(1).0;
        if self.tasks.len() > 0 {
            let task = self.tasks.pop().unwrap();
            task.call(self);
        }
        self.process_cycle();
    }

    pub fn render_image(&self, img: &mut Image) {
        self.render_bg(img);
        self.render_sprites(img);
    }

    fn render_bg(&self, img: &mut Image) {
        let name_table_addr = self.name_table_addr();
        for y in 0..30 {
            for x in 0..32 {
                let address = name_table_addr + x + y * 32;
                let pattern_index = self.vram[address as usize] as u16;
                self.render_pattern(img, x * 8, y * 8, pattern_index, 0x3f00);
            }
        }
    }

    fn render_sprites(&self, img: &mut Image) {
        for sprite_index in 0..64 {
            let y             = self.oam_ram[sprite_index * 4]     as u16;
            let pattern_index = self.oam_ram[sprite_index * 4 + 1] as u16;
            let attr          = self.oam_ram[sprite_index * 4 + 2] as u16;
            let x             = self.oam_ram[sprite_index * 4 + 3] as u16;

            self.render_pattern(img, x, y, pattern_index, 0x3F10);
        }
    }

    fn with_mapper<F>(&self, func: F)
        where F: FnOnce(Rc<RefCell<Box<Mapper>>>){
        let mbc = self.mbc.upgrade().unwrap();
        let mbc = mbc.borrow_mut();
        let mapper = mbc.mapper().clone();
        func(mapper);
    }

    fn render_pattern(&self,
                      img: &mut Image,
                      base_x: u16,
                      base_y: u16,
                      pattern_index: u16,
                      palette_addr: u16) {
        let head_addr = (self.pattern_addr() + pattern_index * 2 * 8) as usize;
        let tail_addr = (head_addr + 16) as usize;
        self.with_mapper(|mapper: Rc<RefCell<Box<Mapper>>>| {
            let mapper = mapper.borrow();
            let memory = &mapper.chr_rom()[head_addr..tail_addr];
            let pattern = Pattern::new(&memory);

            // draw pattern
            for pix_x in 0..8 {
                for pix_y in 0..8 {
                    let x = (base_x + pix_x) as u32;
                    let y = (base_y + pix_y) as u32;
                    if x == 0 || y == 0 ||
                       img.get_width() <= x ||
                       img.get_height() <= y {
                        continue;
                    }
                    let index = pattern.pal_index(pix_x as u8, pix_y as u8) as usize;
                    // color pallete address
                    // BG1 0x3F00-0x3F03
                    // BG2 0x3F04-0x3F07
                    // BG3 0x3F08-0x3F0B
                    // BG4 0x3F0C-0x3F0F
                    // OBJ1 0x3F10-0x3F13
                    // OBJ2 0x3F14-0x3F17
                    // OBJ3 0x3F18-0x3F1B
                    // OBJ4 0x3F1C-0x3F1F
                    let pal_index = self.vram[palette_addr as usize + index] as usize;
                    let color = PALETTES[pal_index];
                    let pixel = Pixel::new(color[0], color[1], color[2]);
                    img.set_pixel(x as u32, y as u32, pixel);
                }
            }
        });
    }

    pub fn dump(&self) {
        let mut file = File::create("vram.dmp").unwrap();
        let _ = file.write_all(&self.vram).unwrap();
        let mut file = File::create("oam_ram.dmp").unwrap();
        let _ = file.write_all(&self.oam_ram).unwrap();
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

        // reset OAM address
        if 257 <= self.current_line && self.current_line <= 320 {
            self.oam_address = 0;
        }

        self.current_cycle += 1;
        if self.current_cycle == 261 {
            self.current_cycle = 0;
            self.current_line += 1;
            if self.current_line == 341 {
                self.current_line = 0;
            }
        }

        // if 0 < self.current_line && self.current_line < 8 {
        //     if 0 < self.current_cycle && self.current_cycle < 8 {
        //         self.mask |= STATUS_SPRITE;
        //     }
        // } else {
        //     self.mask &= !STATUS_SPRITE;
        // }

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

    fn bg_addr(&self) -> u16 {
        if (self.control & CONTROL_MASK_BG_ADDRESS) != 0 {
            0x1000u16
        } else {
            0x0000u16
        }
    }

    fn pattern_addr(&self) -> u16 {
        if (self.control & CONTROL_MASK_SPRITE_ADDRESS) != 0 {
            0x1000u16
        } else {
            0x0000u16
        }
    }

    fn nametable_increment_value(&self) -> u16 {
        match self.control & CONTROL_MASK_ADDR_INCREMENT {
            0 => 1,
            _ => 32,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        info!("PPU read:{:04x}", addr);
        match addr {
            0x2002 => { // PPU_STATUS
                self.status
            },
            0x2003 => { // OAM_ADDDRESS
                self.oam_address
            },
            0x2004 => { // OAM_DATA
                self.oam_ram[self.oam_address as usize]
            },
            0x2007 => { // PPU_DATA
                self.vram[self.vram_addr as usize]
            },
            _ => panic!("PPU read error:#{:x}", addr)
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x2000 => { // PPU_CTRL
                self.control = data;
                self.vram_write_addr.clear();
                self.vram_write_addr.insert(0, 0);
                self.vram_write_addr.insert(0, 0);
                self.is_display_changed = true
            },
            0x2001 => { // PPU_MASK
                self.mask = data;
                self.is_display_changed = true
            },
            0x2003 => { // OAM_ADDRESS
                self.oam_address = data;
                info!("PPU OAM write addr : 0x{:02x}", data);
            },
            0x2004 => { // OAM_DATA
                info!("PPU write oam_ram[{:02x}] = {:02x}",
                      self.oam_address,
                      data);
                self.oam_ram[self.oam_address as usize] = data;
                self.oam_address += 1
            },
            0x2005 => { // PPU_SCROLL
                self.scroll_position.insert(0, data);
                self.scroll_position.truncate(2);
                info!("PPU scroll position:{:x},{:x}",
                         self.scroll_position[0],
                         self.scroll_position[1],
                         );
                self.is_display_changed = true;
            },
            0x2006 => { // PPU_ADDRESS
                self.vram_write_addr.insert(0, data);
                self.vram_write_addr.truncate(2);
                info!("PPU VRAM write addr : 0x{:02x}{:02x}",
                         self.vram_write_addr[1],
                         self.vram_write_addr[0],
                         );
            },
            0x2007 => { // PPU_DATA
                let mut address = self.vram_write_addr[0] as u16;
                address |= (self.vram_write_addr[1] as u16) << 8;
                info!("PPU write vram[{:x}] = {:x}", address, data);
                self.vram[address as usize] = data;

                address += self.nametable_increment_value();
                self.vram_write_addr[0] = (address & 0xFF) as u8;
                self.vram_write_addr[1] = (address >> 8) as u8;
                self.is_display_changed = true;
            },
            0x4014 => { // OAM_DMA
                let source = (data as u16) << 8;
                let target = self.oam_address as u16;
                // push task, because cant borrow mbc here
                self.tasks.push(Box::new(OamDmaTask::new(source, target)));
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

    pub fn is_display_changed(&self) -> bool {
        self.is_display_changed
    }

    pub fn clear_display_changed(&mut self) {
        self.is_display_changed = false
    }
}

struct Pattern<'a> {
    low:  &'a [u8],
    high: &'a [u8],
}

impl<'a> Pattern<'a> {
    fn new(data: &'a [u8]) -> Self {
        Pattern{low: &data[0..8], high: &data[8..16]}
    }

    pub fn pal_index(&self, x: u8, y: u8) -> u8 {
        let low = self.low[y as usize] << x & 0x80;
        let high = self.high[y as usize] << x & 0x80;
        low >> 7 | high >> 6
    }
}

struct OamDmaTask {
    source: u16,
    target: u16,
}

impl OamDmaTask {
    fn new(source: u16, target: u16) -> Self {
        OamDmaTask{
            source: source,
            target: target,
        }
    }

    fn call(&self, ppu: &mut Ppu) {
        info!("PPU write oam(DMA)[{:02x}:{:02x}] = ({:02x}:{:02x})",
              self.target,
              self.target + 0x0100u16,
              self.source,
              self.source + 0x0100u16);
        // TODO:bulk copy
        for i in 0..0x100u16 {
            let s = (self.source + i) as u16;
            let t = (self.target + i) as usize;
            let mbc = ppu.mbc.upgrade().unwrap();
            let v = mbc.borrow().read(s);
            ppu.oam_ram[t] = v;
            info!("oam_ram[0x{:04x}] = mapper.read(0x{:04x}) = {:02x}",
            t,
            s,
            v);
        }
        ppu.is_display_changed = true;
    }
}

