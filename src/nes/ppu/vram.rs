use std::cell::RefCell;
use std::rc::Rc;

const INITIAL_PALETTE_TABLE: [u8; 32] = [
    0x09,0x01,0x00,0x01,0x00,0x02,0x02,0x0D,
    0x08,0x10,0x08,0x24,0x00,0x00,0x04,0x2C,
    0x09,0x01,0x34,0x03,0x00,0x04,0x00,0x14,
    0x08,0x3A,0x00,0x02,0x00,0x20,0x2C,0x08,
];

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

pub struct Vram {
    pattern_tables: Vec<PatternTable>,
    name_tables:    Vec<Rc<RefCell<Box<NameTable>>>>,
    palette_tables: Vec<Rc<RefCell<Box<PaletteTable>>>>,

    vram_write_addr: Vec<u8>,
    read_buffer: u8,
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
    fn new(initial: &[u8]) -> Self {
        PaletteTable{ram: initial.to_vec()}
    }

    fn read(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data
    }
}

impl Vram {
    pub fn new(is_horizontal: bool) -> Self {
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
        for index in 0..8 {
            let head = index * 4;
            let initial_palette = &INITIAL_PALETTE_TABLE[head..(head+4)];
            let table = Rc::new(RefCell::new(Box::new(PaletteTable::new(initial_palette))));
            palette_tables.push(table);
        }
        // mirror of palette 3F00~3F1F (3F20)-(3FFF)
        for _ in 0..8 {
            for i in 0..8 {
                let p = palette_tables[i].clone();
                palette_tables.push(p);
            }
        }

        Vram {
            pattern_tables:  pattern_tables,
            name_tables:     name_tables,
            palette_tables:  palette_tables,
            vram_write_addr: vec![0,0],
            read_buffer:     0x00,
        }
    }

    pub fn read_with_buffer(&mut self, addr: u16) -> u8 {
        let mut result = self.read_buffer;
        match addr {
            0x0000...0x3EFF => {
                self.read_buffer = self.read_internal(addr);
            },
            0x3F00...0x3FFF => {
                // retrun direct value if palette address
                // but buffering vram(addr - 0x1000) value
                self.read_buffer = self.read_internal(addr - 0x1000);
                result = self.read_internal(addr);
            },
            _ => {
                panic!("cant read PPU:0x{:04x}", addr);
            }
        };
        info!("read_with_buffer({:x}) buf:{:x} res:{:x}",
              addr,
              self.read_buffer,
              result);

        result
    }

    pub fn read_internal(&mut self, addr: u16) -> u8 {
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

    pub fn read(&mut self, addr: u16) -> u8 {
        info!("Vram::read({:04x})", addr);
        self.read_with_buffer(addr)
    }

    pub fn get_addr(&self) -> u16 {
        let mut address = self.vram_write_addr[0] as u16;
        address |= (self.vram_write_addr[1] as u16) << 8;
        address
    }

    pub fn set_addr(&mut self, half_addr: u8) {
        self.vram_write_addr.insert(0, half_addr);
        self.vram_write_addr.truncate(2);
        info!("PPU VRAM write addr : 0x{:02x}{:02x}",
                 self.vram_write_addr[1],
                 self.vram_write_addr[0],
                 );
    }

    pub fn clear_addr(&mut self) {
        self.vram_write_addr = vec![0,0];
    }

    pub fn increment_addr(&mut self, value: u16) {
        let address = self.get_addr().wrapping_add(value);
        self.vram_write_addr[0] = (address & 0xFF) as u8;
        self.vram_write_addr[1] = (address >> 8) as u8;
    }

    pub fn write(&mut self, addr: u16, data: u8) {
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

