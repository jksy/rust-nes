mod vram;

use nes::bmp::Image;
use nes::bmp::Pixel;
use nes::mapper::Mapper;
use nes::mbc::Mbc;
use nes::rom::Rom;
use std::cell::RefCell;
use std::fs::File;
use std::io::prelude::*;
use std::rc::Rc;
use std::rc::Weak;
use std::mem;
use self::vram::Vram;

pub struct Ppu {
    // PPU register
    control: u8,     // $2000(w)
    mask: u8,        // $2001(w)
    status: u8,      // $2002(r)
    oam_address: u8, // $2003(w)
    scroll_position: Vec<u8>, //  $2005(w*2)
    vram: Vram,      // 0x0000-0x0FFF:Pattern table1(mapped by chr rom)
                     // 0x1000-0x1FFF:Pattern table2(mapped by chr rom)
                     // 0x2000-0x23FF:Name table1
                     // 0x2400-0x27FF:Name table2
                     // 0x2800-0x2BFF:Name table3
                     // 0x2C00-0x2FFF:Name table4
                     // 0x3F00-0x3F1F:Pallete
                     //
    oam_ram: Vec<u8>, // for sprites
    mbc: Weak<RefCell<Box<Mbc>>>,
    mapper: Rc<RefCell<Box<Mapper>>>,
    cycle: u64,
    current_line: u16,
    current_cycle: u16,
    is_raise_nmi:    bool, // true:when raise interruput
    is_display_changed: bool,
    is_horizontal: bool, // horizontal scroll

    raw_bmp: Image,

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
#[allow(dead_code)] const CONTROL_MASK_SPRITE_SIZE_16  :u8 = 0x20;  // 0:8x8, 1:8x16
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
    pub fn new(mapper: Rc<RefCell<Box<Mapper>>>) -> Self {
        let horizontal = {
            mapper.borrow().is_horizontal()
        };

        Ppu{
            control:       0u8,
            mask:          0u8,
            status:        0u8,
            oam_address:   0u8,
            vram:          Vram::new(horizontal),
            oam_ram:       vec![0x00u8; 0x0100],
            cycle:         0u64,
            current_line:  0,
            current_cycle: 0,
            scroll_position:    vec![0,0],
            is_raise_nmi:       false,
            is_display_changed: false,
            is_horizontal:      horizontal,

            raw_bmp:  Image::new(512, 480),
            mbc:      Weak::default(),
            mapper:   mapper,
            tasks   : vec![],
        }
    }

    pub fn set_mbc(&mut self, mbc: Weak<RefCell<Box<Mbc>>>) {
        self.mbc = mbc;
    }

    pub fn setup(&mut self) {
        // copy chr from rom
        // TODO: directr read from rom
        let mapper = self.mapper.borrow();
        let chr_rom = mapper.chr_rom();
        for i in 0..chr_rom.len() {
            self.vram.write(i as u16, chr_rom[i]);
        }
    }

    #[inline]
    pub fn cycle(&self) -> u64 {
        self.cycle
    }

    pub fn tick(&mut self) {
        self.cycle = self.cycle.wrapping_add(1);
        info!("ppu cycle:{:}", self.cycle);
        if self.tasks.len() > 0 {
            let task = self.tasks.pop().unwrap();
            task.call(self);
        }
        self.process_cycle();
    }

    pub fn render_image(&self, img: &mut Image) {
        for y in 0..240 {
            for x in 0..256 {
                img.set_pixel(x, y, self.raw_bmp.get_pixel(x, y));
            }
        }
    }

    // TODO:no copy
    fn read_vram_range(&mut self, start: u16, end: u16) -> Vec<u8> {
        let mut v = vec![];
        for i in start..end {
            v.push(self.vram.read_internal(i));
        }
        v
    }

    pub fn dump(&self) {
        // let mut file = File::create("vram.dmp").unwrap();
        // let _ = file.write_all(&self.vram).unwrap();
        // let mut vec = vec![];
        // for i in 0x000..0x4000 {
        //     vec.push(self.vram.read(i));
        // }
        // let mut file = File::create("vram.dmp").unwrap();
        // let _ = file.write_all(&vec).unwrap();

    }

    fn process_cycle(&mut self) {
        info!("ppu ({:x}({}),{:x}({}))",
                 self.current_cycle,
                 self.current_cycle,
                 self.current_line,
                 self.current_line);

        if self.current_cycle == 1 {
            if self.current_line == 241 {
                self.status |= STATUS_VBLANK; // on vblank flag
                self.is_raise_nmi = true;
            }
            if self.current_line == 260 {
                self.is_raise_nmi = false;
            }
            if self.current_line == 261 {
                self.status &= !STATUS_VBLANK; // clear vblank flag
                self.is_raise_nmi = false;
            }
        }

        if self.current_line < 240 && self.current_cycle < 256 {
            self.process_pixel();
        }

        // reset OAM address
        if 257 <= self.current_line && self.current_line <= 320 {
            self.oam_address = 0;
        }

        self.current_cycle += 1;
        if self.current_cycle == 341 {
            self.current_cycle = 0;
            self.current_line += 1;
            if self.current_line == 262 {
                self.current_line = 0;
            }
        }
    }

    fn bg_pattern_addr(&self) -> u16 {
        if (self.control & CONTROL_MASK_BG_ADDRESS) != 0 {
            0x1000u16
        } else {
            0x0000u16
        }
    }

