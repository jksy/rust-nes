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

bitflags! {
    struct Control: u8 {
        const ENABLE_NMI = 0x80; // VBlank時にNMIを発生
        const MASTER_SLAVE = 0x40; // always true
        const SPRITE_SIZE_16 = 0x20; // 0:8x8, 1:8x16
        const BG_ADDRESS = 0x10; // 0:$0000, 1:$1000
        const SPRITE_ADDRESS = 0x08; // 0:$0000, 1:$1000
        const ADDR_INCREMENT_32 = 0x04; // 0: +=1 1: +=32
        const NAME_TABLE_ADDR = 0x03; // 00:$2000, 01:$2400, 10:$2800, 11:$2C00
    }
}

bitflags! {
    struct Mask: u8 {
        const GRA = 0x01u8;
        const SHOW_BACKGROUND_LEFTMOST = 0x02u8;
        const SHOW_SPRITE_LEFTMOST = 0x04u8;
        const SHOW_BACKGROUND = 0x08u8;
        const SHOW_SPRITE = 0x10u8;
        const EMP_RED = 0x20u8;
        const EMP_GREEN = 0x40u8;
        const EMP_BLUE = 0x80u8;
    }
}

bitflags! {
    struct Status: u8 {
        const SPRITE_OVERFLOW = 0x20u8; // sprite over flow
        const SPRITE_ZERO_HIT = 0x40u8; // sprite zero hit
        const VBLANK = 0x80u8;
    }
}

pub struct Ppu {
    // PPU register
    control: Control,         // $2000(w)
    mask: Mask,               // $2001(w)
    status: Status,           // $2002(r)
    oam_address: u8,          // $2003(w)
    scroll_position: Vec<u8>, //  $2005(w*2)
    // 0x0000-0x0FFF:Pattern table1(mapped by chr rom)
    // 0x1000-0x1FFF:Pattern table2(mapped by chr rom)
    // 0x2000-0x23FF:Name table1
    // 0x2400-0x27FF:Name table2
    // 0x2800-0x2BFF:Name table3
    // 0x2C00-0x2FFF:Name table4
    // 0x3F00-0x3F1F:Pallete
    vram: Vram,

    oam_ram: Vec<u8>, // for sprites
    mbc: Weak<RefCell<Box<Mbc>>>,
    mapper: Rc<RefCell<Box<Mapper>>>,
    cycle: u64,
    current_line: i16,
    current_cycle: i16,
    is_raise_nmi: bool, // true:when raise interruput
    is_display_changed: bool,
    is_horizontal: bool, // horizontal scroll

    output_frame: Vec<u8>,

    tasks: Vec<Box<OamDmaTask>>,
}

