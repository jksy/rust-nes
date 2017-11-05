use std::cell::RefCell;
use std::rc::Weak;
use nes::mapper::Mapper;
use nes::mbc::Mbc;
use nes::bmp::Image;
use nes::bmp::Pixel;
use std::fs::File;
use std::io::prelude::*;
use std::rc::Rc;
use nes::rom::Rom;

pub struct Ppu {
    // PPU register
    control: u8,     // $2000(w)
    mask: u8,        // $2001(w)
    status: u8,      // $2002(r)
    oam_address: u8, // $2003(w)
    scroll_position: Vec<u8>, //  $2005(w*2)
    vram_write_addr: Vec<u8>, // $2006(w*2)
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
    pub fn new(mapper: Rc<RefCell<Box<Mapper>>>) -> Self {
        let horizontal = {
            mapper.borrow().is_horizontal()
        };

        Ppu{
            control:       0u8,
            mask:          0u8,
            status:        0u8,
            oam_address:   0u8,
            vram:         Vram::new(horizontal),
            oam_ram:       vec![0x00u8; 0x0100],
            cycle:          0u64,
            current_line:  0,
            current_cycle: 0,
            vram_write_addr: vec![0,0],
            scroll_position: vec![0,0],
            is_raise_nmi   : false,
            is_display_changed   : false,
            is_horizontal  : horizontal,

            raw_bmp:       Image::new(512, 480),
            mbc:           Weak::default(),
            mapper:        mapper,
            tasks   : vec![],
        }
    }

    pub fn set_mbc(&mut self, mbc: Weak<RefCell<Box<Mbc>>>) {
        self.mbc = mbc;
    }