    fn sprite_pattern_addr(&self) -> u16 {
        if (self.control & CONTROL_MASK_SPRITE_ADDRESS) != 0 {
            0x1000u16
        } else {
            0x0000u16
        }
    }

    fn sprite_size_16(&self) -> bool {
        (self.control & CONTROL_MASK_SPRITE_SIZE_16) != 0
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

    fn name_table_addr_from_point(&self, x: u16, y: u16) -> u16 {
        let base = self.name_table_addr();
        base + (x / 8) + (y / 8) * 32
    }

    fn attribute_from_point(&mut self, x: u16, y: u16) -> u16 {
        let base = self.name_table_addr();
        let index_x = x / 32;
        let index_y = (y / 32) * 8;
        let index = index_x + index_y;

        let addr = base + index + 0x03C0;
        info!("attr addr:{:x}", addr);
        let attr = self.vram.read_internal(addr);

        let mut shift = (x / 8) % 2;
        shift += ((y / 8) % 2) * 2;

        ((attr >> shift) & 0x03) as u16
    }

    fn process_pixel(&mut self) {
        let x = self.current_cycle + self.scroll_position[0] as u16;
        let y = self.current_line + self.scroll_position[1] as u16;

        info!("process_pixel {},{}, ctrl:{:x}", x, y, self.control);
        // render BG
        let addr = self.name_table_addr_from_point(x, y);
        let attribute = self.attribute_from_point(x, y);
        let pattern_index = self.vram.read_internal(addr);
        let bg_pattern_addr = self.bg_pattern_addr();
        info!("bg addr:{:04x}, attr:{:x}, pat_index:{:04x}",
              addr,
              attribute,
              pattern_index);
        self.render_pattern_pixel(bg_pattern_addr,
                                  pattern_index,
                                  x, y,
                                  x, y,
                                  attribute,
                                  0x3F00); // TODO:replace optimal palette addr

        // render sprite
        let sprite_pattern_base = self.sprite_pattern_addr();
        self.status &= !STATUS_SPRITE; // clear sprite zero hit
        for sprite_index in 0..64 {
            let sprite_y      = self.oam_ram[sprite_index * 4] as u16;
            let pattern_index = self.oam_ram[sprite_index * 4 + 1];
            let attr          = self.oam_ram[sprite_index * 4 + 2] as u16;
            let sprite_x      = self.oam_ram[sprite_index * 4 + 3] as u16;
            if y < sprite_y || sprite_y + 8 < y {
                continue;
            }
            if x < sprite_x || sprite_x + 8 < x {
                continue;
            }
            if (x - sprite_x) == 0 {
                continue;
            }

            // TODO:replace optimal palette addr
            self.render_pattern_pixel(sprite_pattern_base,
                                      pattern_index,
                                      x - sprite_x - 1, y - sprite_y,
                                      x, y,
                                      attr,
                                      0x3F10);
            if sprite_index == 0 {
                self.status |= STATUS_SPRITE; // set sprite zero hit
            }
        }
    }

    fn render_pattern_pixel(&mut self,
                            pattern_base: u16,
                            pattern_index: u8,
                            pattern_x: u16,
                            pattern_y: u16,
                            x: u16,
                            y: u16,
                            attribute: u16,
                            palette_addr: u16) {
        let pattern_addr = (pattern_index as u16) * 2 * 8 + pattern_base;
        let memory = self.read_vram_range(pattern_addr, pattern_addr+16);
        let pattern = Pattern::new(&memory);

        let index = pattern.pal_index((pattern_x & 0x07) as u8,
                                      (pattern_y & 0x07) as u8);

        info!("pattern:addr:0x{:04x}, pattern_index:{:02x}", pattern_addr, pattern_index);

        let pal_index = self.vram.read_internal(palette_addr + index as u16) as usize +
                        ((attribute & 0x03)* 4) as usize;
        // let pal_index = self.vram.read(palette_addr + index as u16) as usize;
        info!("palette:addr:0x{:04x}, palette_index:{:02x}",
              palette_addr + index as u16,
              pattern_index);

        let color = PALETTES[pal_index];
        let pixel = Pixel::new(color[0], color[1], color[2]);
        self.raw_bmp.set_pixel(x as u32,
                               y as u32,
                               pixel);
    }


    fn nametable_increment_value(&self) -> u16 {
        match self.control & CONTROL_MASK_ADDR_INCREMENT {
            0 => 1,
            _ => 32,
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        info!("PPU read:{:04x}", addr);
        match addr {
            0x2002 => { // PPU_STATUS
                self.status
            },
            0x2004 => { // OAM_DATA
                self.oam_ram[self.oam_address as usize]
            },
            0x2007 => { // PPU_DATA
                let address = self.vram.get_addr();
                let result = self.vram.read(address);
                let inc = self.nametable_increment_value();
                self.vram.increment_addr(inc);
                result
            },
            _ => panic!("PPU read error:#{:x}", addr)
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x2000 => { // PPU_CTRL
                self.control = data;
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
                self.oam_address = self.oam_address.wrapping_add(1);
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
                self.vram.set_addr(data);
            },
            0x2007 => { // PPU_DATA
                let mut address = self.vram.get_addr();
                self.vram.write(address, data);
                let inc = self.nametable_increment_value();
                self.vram.increment_addr(inc);
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

// == TASK ==
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
        for i in 0..0x0100u16 {
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