const PALETTE_COLORS: [[u8; 3]; 64] = [
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

const PALETTE_BASE_ADDR: u16 = 0x3F00;
const PALETTE_SPRITE_ADDR: u16 = 0x3F10;

#[allow(dead_code)]
const SCANLINE: i32 = 261;
#[allow(dead_code)]
const CYCLE_PER_LINE: i32 = 341;

const SCREEN_WIDTH: i32 = 256;
const SCREEN_HIGHT: i32 = 240;

const RAISE_NMI_LINE: i16 = SCREEN_HIGHT as i16 + 1;
const DROP_NMI_LINE: i16 = 260;
const RAISE_VBLANK_LINE: i16 = SCREEN_HIGHT as i16 + 1;
const DROP_VBLANK_LINE: i16 = 260;

impl Ppu {
    pub fn new(mapper: Rc<RefCell<Box<Mapper>>>) -> Self {
        let horizontal = { mapper.borrow().is_horizontal() };

        Ppu {
            control: Control::empty(),
            mask: Mask::empty(),
            status: Status::empty(),
            oam_address: 0u8,
            vram: Vram::new(horizontal),
            oam_ram: vec![0x00u8; 0x0100],
            cycle: 0u64,
            current_line: -1,
            current_cycle: 0,
            scroll_position: vec![0, 0],
            is_raise_nmi: false,
            is_display_changed: false,
            is_horizontal: horizontal,

            output_frame: vec![0; (SCREEN_WIDTH * SCREEN_HIGHT) as usize],
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
        // TODO: direct read from rom
        let mapper = self.mapper.borrow();
        let chr_rom = mapper.chr_rom();
        for i in 0..chr_rom.len() {
            self.vram.write(i as u16, chr_rom[i]);
        }
    }

    pub fn cycle(&self) -> u64 {
        self.cycle
    }

    #[inline(never)]
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
        for y in 0..SCREEN_HIGHT {
            for x in 0..SCREEN_WIDTH {
                let palette_index = self.output_frame[(x + y * SCREEN_WIDTH) as usize];
                let color = PALETTE_COLORS[palette_index as usize];
                let pixel = Pixel::new(color[0], color[1], color[2]);
                img.set_pixel(x as u32,
                              y as u32,
                              pixel);
            }
        }
    }

    // TODO:no copy
    fn read_vram_range(&mut self, start: u16, end: u16) -> Vec<u8> {
        let size = (end - start) as usize;
        let mut v = Vec::with_capacity(size);
        unsafe {
            v.set_len(size);
        }
        info!("v:{:?}, v.len:{:?}", v, v.len());
        self.vram.read_internal_range(start..end, &mut v);
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

    #[inline(never)]
    fn process_cycle(&mut self) {
        info!(
            "ppu ({:x}({}),{:x}({}))",
            self.current_cycle, self.current_cycle, self.current_line, self.current_line
        );

        if self.current_cycle == 1 {
            if self.current_line == RAISE_VBLANK_LINE {
                self.status.insert(Status::VBLANK); // on vblank flag
            }
            if self.current_line == DROP_VBLANK_LINE {
                self.status.remove(Status::VBLANK); // clear vblank flag
            }
            if self.current_line == RAISE_NMI_LINE {
                self.is_raise_nmi = true;
            }
            if self.current_line == DROP_NMI_LINE {
                self.is_raise_nmi = false;
            }
        }

        if -1 < self.current_line && self.current_line < SCREEN_HIGHT as i16 && self.current_cycle < SCREEN_WIDTH as i16 {
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
                self.current_line = -1;
            }
        }
    }

    fn bg_pattern_addr(&self) -> u16 {
        if self.control.contains(Control::BG_ADDRESS) {
            0x1000u16
        } else {
            0x0000u16
        }
    }

    fn sprite_pattern_addr(&self) -> u16 {
        if self.control.contains(Control::SPRITE_ADDRESS) {
            0x1000u16
        } else {
            0x0000u16
        }
    }

    fn sprite_size_16(&self) -> bool {
        self.control.contains(Control::SPRITE_SIZE_16)
    }

    fn name_table_addr(&self) -> u16 {
        match (self.control & Control::NAME_TABLE_ADDR).bits {
            0 => 0x2000u16,
            1 => 0x2400u16,
            2 => 0x2800u16,
            3 => 0x2C00u16,
            _ => unreachable!(),
        }
    }

    fn name_table_addr_from_point(&self, x: u16, y: u16) -> u16 {
        let base = self.name_table_addr();
        base + (x / 8) + (y / 8) * 32
    }

    fn attribute_from_point(&mut self, x: u16, y: u16) -> u8 {
        let base = self.name_table_addr();
        let index_x = x / 32;
        let index_y = (y / 32) * 8;
        let index = index_x + index_y;

        let addr = base + index + 0x03C0;
        info!("attr addr:{:x}", addr);
        let attr = self.vram.read_internal(addr);
        attr
    }

    #[inline(never)]
    fn process_pixel(&mut self) {
        let x = self.current_cycle as u16 + self.scroll_position[0] as u16;
        let y = self.current_line as u16 + self.scroll_position[1] as u16;

        info!("process_pixel {},{}, ctrl:{:x}", x, y, self.control);
        // render BG
        let nametable_addr = self.name_table_addr_from_point(x, y);
        let attribute = Attribute::new(self.attribute_from_point(x, y));
        let pattern_index = self.vram.read_internal(nametable_addr);
        let bg_pattern_addr = self.bg_pattern_addr();

        info!(
            "nametable_addr:{:04x}, attr:{:?}, pat_index:{:04x}",
            nametable_addr, attribute, pattern_index
        );

        self.render_pattern_pixel(
            bg_pattern_addr,
            pattern_index,
            x,
            y,
            x,
            y,
            &attribute,
            false,
        );

        // render sprite
        let sprite_pattern_base = self.sprite_pattern_addr();
        self.status.remove(Status::SPRITE_ZERO_HIT); // clear sprite zero hit
        for sprite_index in 0..64 {
            let sprite_y = self.oam_ram[sprite_index * 4] as u16;
            let sprite_x = self.oam_ram[sprite_index * 4 + 3] as u16;
            if y < sprite_y || sprite_y + 8 < y {
                continue;
            }
            if x < sprite_x || sprite_x + 8 < x {
                continue;
            }
            if (x - sprite_x) == 0 {
                continue;
            }

            let pattern_index = self.oam_ram[sprite_index * 4 + 1];
            let attr = Attribute::new(self.oam_ram[sprite_index * 4 + 2]);

            // TODO:replace optimal palette addr
            self.render_pattern_pixel(
                sprite_pattern_base,
                pattern_index,
                x - sprite_x,
                y - sprite_y,
                x,
                y,
                &attr,
                true,
            );
            if sprite_index == 0 {
                self.status.insert(Status::SPRITE_ZERO_HIT); // set sprite zero hit
            }
        }
    }

    #[inline(never)]
    fn render_pattern_pixel(
        &mut self,
        pattern_base: u16,
        pattern_index: u8,
        mut pattern_x: u16,
        mut pattern_y: u16,
        x: u16,
        y: u16,
        attribute: &Attribute,
        is_sprite: bool,
    ) {
        let pattern_addr = (pattern_index as u16) * 2 * 8 + pattern_base;
        let memory = self.read_vram_range(pattern_addr, pattern_addr + 16);
        let pattern = Pattern::new(&memory);

        if is_sprite && attribute.is_frip_horizontally() {
            pattern_x = pattern.width() - (pattern_x & 0x07);
        }
        if is_sprite && attribute.is_frip_vertically() {
            pattern_y = pattern.height() - (pattern_y & 0x07);
        }

        let color_index = pattern.color_index((pattern_x & 0x07) as u8, (pattern_y & 0x07) as u8);

        if is_sprite && color_index == 0 {
            return;
        }

        info!(
            "pattern:addr:0x{:04x}, pattern_index:{:02x}",
            pattern_addr, pattern_index
        );

        let tile_color = attribute.table_color(pattern_x, pattern_y) | color_index;
        let palette_addr = if is_sprite {
            PALETTE_SPRITE_ADDR + tile_color as u16
        } else {
            PALETTE_BASE_ADDR + tile_color as u16
        };
        let palette_index = self.vram.read_internal(palette_addr) & 0x3f;
        self.put_pixel(palette_index, x, y);
    }

    #[inline(always)]
    fn put_pixel(&mut self, palette_index: u8, x: u16, y: u16) {
        self.output_frame[(x + y * SCREEN_WIDTH as u16) as usize] = palette_index;
    }

    fn nametable_increment_value(&self) -> u16 {
        if self.control.contains(Control::ADDR_INCREMENT_32) {
            32
        } else {
            1
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        info!("PPU read:{:04x}", addr);
        match addr {
            0x2002 => {
                // PPU_STATUS
                self.status.bits()
            }
            0x2004 => {
                // OAM_DATA
                self.oam_ram[self.oam_address as usize]
            }
            0x2007 => {
                // PPU_DATA
                let address = self.vram.get_addr();
                let result = self.vram.read(address);
                let inc = self.nametable_increment_value();
                self.vram.increment_addr(inc);
                result
            }
            _ => panic!("PPU read error:#{:x}", addr),
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x2000 => {
                // PPU_CTRL
                self.control = Control::from_bits_truncate(data);
                self.is_display_changed = true
            }
            0x2001 => {
                // PPU_MASK
                self.mask = Mask::from_bits_truncate(data);
                self.is_display_changed = true
            }
            0x2003 => {
                // OAM_ADDRESS
                self.oam_address = data;
                info!("PPU OAM write addr : 0x{:02x}", data);
            }
            0x2004 => {
                // OAM_DATA
                info!("PPU write oam_ram[{:02x}] = {:02x}", self.oam_address, data);
                self.oam_ram[self.oam_address as usize] = data;
                self.oam_address = self.oam_address.wrapping_add(1);
            }
            0x2005 => {
                // PPU_SCROLL
                self.scroll_position.insert(0, data);
                self.scroll_position.truncate(2);
                info!(
                    "PPU scroll position:{:x},{:x}",
                    self.scroll_position[0], self.scroll_position[1],
                );
                self.is_display_changed = true;
            }
            0x2006 => {
                // PPU_ADDRESS
                self.vram.set_addr(data);
            }
            0x2007 => {
                // PPU_DATA
                let mut address = self.vram.get_addr();
                self.vram.write(address, data);
                let inc = self.nametable_increment_value();
                self.vram.increment_addr(inc);
                self.is_display_changed = true;
            }
            0x4014 => {
                // OAM_DMA
                let source = (data as u16) << 8;
                // push task, because cant borrow mbc here
                self.tasks.push(Box::new(OamDmaTask::new(source)));
            }
            _ => panic!("PPU write error:#{:x},#{:x}", addr, data),
        }
    }

    pub fn is_enable_nmi(&self) -> bool {
        self.control.contains(Control::ENABLE_NMI)
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

#[derive(Debug)]
struct Pattern<'a> {
    low: &'a [u8],
    high: &'a [u8],
}

impl<'a> Pattern<'a> {
    fn new(data: &'a [u8]) -> Self {
        Pattern {
            low: &data[0..8],
            high: &data[8..16],
        }
    }

    pub fn color_index(&self, x: u8, y: u8) -> u8 {
        let low = self.low[y as usize] << x & 0x80;
        let high = self.high[y as usize] << x & 0x80;
        low >> 7 | high >> 6
    }

    pub fn width(&self) -> u16 {
        // TODO: 8x16 sprite
        8
    }

    pub fn height(&self) -> u16 {
        // TODO: 8x16 sprite
        8
    }
}

#[derive(Debug)]
struct Attribute {
    attribute: u8,
}

impl Attribute {
    fn new(attr: u8) -> Self {
        Attribute { attribute: attr }
    }

    #[inline(always)]
    pub fn table_color(&self, pattern_x: u16, pattern_y: u16) -> u8 {
        let attr_table_color = match (pattern_x & 0x03 < 2, pattern_y & 0x03 < 2) {
            (true, true) => self.attribute & 0x03,
            (false, true) => (self.attribute >> 2) & 0x03,
            (true, false) => (self.attribute >> 4) & 0x03,
            (false, false) => (self.attribute >> 6) & 0x03,
        };
        attr_table_color << 2
    }

    pub fn is_frip_horizontally(&self) -> bool {
        (self.attribute & 0x40) != 0
    }

    pub fn is_frip_vertically(&self) -> bool {
        (self.attribute & 0x80) != 0
    }
}

// == TASK ==
struct OamDmaTask {
    source: u16,
}

impl OamDmaTask {
    fn new(source: u16) -> Self {
        OamDmaTask {
            source: source,
        }
    }

    fn call(&self, ppu: &mut Ppu) {
        info!(
            "PPU write oam(DMA)[:] = ({:02x}:{:02x})",
            self.source,
            self.source + 0x0100u16
        );
        // TODO:bulk copy
        for i in 0..0x0100u16 {
            let s = (self.source + i) as u16;
            let mbc = ppu.mbc.upgrade().unwrap();
            let v = mbc.borrow().read(s);
            ppu.oam_ram[i as usize] = v;
            info!(
                "oam_ram[0x{:04x}] = mapper.read(0x{:04x}) = {:02x}",
                i, s, v
            );
        }
        ppu.is_display_changed = true;
    }
}