    pub fn setup(&mut self) {
        // copy chr from rom
        // TODO: directry read from rom
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
    fn read_vram_range(&self, start: u16, end: u16) -> Vec<u8> {
        let mut v = vec![];
        for i in start..end {
            v.push(self.vram.read_no_log(i));
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
        if self.current_cycle == 256 {
            self.current_cycle = 0;
            self.current_line += 1;
            if self.current_line == 341 {
                self.current_line = 0;
            }
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

    fn attriute_from_point(&self, x: u16, y: u16) -> u16 {
        let base = self.name_table_addr();
        let index_x = x / 64;
        let index_y = y / 4;
        let index = index_x + index_y;

        let addr = base + index + 0x03C0;
        let attr = self.vram.read(addr);
        println!("attribute addr = {:x}, {:x}", addr, attr);

        let mut shift = (x / 8) % 2;
        shift += ((y / 8) % 2) * 2;

        ((attr >> shift) & 0x03) as u16
    }

    fn process_pixel(&mut self) {
        println!("current_line,current_cycle = {:}, {:}",
                 self.current_line,
                 self.current_cycle);
        // let x = self.current_cycle;
        // let y = self.current_line;

        let x = self.current_cycle + self.scroll_position[0] as u16;
        let y = self.current_line + self.scroll_position[1] as u16;
        // render BG
        let addr = self.name_table_addr_from_point(x, y);
        let attribute = self.attriute_from_point(x, y);
        let pattern_index = self.vram.read(addr);
        self.render_pattern_pixel(pattern_index,
                                  x, y,
                                  x, y,
                                  attribute,
                                  0x3F00); // TODO:replace optimal palette addr

        // render sprite
        self.status &= !STATUS_SPRITE; // clear sprite zero hit
        for sprite_index in 0..64 {
            let sprite_y      = self.oam_ram[sprite_index * 4]     as u16;
            let pattern_index = self.oam_ram[sprite_index * 4 + 1];
            let attr          = self.oam_ram[sprite_index * 4 + 2] as u16;
            let sprite_x      = self.oam_ram[sprite_index * 4 + 3] as u16;
            if y < sprite_y || sprite_y + 8 < y {
                continue;
            }
            if x < sprite_x || sprite_x + 8 < x {
                continue;
            }

            // TODO:replace optimal palette addr
            self.render_pattern_pixel(pattern_index,
                                      x - sprite_x, y - sprite_y,
                                      x, y,
                                      attr,
                                      0x3F10);
            if sprite_index == 0 {
                self.status |= STATUS_SPRITE; // set sprite zero hit
            }
        }
    }

    fn render_pattern_pixel(&mut self,
                            pattern_index: u8,
                            pattern_x: u16,
                            pattern_y: u16,
                            x: u16,
                            y: u16,
                            attribute: u16,
                            palette_addr: u16) {
        let pattern_addr = (pattern_index as u16) * 2 * 8 + self.pattern_addr();
        let memory = self.read_vram_range(pattern_addr, pattern_addr+16);
        let pattern = Pattern::new(&memory);

        let index = pattern.pal_index((pattern_x & 0x07) as u8,
                                      (pattern_y & 0x07) as u8);

        let pal_index = self.vram.read(palette_addr + index as u16) as usize +
                        (attribute * 4) as usize;
        // let pal_index = self.vram.read(palette_addr + index as u16) as usize;

        let color = PALETTES[pal_index];
        println!("{}, {}, {:02x},{:02x},{:02x}",
                 pal_index,
                 attribute,
                 color[0],
                 color[1],
                 color[2]);
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

    pub fn read(&self, addr: u16) -> u8 {
        info!("PPU read:{:04x}", addr);
        match addr {
            0x2002 => { // PPU_STATUS
                self.status
            },
            0x2004 => { // OAM_DATA
                self.oam_ram[self.oam_address as usize]
            },
            0x2007 => { // PPU_DATA
                let mut address = self.vram_write_addr[0] as u16;
                address |= (self.vram_write_addr[1] as u16) << 8;
                self.vram.read(address)
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
                self.vram.write(address, data);

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


// == VRAM ==

struct NameTable {
    ram: Vec<u8>,
}

struct PatternTable {
    ram: Vec<u8>,
    is_writable: bool,
}

struct PaletteTable {
    ram: Vec<u8>,
}

struct Vram {
    pattern_tables: Vec<PatternTable>,
    name_tables:    Vec<Rc<RefCell<Box<NameTable>>>>,
    palette_tables: Vec<Rc<RefCell<Box<PaletteTable>>>>,
}

impl NameTable {
    fn new() -> Self {
        NameTable{ram: vec![0x0u8; 0x0400]}
    }

    fn read(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    fn write(&mut self, addr: u16, data: u8) {
        info!("{:04x},{:02x}", addr, data);
        self.ram[addr as usize] = data
    }
}

impl PatternTable {
    fn new() -> Self {
        PatternTable{ram: vec![0x0u8; 0x1000], is_writable: true}
    }

    fn read(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    // fn copy_from(data: Vec<u8>) {
    // }

    fn write(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data
    }
}

impl PaletteTable {
    fn new() -> Self {
        PaletteTable{ram: vec![0x0u8; 0x004]}
    }

    fn read(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data
    }
}

impl Vram {
    fn new(is_horizontal: bool) -> Self {
        let mut name_tables = Vec::new();

        if is_horizontal {
            let table0 = Rc::new(RefCell::new(Box::new(NameTable::new())));
            let table2 = Rc::new(RefCell::new(Box::new(NameTable::new())));
            name_tables.push(table0.clone());
            name_tables.push(table0); // mirror of name table 0
            name_tables.push(table2.clone());
            name_tables.push(table2); // mirror of name table 2
        } else {
            let table0 = Rc::new(RefCell::new(Box::new(NameTable::new())));
            let table1 = Rc::new(RefCell::new(Box::new(NameTable::new())));
            name_tables.push(table0.clone());
            name_tables.push(table1.clone());
            name_tables.push(table0); // mirror of name table 0
            name_tables.push(table1); // mirror of name table 1
        }

        let mut pattern_tables = Vec::new();
        for _ in 0..2 {
            pattern_tables.push(PatternTable::new());
        }

        let mut palette_tables = Vec::new();
        for _ in 0..8 {
            palette_tables.push(Rc::new(RefCell::new(Box::new(PaletteTable::new()))));
        }
        // mirror of palette 3F00~3F1F (3F20)-(3FFF)
        for _ in 0..8 {
            for i in 0..8 {
                let p = palette_tables[i].clone();
                palette_tables.push(p);
            }
        }

        Vram {
            pattern_tables: pattern_tables,
            name_tables:    name_tables,
            palette_tables: palette_tables,
        }
    }

    fn read_no_log(&self, addr: u16) -> u8 {
        match addr {
            0x0000...0x1FFF => {
                let (index, target_addr) = Vram::calclate_patterntable_addr(addr);
                self.pattern_tables[index].read(target_addr)
            },
            0x2000...0x3EFF => {
                let (index, target_addr) = Vram::calclate_nametable_addr(addr);
                self.name_tables[index].borrow().read(target_addr)
            },
            0x3F00...0x3FFF => {
                let (index, target_addr) = Vram::calclate_palettetable_addr(addr);
                self.palette_tables[index].borrow().read(target_addr)
            },
            _ => {
                panic!("cant read PPU:0x{:04x}", addr);
            }
        }
    }

    fn read(&self, addr: u16) -> u8 {
        info!("Vram::read({:04x})", addr);
        self.read_no_log(addr)
    }

    fn write(&mut self, addr: u16, data: u8) {
        info!("Vram::write({:04x}, {:02x})", addr, data);
        match addr {
            0x0000...0x1FFF => {
                let (index, target_addr) = Vram::calclate_patterntable_addr(addr);
                self.pattern_tables[index].write(target_addr, data)
            },
            0x2000...0x3E00 => {
                let (index, target_addr) = Vram::calclate_nametable_addr(addr);
                self.name_tables[index].borrow_mut().write(target_addr, data)
            },
            0x3F00...0x3FFF => {
                let (index, target_addr) = Vram::calclate_palettetable_addr(addr);
                self.palette_tables[index].borrow_mut().write(target_addr, data)
            },
            _ => {
                panic!("cant write PPU:0x{:04x} = {:02x}", addr, data);
            }
        }
    }

    fn calclate_nametable_addr(addr : u16) -> (usize, u16) {
        let index       = (addr - 0x2000) / 0x0400;
        let target_addr = (addr - 0x2000) % 0x0400;

        (index as usize, target_addr)
    }

    fn calclate_patterntable_addr(addr : u16) -> (usize, u16) {
        let index       = addr / 0x1000;
        let target_addr = addr % 0x1000;

        (index as usize, target_addr)
    }

    fn calclate_palettetable_addr(addr : u16) -> (usize, u16) {
        let index       = (addr - 0x3F00) / 0x0004;
        let target_addr = (addr - 0x3F00) % 0x0004;

        (index as usize, target_addr)
    }
}



