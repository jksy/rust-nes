use nes::mbc::Mbc;
use nes::opcode::{OpType, AddressMode};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Cpu {
    pub a: u8,      // accumulator
    pub x: u8,      // index register(X)
    pub y: u8,      // index register(Y)
    pub pc: u16,    // program counter
    pub s: u8,      // stack pointer
    pub p: u8,      // processor status register
    pub mbc: Rc<RefCell<Box<Mbc>>>,
    pub step: u32,
    //
}

const FLAG_CRY:u8 = 0x01; // carry flag
const FLAG_ZER:u8 = 0x02; // zero flag
const FLAG_IRQ:u8 = 0x04; // irq flag
const FLAG_DEC:u8 = 0x08; // decimal model flag(dont use on nes)
const FLAG_BRK:u8 = 0x10; // break command flag
const FLAG_RSV:u8 = 0x20; // reserved (always 1)
const FLAG_OVF:u8 = 0x40; // over flow flag
const FLAG_NEG:u8 = 0x80; // negative flag

macro_rules !sub {
    ($self:ident, $target:expr, $value:expr) => {
        {
            let (r, overflow) = ($target).overflowing_sub($value);
            $self.set_flag(FLAG_OVF, overflow);
            $self.set_flag(FLAG_ZER, r == 0);
            $target = r;
        }

    }
}

macro_rules !add {
    ($self:ident, $target:expr, $value:expr) => {
        {
            let (r, overflow) = ($target).overflowing_add($value);
            $self.set_flag(FLAG_OVF, overflow);
            $self.set_flag(FLAG_ZER, r == 0);
            $target = r;
        }

    }
}

macro_rules !cmp {
    ($self:ident, $target:expr, $value:expr) => {
        {
            let target = $target;
            let value = $value as u8;
            $self.set_flag(FLAG_CRY, target < value);
            $self.set_flag(FLAG_ZER, target == value);
        }

    }
}

macro_rules !load {
    ($self:ident, $target:expr, $addr:expr) => {
        {
            $target = $self.read($addr);
            let z = $target == 0;
            $self.set_flag(FLAG_ZER, z);
            let n = $target & FLAG_NEG;
            $self.set_flag(FLAG_NEG, n != 0);
        }
    }
}

macro_rules !branch {
    ($self:ident, $name:expr, $flag:expr, $jump_addr:expr, $result:expr) => {
        {
            if $self.get_flag($flag) == $result {
                println!("{} Jump pc:{:x} -> {:x}", $name, $self.pc, $jump_addr);
                $self.pc = $jump_addr;
                return;
            }
        }
    }
}

trait AddressingMode {
    fn read(&self) -> u8;
    fn write(&self, data u8);
}

struct ImmediateAddressingMode;
impl AddressingMode for ImmediateAddressingMode {
    fn read(&self) -> u8 {
        let offset = cpu.read(pc) as i8 as i16 as u16;
        pc.overflowing_add(offset).0
    };
    fn write(&self, data u8) {
        panic!("cant call ImmediateAddressingMode::write()");
    }
}

struct ZeroPageAddressingMode;
impl AddressingMode for ZeroPageAddressingMode {
    fn read(&self, cpu: &mut Cpu) -> u8 {
        cpu.read(argument)
    };
    fn write(&self, cpu: &mut Cpu, data: u8) {
        cpu.write(argument)
    }
}
struct ZeroPageXAddressingMode;
impl AddressingMode for ZeroPageXAddressingMode {
    fn read(&self, cpu: &mut Cpu) -> u8 {
        cpu.read(argument)
    };
    fn write(&self, cpu: &mut Cpu, data: u8) {
        cpu.write(argument)
    }
}

struct ZeroPageYAddressingMode;
impl AddressingMode for ZeroPageYAddressingMode {
    fn read(&self, cpu: &mut Cpu) -> u8 {
        cpu.read(argument)
    };
    fn write(&self, cpu: &mut Cpu, data: u8) {
        cpu.write(argument)
    }
}

struct RelativeAddressingMode;
impl AddressingMode for RelativeAddressingMode {
    fn read(&self, cpu: &mut Cpu) -> u8 {
        argument + cpu.pc
    };
    fn write(&self, cpu: &mut Cpu, data: u8) {
        panic!("cant call RelativeAddressingMode::write()");
    }
}

struct AbsoluteAddressingMode;
impl AddressingMode for AbsoluteAddressingMode {
    fn read(&self, cpu: &mut Cpu) -> u8 {
        cpu.read(argument)
    };
    fn write(&self, cpu: &mut Cpu, data: u8) {
        panic!("cant call RelativeAddressingMode::write()");
    }
}

struct AbsoluteXAddressingMode;
impl AddressingMode for AbsoluteXAddressingMode {
    fn read(&self, cpu: &mut Cpu) -> u8 {
        cpu.read(argument)
    };
    fn write(&self, cpu: &mut Cpu, data: u8) {

    }
}

struct AbsoluteYAddressingMode;
impl AddressingMode for AbsoluteYAddressingMode {
    fn read(&self, cpu: &mut Cpu) -> u8 {
        cpu.read(argument)
    };
    fn write(&self, cpu: &mut Cpu, data: u8) {

    }
}

struct IndirectAddressingMode;
impl AddressingMode for IndirectAddressingMode {
    fn read(&self, cpu: &mut Cpu) -> u8 {
        cpu.read(argument)
    };
    fn write(&self, cpu: &mut Cpu, data: u8) {

    }
}

struct IndirectXAddressingMode;
impl AddressingMode for IndirectXAddressingMode {
    fn read(&self, cpu: &mut Cpu) -> u8 {
        cpu.read(argument)
    };
    fn write(&self, cpu: &mut Cpu, data: u8) {

    }
}

struct IndirectYAddressingMode;
impl AddressingMode for IndirectYAddressingMode {
    fn read(&self, cpu: &mut Cpu) -> u8 {
        cpu.read(argument)
    };
    fn write(&self, cpu: &mut Cpu, data: u8) {

    }
}

impl Cpu {
    pub fn new(mbc: Rc<RefCell<Box<Mbc>>>) -> Self {
        Cpu {
            a: 0, x: 0, y: 0,
            pc: 0, s: 0xFF, p: 0,
            mbc: mbc,
            step: 0,
            }
    }

    pub fn tick(&mut self) {
        self.step += 1;
        self.debug();

        if self.pc < 0xc000 {
            panic!("== invalidate pc:${:x}", self.pc);
        }
        // let mut counter: u16 = self.pc;
        let mut counter: u16 = self.pc;
        let opcode = self.read(&mut counter);
        let (official, optype, bytes, cycle, addr_mode) = self.decode(opcode);
        let mut addr = self.decode_address(&addr_mode, &mut counter);

        println!("optype:{:?}({:x}), addr:{:x}({:?}), bytes:{:x}", optype, opcode, addr, addr_mode, bytes);

        match optype {
            OpType::BRK => {
                let pc = self.pc;
                self.push16(&pc);
                let p = self.p;
                self.push(&p);
                self.set_flag(FLAG_IRQ, true);
                let mut vector = self.vector("irq");
                self.pc = self.read16(&mut vector);
                return
            },
            OpType::CLC => { self.set_flag(FLAG_CRY, false) },
            OpType::SEC => { self.set_flag(FLAG_CRY, true) },
            OpType::CLI => { self.set_flag(FLAG_IRQ, false) },
            OpType::SEI => { self.set_flag(FLAG_IRQ, true) },
            OpType::CLV => { self.set_flag(FLAG_OVF, false) },
            OpType::CLD => { self.set_flag(FLAG_DEC, false) },
            OpType::SED => { self.set_flag(FLAG_DEC, true) },

            // copy operators
            OpType::LDA => { load!(self, self.a, &mut addr) },
            OpType::LDX => { load!(self, self.x, &mut addr) },
            OpType::LDY => { load!(self, self.y, &mut addr) },

            OpType::STA => { let a = self.a; self.write(&mut addr, &a) },
            OpType::STX => { let x = self.x; self.write(&mut addr, &x) },
            OpType::STY => { let y = self.y; self.write(&mut addr, &y) },

            OpType::TAX => { self.x = self.a; },
            OpType::TAY => { self.y = self.a; },
            OpType::TSX => { self.x = self.s; },
            OpType::TXA => { self.a = self.x; },
            OpType::TXS => { self.s = self.x; },
            OpType::TYA => { self.a = self.y; },

            // caluculate oprators
            OpType::ADC => { self.a = self.a + addr as u8; },
            OpType::AND => { self.a = self.a & addr as u8; },
            OpType::ASL => {
                let carry = (self.a & 0x80) != 0;
                self.set_flag(FLAG_CRY, carry);
                self.a = self.a << 1;
            },

            OpType::BIT => {
                let value = self.read(&mut addr);
                let z = self.a & value;
                self.set_flag(FLAG_ZER, z == 0);
                self.set_flag(FLAG_NEG, (value & FLAG_NEG) == 0);
                // self.set_flag(FLAG_OVF, (value & FLAG_NEG) == 0);
            },
            OpType::CMP => { cmp!(self, self.a, addr) },
            OpType::CPX => { cmp!(self, self.x, addr) },
            OpType::CPY => { cmp!(self, self.y, addr) },
            OpType::DEC => { sub!(self, self.a, 1) },
            OpType::DEX => { sub!(self, self.x, 1) },
            OpType::DEY => { sub!(self, self.y, 1) },
            OpType::EOR => { self.a ^= addr as u8; },
            OpType::INC => {
                let mut data = self.read(&mut addr);
                add!(self, data, 1);
                let mut write_addr = addr - 1;
                self.write(&mut write_addr, &data);
            },
            OpType::INX => { add!(self, self.x, 1) },
            OpType::INY => { add!(self, self.y, 1) },
            OpType::LSR => {
                let carry = (self.a & 0x01) != 0;
                self.set_flag(FLAG_CRY, carry);
                let result = self.a >> 1;
                self.set_flag(FLAG_ZER, result == 0);
                self.set_flag(FLAG_NEG, (result & 0x80) != 0);
                self.a = result
            },
            OpType::ORA => { self.a |= addr as u8; },
            OpType::ROL => {
                let data = self.read(&mut addr);
                let result = (data >> 7) | (data << 1);
                addr = addr - 1;
                self.write(&mut addr, &result);
            },
            OpType::ROR => {
                let data = self.read(&mut addr);
                let result = (data << 7) | (data >> 1);
                addr = addr - 1;
                self.write(&mut addr, &result);
            },
            OpType::SRE => {
                let mut data = self.read(&mut addr);
                self.set_flag(FLAG_CRY, (data & 0x01) != 0);
                data >>= 1;
                addr = addr - 1;
                self.write(&mut addr, &data);
            },
            OpType::SBC => {
                let data = self.read(&mut addr) ^ 0xFF;
                let mut temp = self.a;
                {
                    let (r, _) = temp.overflowing_add(data);
                    temp = r;
                }
                {
                    let (r, _) = temp.overflowing_add(self.s & FLAG_CRY);
                    temp = r;
                }
                self.set_flag(FLAG_CRY, (temp & 0x80) != 0);
                self.set_flag(FLAG_NEG, (temp & 0x80) != 0);
                self.set_flag(FLAG_ZER, temp == 0);
                let a_xor = self.a ^ temp;
                let data_xor = data ^ temp;
                self.set_flag(FLAG_OVF, (a_xor & data_xor & 0x80) != 0);
                self.a = temp;
            },

            // STACK
            OpType::PHA => { let a = self.a; self.push(&a) },
            OpType::PHP => { let p = self.p; self.push(&p) },
            OpType::PLA => { self.a = self.pop() },
            OpType::PLP => { self.p = self.pop() },

            // JMP
            OpType::JMP => { self.pc = addr; return; },
            OpType::JSR => {
                println!("self.pc({:x}) => {:x}", self.pc, addr);
                self.push16(&counter);
                self.pc = addr;
                return;
            },
            OpType::RTS => {
                let return_addr = self.pop16();
                println!("self.pc({:x}) => {:x}", self.pc, return_addr);
                self.pc = return_addr;
                return;
            },
            OpType::RTI => {
                let flag = (self.pop() & FLAG_IRQ) != 0;
                self.set_flag(FLAG_IRQ, flag);
                let return_addr = self.pop16() + 1;
                println!("self.pc({:x}) => {:x}", self.pc, return_addr);
                self.pc = addr;
                return
            }
            // BRANCH
            OpType::BCC => { branch!(self, "BCC", FLAG_CRY, addr, false) },
            OpType::BCS => { branch!(self, "BCS", FLAG_CRY, addr, true ) },
            OpType::BEQ => { branch!(self, "BEQ", FLAG_ZER, addr, true ) },
            OpType::BMI => { branch!(self, "BMI", FLAG_NEG, addr, true ) },
            OpType::BNE => { branch!(self, "BNE", FLAG_ZER, addr, false) },
            OpType::BPL => { branch!(self, "BPL", FLAG_NEG, addr, false) },
            OpType::BVC => { branch!(self, "BVC", FLAG_OVF, addr, false) },
            OpType::BVS => { branch!(self, "BVS", FLAG_OVF, addr, true ) },


            // other
            OpType::NOP => { },
            _ => {panic!("NO operand:{:?}", optype);},
        }

        if counter - self.pc != bytes {
            panic!("PC({:x}) error, {} != {}", self.pc, counter - self.pc, bytes);
        }

        self.pc += bytes
    }

    fn debug(&mut self) {
        let mut addr = 0x02u16;
        let test_result = self.read16(&mut addr);
        println!("=====CPU(step:[{:07}],pc:[{:02x}]====", self.step, self.pc);
        print!("a:{:02x}", self.a);
        print!(" x:{:02x}", self.x);
        print!(" y:{:02x}", self.y);
        println!(" s:{:02x}", self.s);
        println!("p[{:x}] CRY:{}, ZER:{}, IRQ:{}, DEC:{}, BRK:{}, RSV:{}, OVF:{}, NEG:{}",
                 self.p,
                 (self.p & FLAG_CRY) != 0,
                 (self.p & FLAG_ZER) != 0,
                 (self.p & FLAG_IRQ) != 0,
                 (self.p & FLAG_DEC) != 0,
                 (self.p & FLAG_BRK) != 0,
                 (self.p & FLAG_RSV) != 0,
                 (self.p & FLAG_OVF) != 0,
                 (self.p & FLAG_NEG) != 0,
                 );
        println!("test_result:0x{:x}", test_result);
    }

    fn read(&mut self, addr: &mut u16) -> u8 {
        self.mbc.borrow().read(addr)
    }

    fn read16(&mut self, addr: &mut u16) -> u16 {
        self.mbc.borrow().read16(addr)
    }

    fn read16bug(&mut self, addr: &u16) -> u16 {
        println!("read16bug({:x})", *addr);
        let mut a = *addr;
        let mut b = (a & 0xFF00u16) | (((a & 0x00FFu16) + 1) & 0x00FF);
        let low = self.read(&mut a) as u16;
        let high = self.read(&mut b) as u16;
        println!("high:{:x}, low:{:x}", high, low);
        high << 8 | low
    }

    fn write(&self, addr: &mut u16, data: &u8) {
        self.mbc.borrow_mut().write(addr, data)
    }

    fn push(&mut self, data: &u8) {
        let addr = self.s as u16 + 0x0100;
        println!("push(addr => {:x}, data => {:x})", addr, *data);
        self.mbc.borrow_mut().write(&addr, data);
        self.s -= 1;
    }

    fn push16(&mut self, data: &u16) {
        println!("push16(self.s:{:x}, {:x})", self.s, *data);
        let low = (*data & 0xFF) as u8;
        let high = (*data >> 8) as u8;
        self.push(&low);
        self.push(&high);
    }

    fn pop(&mut self) -> u8 {
        self.s += 1;
        let mut addr = (self.s as u16) + 0x0100;
        let data = self.mbc.borrow().read(&mut addr);
        // println!("pop(addr => {:x}, data => {:x})", addr, data);
        data
    }

    fn pop16(&mut self) -> u16 {
        // println!("pop() => low({:x})", low);
        let high = self.pop() as u16;
        let low = self.pop() as u16;
        println!("pop() => high({:x})", high);
        let data = (high << 8) | low;
        println!("pop16(self.s:{:x}, {:x})", self.s, data);
        data
    }

    fn decode(&self, opcode: u8) -> (bool, OpType, u16, u16, AddressMode) {
        let r = match opcode {
            0x00 => (true , OpType::BRK, 1, 0, AddressMode::Implid),
            0x01 => (true , OpType::ORA, 2, 6, AddressMode::IdxInd),
            0x02 => (false, OpType::KIL, 1, 0, AddressMode::Immedt),
            0x03 => (false, OpType::SLO, 2, 8, AddressMode::IdxInd),
            0x04 => (false, OpType::NOP, 2, 3, AddressMode::ZeroPg),
            0x05 => (true , OpType::ORA, 2, 3, AddressMode::ZeroPg),
            0x06 => (true , OpType::ASL, 2, 5, AddressMode::ZeroPg),
            0x07 => (false, OpType::SLO, 2, 5, AddressMode::ZeroPg),
            0x08 => (true , OpType::PHP, 1, 3, AddressMode::Implid),
            0x09 => (true , OpType::ORA, 2, 2, AddressMode::Immedt),
            0x0A => (true , OpType::ASL, 1, 2, AddressMode::Accumu),
            0x0B => (false, OpType::ANC, 2, 2, AddressMode::Immedt),
            0x0C => (false, OpType::NOP, 3, 4, AddressMode::Absolu),
            0x0D => (true , OpType::ORA, 3, 4, AddressMode::Absolu),
            0x0E => (true , OpType::ASL, 3, 6, AddressMode::Absolu),
            0x0F => (false, OpType::SLO, 3, 6, AddressMode::Absolu),
            0x10 => (true , OpType::BPL, 2, 3, AddressMode::Immedt),
            0x11 => (true , OpType::ORA, 2, 5, AddressMode::IndIdx),
            0x12 => (false, OpType::KIL, 1, 0, AddressMode::Immedt),
            0x13 => (false, OpType::SLO, 2, 8, AddressMode::IndIdx),
            0x14 => (false, OpType::NOP, 2, 4, AddressMode::ZPIdxX),
            0x15 => (true , OpType::ORA, 2, 4, AddressMode::ZPIdxX),
            0x16 => (true , OpType::ASL, 2, 6, AddressMode::ZPIdxX),
            0x17 => (false, OpType::SLO, 2, 6, AddressMode::ZPIdxX),
            0x18 => (true , OpType::CLC, 1, 2, AddressMode::Implid),
            0x19 => (true , OpType::ORA, 3, 4, AddressMode::AbIdxY),
            0x1A => (false, OpType::NOP, 1, 2, AddressMode::Immedt),
            0x1B => (false, OpType::SLO, 3, 7, AddressMode::AbIdxY),
            0x1C => (false, OpType::NOP, 3, 4, AddressMode::AbIdxX),
            0x1D => (true , OpType::ORA, 3, 4, AddressMode::AbIdxX),
            0x1E => (true , OpType::ASL, 3, 7, AddressMode::AbIdxX),
            0x1F => (false, OpType::SLO, 3, 7, AddressMode::AbIdxX),
            0x20 => (true , OpType::JSR, 3, 6, AddressMode::Absolu),
            0x21 => (true , OpType::AND, 2, 6, AddressMode::IdxInd),
            0x22 => (false, OpType::KIL, 1, 0, AddressMode::Immedt),
            0x23 => (false, OpType::RLA, 2, 8, AddressMode::IdxInd),
            0x24 => (true , OpType::BIT, 2, 3, AddressMode::ZeroPg),
            0x25 => (true , OpType::AND, 2, 3, AddressMode::ZeroPg),
            0x26 => (true , OpType::ROL, 2, 5, AddressMode::ZeroPg),
            0x27 => (false, OpType::RLA, 2, 5, AddressMode::ZeroPg),
            0x28 => (true , OpType::PLP, 1, 4, AddressMode::Implid),
            0x29 => (true , OpType::AND, 2, 2, AddressMode::Immedt),
            0x2A => (true , OpType::ROL, 1, 2, AddressMode::Accumu),
            0x2B => (false, OpType::ANC, 2, 2, AddressMode::Immedt),
            0x2C => (true , OpType::BIT, 3, 4, AddressMode::Absolu),
            0x2D => (true , OpType::AND, 3, 4, AddressMode::Absolu),
            0x2E => (true , OpType::ROL, 3, 6, AddressMode::Absolu),
            0x2F => (false, OpType::RLA, 3, 6, AddressMode::Absolu),
            0x30 => (true , OpType::BMI, 2, 2, AddressMode::Immedt),
            0x31 => (true , OpType::AND, 2, 5, AddressMode::IndIdx),
            0x32 => (false, OpType::KIL, 1, 0, AddressMode::Immedt),
            0x33 => (false, OpType::RLA, 2, 8, AddressMode::IndIdx),
            0x34 => (false, OpType::NOP, 2, 4, AddressMode::ZPIdxX),
            0x35 => (true , OpType::AND, 2, 4, AddressMode::ZPIdxX),
            0x36 => (true , OpType::ROL, 2, 6, AddressMode::ZPIdxX),
            0x37 => (false, OpType::RLA, 2, 6, AddressMode::ZPIdxX),
            0x38 => (true , OpType::SEC, 1, 2, AddressMode::Implid),
            0x39 => (true , OpType::AND, 3, 4, AddressMode::AbIdxY),
            0x3A => (false, OpType::NOP, 1, 2, AddressMode::Immedt),
            0x3B => (false, OpType::RLA, 3, 7, AddressMode::AbIdxY),
            0x3C => (false, OpType::NOP, 3, 4, AddressMode::AbIdxX),
            0x3D => (true , OpType::AND, 3, 4, AddressMode::AbIdxX),
            0x3E => (true , OpType::ROL, 3, 7, AddressMode::AbIdxX),
            0x3F => (false, OpType::RLA, 3, 7, AddressMode::AbIdxX),
            0x40 => (true , OpType::RTI, 1, 6, AddressMode::Implid),
            0x41 => (true , OpType::EOR, 2, 6, AddressMode::IdxInd),
            0x42 => (false, OpType::KIL, 1, 0, AddressMode::Immedt),
            0x43 => (false, OpType::SRE, 2, 8, AddressMode::IdxInd),
            0x44 => (false, OpType::NOP, 2, 3, AddressMode::ZeroPg),
            0x45 => (true , OpType::EOR, 2, 3, AddressMode::ZeroPg),
            0x46 => (true , OpType::LSR, 2, 5, AddressMode::ZeroPg),
            0x47 => (false, OpType::SRE, 2, 5, AddressMode::ZeroPg),
            0x48 => (true , OpType::PHA, 1, 3, AddressMode::Implid),
            0x49 => (true , OpType::EOR, 2, 2, AddressMode::Immedt),
            0x4A => (true , OpType::LSR, 1, 2, AddressMode::Accumu),
            0x4B => (false, OpType::ALR, 2, 2, AddressMode::Immedt),
            0x4C => (true , OpType::JMP, 3, 3, AddressMode::Absolu),
            0x4D => (true , OpType::EOR, 3, 4, AddressMode::Absolu),
            0x4E => (true , OpType::LSR, 3, 6, AddressMode::Absolu),
            0x4F => (false, OpType::SRE, 3, 6, AddressMode::Absolu),
            0x50 => (true , OpType::BVC, 2, 3, AddressMode::Immedt),
            0x51 => (true , OpType::EOR, 2, 5, AddressMode::IndIdx),
            0x52 => (false, OpType::KIL, 1, 0, AddressMode::Immedt),
            0x53 => (false, OpType::SRE, 2, 8, AddressMode::IndIdx),
            0x54 => (false, OpType::NOP, 2, 4, AddressMode::ZPIdxX),
            0x55 => (true , OpType::EOR, 2, 4, AddressMode::ZPIdxX),
            0x56 => (true , OpType::LSR, 2, 6, AddressMode::ZPIdxX),
            0x57 => (false, OpType::SRE, 2, 6, AddressMode::ZPIdxX),
            0x58 => (true , OpType::CLI, 1, 2, AddressMode::Implid),
            0x59 => (true , OpType::EOR, 3, 4, AddressMode::AbIdxY),
            0x5A => (false, OpType::NOP, 1, 2, AddressMode::Immedt),
            0x5B => (false, OpType::SRE, 3, 7, AddressMode::AbIdxY),
            0x5C => (false, OpType::NOP, 3, 4, AddressMode::AbIdxX),
            0x5D => (true , OpType::EOR, 3, 4, AddressMode::AbIdxX),
            0x5E => (true , OpType::LSR, 3, 7, AddressMode::AbIdxX),
            0x5F => (false, OpType::SRE, 3, 7, AddressMode::AbIdxX),
            0x60 => (true , OpType::RTS, 1, 6, AddressMode::Implid),
            0x61 => (true , OpType::ADC, 2, 6, AddressMode::IdxInd),
            0x62 => (false, OpType::KIL, 1, 0, AddressMode::Immedt),
            0x63 => (false, OpType::RRA, 2, 8, AddressMode::IdxInd),
            0x64 => (false, OpType::NOP, 2, 3, AddressMode::ZeroPg),
            0x65 => (true , OpType::ADC, 2, 3, AddressMode::ZeroPg),
            0x66 => (true , OpType::ROR, 2, 5, AddressMode::ZeroPg),
            0x67 => (false, OpType::RRA, 2, 5, AddressMode::ZeroPg),
            0x68 => (true , OpType::PLA, 1, 4, AddressMode::Implid),
            0x69 => (true , OpType::ADC, 2, 2, AddressMode::Immedt),
            0x6A => (true , OpType::ROR, 1, 2, AddressMode::Accumu),
            0x6B => (false, OpType::ARR, 2, 2, AddressMode::Immedt),
            0x6C => (true , OpType::JMP, 3, 5, AddressMode::Indrct),
            0x6D => (true , OpType::ADC, 3, 4, AddressMode::Absolu),
            0x6E => (true , OpType::ROR, 3, 6, AddressMode::Absolu),
            0x6F => (false, OpType::RRA, 3, 6, AddressMode::Absolu),
            0x70 => (true , OpType::BVS, 2, 2, AddressMode::Immedt),
            0x71 => (true , OpType::ADC, 2, 5, AddressMode::IndIdx),
            0x72 => (false, OpType::KIL, 1, 0, AddressMode::Immedt),
            0x73 => (false, OpType::RRA, 2, 8, AddressMode::IndIdx),
            0x74 => (false, OpType::NOP, 2, 4, AddressMode::ZPIdxX),
            0x75 => (true , OpType::ADC, 2, 4, AddressMode::ZPIdxX),
            0x76 => (true , OpType::ROR, 2, 6, AddressMode::ZPIdxX),
            0x77 => (false, OpType::RRA, 2, 6, AddressMode::ZPIdxX),
            0x78 => (true , OpType::SEI, 1, 2, AddressMode::Implid),
            0x79 => (true , OpType::ADC, 3, 4, AddressMode::AbIdxY),
            0x7A => (false, OpType::NOP, 1, 2, AddressMode::Immedt),
            0x7B => (false, OpType::RRA, 3, 7, AddressMode::AbIdxY),
            0x7C => (false, OpType::NOP, 3, 4, AddressMode::AbIdxX),
            0x7D => (true , OpType::ADC, 3, 4, AddressMode::AbIdxX),
            0x7E => (true , OpType::ROR, 3, 7, AddressMode::AbIdxX),
            0x7F => (false, OpType::RRA, 3, 7, AddressMode::AbIdxX),
            0x80 => (false, OpType::NOP, 2, 2, AddressMode::Immedt),
            0x81 => (true , OpType::STA, 2, 6, AddressMode::IdxInd),
            0x82 => (false, OpType::NOP, 2, 2, AddressMode::Immedt),
            0x83 => (false, OpType::SAX, 2, 6, AddressMode::IdxInd),
            0x84 => (true , OpType::STY, 2, 3, AddressMode::ZeroPg),
            0x85 => (true , OpType::STA, 2, 3, AddressMode::ZeroPg),
            0x86 => (true , OpType::STX, 2, 3, AddressMode::ZeroPg),
            0x87 => (false, OpType::SAX, 2, 3, AddressMode::ZeroPg),
            0x88 => (true , OpType::DEY, 1, 2, AddressMode::Implid),
            0x89 => (false, OpType::NOP, 2, 2, AddressMode::Immedt),
            0x8A => (true , OpType::TXA, 1, 2, AddressMode::Implid),
            0x8B => (false, OpType::XAA, 2, 2, AddressMode::Immedt),
            0x8C => (true , OpType::STY, 3, 4, AddressMode::Absolu),
            0x8D => (true , OpType::STA, 3, 4, AddressMode::Absolu),
            0x8E => (true , OpType::STX, 3, 4, AddressMode::Absolu),
            0x8F => (false, OpType::SAX, 3, 4, AddressMode::Absolu),
            0x90 => (true , OpType::BCC, 2, 3, AddressMode::Immedt),
            0x91 => (true , OpType::STA, 2, 6, AddressMode::IndIdx),
            0x92 => (false, OpType::KIL, 1, 0, AddressMode::Immedt),
            0x93 => (false, OpType::AHX, 2, 6, AddressMode::IndIdx),
            0x94 => (true , OpType::STY, 2, 4, AddressMode::ZPIdxX),
            0x95 => (true , OpType::STA, 2, 4, AddressMode::ZPIdxX),
            0x96 => (true , OpType::STX, 2, 4, AddressMode::ZPIdxY),
            0x97 => (false, OpType::SAX, 2, 4, AddressMode::ZPIdxY),
            0x98 => (true , OpType::TYA, 1, 2, AddressMode::Implid),
            0x99 => (true , OpType::STA, 3, 5, AddressMode::AbIdxY),
            0x9A => (true , OpType::TXS, 1, 2, AddressMode::Implid),
            0x9B => (false, OpType::TAS, 1, 5, AddressMode::Immedt),
            0x9C => (false, OpType::SHY, 3, 5, AddressMode::AbIdxX),
            0x9D => (true , OpType::STA, 3, 5, AddressMode::AbIdxX),
            0x9E => (false, OpType::SHX, 3, 5, AddressMode::AbIdxY),
            0x9F => (false, OpType::AHX, 3, 5, AddressMode::AbIdxY),
            0xA0 => (true , OpType::LDY, 2, 2, AddressMode::Immedt),
            0xA1 => (true , OpType::LDA, 2, 6, AddressMode::IdxInd),
            0xA2 => (true , OpType::LDX, 2, 2, AddressMode::Immedt),
            0xA3 => (false, OpType::LAX, 2, 6, AddressMode::IdxInd),
            0xA4 => (true , OpType::LDY, 2, 3, AddressMode::ZeroPg),
            0xA5 => (true , OpType::LDA, 2, 3, AddressMode::ZeroPg),
            0xA6 => (true , OpType::LDX, 2, 3, AddressMode::ZeroPg),
            0xA7 => (false, OpType::LAX, 2, 3, AddressMode::ZeroPg),
            0xA8 => (true , OpType::TAY, 1, 2, AddressMode::Implid),
            0xA9 => (true , OpType::LDA, 2, 2, AddressMode::Immedt),
            0xAA => (true , OpType::TAX, 1, 2, AddressMode::Implid),
            0xAB => (false, OpType::LAX, 2, 2, AddressMode::Immedt),
            0xAC => (true , OpType::LDY, 3, 4, AddressMode::Absolu),
            0xAD => (true , OpType::LDA, 3, 4, AddressMode::Absolu),
            0xAE => (true , OpType::LDX, 3, 4, AddressMode::Absolu),
            0xAF => (false, OpType::LAX, 3, 4, AddressMode::Absolu),
            0xB0 => (true , OpType::BCS, 2, 2, AddressMode::Immedt),
            0xB1 => (true , OpType::LDA, 2, 5, AddressMode::IndIdx),
            0xB2 => (false, OpType::KIL, 1, 0, AddressMode::Immedt),
            0xB3 => (false, OpType::LAX, 2, 5, AddressMode::IndIdx),
            0xB4 => (true , OpType::LDY, 2, 4, AddressMode::ZPIdxX),
            0xB5 => (true , OpType::LDA, 2, 4, AddressMode::ZPIdxX),
            0xB6 => (true , OpType::LDX, 2, 4, AddressMode::ZPIdxY),
            0xB7 => (false, OpType::LAX, 2, 4, AddressMode::ZPIdxY),
            0xB8 => (true , OpType::CLV, 1, 2, AddressMode::Implid),
            0xB9 => (true , OpType::LDA, 3, 4, AddressMode::AbIdxY),
            0xBA => (true , OpType::TSX, 1, 2, AddressMode::Implid),
            0xBB => (false, OpType::LAS, 3, 4, AddressMode::AbIdxY),
            0xBC => (true , OpType::LDY, 3, 4, AddressMode::AbIdxX),
            0xBD => (true , OpType::LDA, 3, 4, AddressMode::AbIdxX),
            0xBE => (true , OpType::LDX, 3, 4, AddressMode::AbIdxY),
            0xBF => (false, OpType::LAX, 3, 4, AddressMode::AbIdxY),
            0xC0 => (true , OpType::CPY, 2, 2, AddressMode::Immedt),
            0xC1 => (true , OpType::CMP, 2, 6, AddressMode::IdxInd),
            0xC2 => (false, OpType::NOP, 2, 2, AddressMode::Immedt),
            0xC3 => (false, OpType::DCP, 2, 8, AddressMode::IdxInd),
            0xC4 => (true , OpType::CPY, 2, 3, AddressMode::ZeroPg),
            0xC5 => (true , OpType::CMP, 2, 3, AddressMode::ZeroPg),
            0xC6 => (true , OpType::DEC, 2, 5, AddressMode::ZeroPg),
            0xC7 => (false, OpType::DCP, 2, 5, AddressMode::ZeroPg),
            0xC8 => (true , OpType::INY, 1, 2, AddressMode::Implid),
            0xC9 => (true , OpType::CMP, 2, 2, AddressMode::Immedt),
            0xCA => (true , OpType::DEX, 1, 2, AddressMode::Implid),
            0xCB => (false, OpType::AXS, 2, 2, AddressMode::Immedt),
            0xCC => (true , OpType::CPY, 3, 4, AddressMode::Absolu),
            0xCD => (true , OpType::CMP, 3, 4, AddressMode::Absolu),
            0xCE => (true , OpType::DEC, 3, 6, AddressMode::Absolu),
            0xCF => (false, OpType::DCP, 3, 6, AddressMode::Absolu),
            0xD0 => (true , OpType::BNE, 2, 2, AddressMode::Immedt),
            0xD1 => (true , OpType::CMP, 2, 5, AddressMode::IndIdx),
            0xD2 => (false, OpType::KIL, 1, 0, AddressMode::Immedt),
            0xD3 => (false, OpType::DCP, 2, 8, AddressMode::IndIdx),
            0xD4 => (false, OpType::NOP, 2, 4, AddressMode::ZPIdxX),
            0xD5 => (true , OpType::CMP, 2, 4, AddressMode::ZPIdxX),
            0xD6 => (true , OpType::DEC, 2, 6, AddressMode::ZPIdxX),
            0xD7 => (false, OpType::DCP, 2, 6, AddressMode::ZPIdxX),
            0xD8 => (true , OpType::CLD, 1, 2, AddressMode::Implid),
            0xD9 => (true , OpType::CMP, 3, 4, AddressMode::AbIdxY),
            0xDA => (false, OpType::NOP, 1, 2, AddressMode::Immedt),
            0xDB => (false, OpType::DCP, 3, 7, AddressMode::AbIdxY),
            0xDC => (false, OpType::NOP, 3, 4, AddressMode::AbIdxX),
            0xDD => (true , OpType::CMP, 3, 4, AddressMode::AbIdxX),
            0xDE => (true , OpType::DEC, 3, 7, AddressMode::AbIdxX),
            0xDF => (false, OpType::DCP, 3, 7, AddressMode::AbIdxX),
            0xE0 => (true , OpType::CPX, 2, 2, AddressMode::Immedt),
            0xE1 => (true , OpType::SBC, 2, 6, AddressMode::IdxInd),
            0xE2 => (false, OpType::NOP, 2, 2, AddressMode::Immedt),
            0xE3 => (false, OpType::ISC, 2, 8, AddressMode::IdxInd),
            0xE4 => (true , OpType::CPX, 2, 3, AddressMode::ZeroPg),
            0xE5 => (true , OpType::SBC, 2, 3, AddressMode::ZeroPg),
            0xE6 => (true , OpType::INC, 2, 5, AddressMode::ZeroPg),
            0xE7 => (false, OpType::ISC, 2, 5, AddressMode::ZeroPg),
            0xE8 => (true , OpType::INX, 1, 2, AddressMode::Implid),
            0xE9 => (true , OpType::SBC, 2, 2, AddressMode::Immedt),
            0xEA => (true , OpType::NOP, 1, 2, AddressMode::Implid),
            0xEB => (false, OpType::SBC, 2, 2, AddressMode::Immedt),
            0xEC => (true , OpType::CPX, 3, 4, AddressMode::Absolu),
            0xED => (true , OpType::SBC, 3, 4, AddressMode::Absolu),
            0xEE => (true , OpType::INC, 3, 6, AddressMode::Absolu),
            0xEF => (false, OpType::ISC, 3, 6, AddressMode::Absolu),
            0xF0 => (true , OpType::BEQ, 2, 2, AddressMode::Immedt),
            0xF1 => (true , OpType::SBC, 2, 5, AddressMode::IndIdx),
            0xF2 => (false, OpType::KIL, 1, 0, AddressMode::Immedt),
            0xF3 => (false, OpType::ISC, 2, 8, AddressMode::IndIdx),
            0xF4 => (false, OpType::NOP, 2, 4, AddressMode::ZPIdxX),
            0xF5 => (true , OpType::SBC, 2, 4, AddressMode::ZPIdxX),
            0xF6 => (true , OpType::INC, 2, 6, AddressMode::ZPIdxX),
            0xF7 => (false, OpType::ISC, 2, 6, AddressMode::ZPIdxX),
            0xF8 => (true , OpType::SED, 1, 2, AddressMode::Implid),
            0xF9 => (true , OpType::SBC, 3, 4, AddressMode::AbIdxY),
            0xFA => (false, OpType::NOP, 1, 2, AddressMode::Immedt),
            0xFB => (false, OpType::ISC, 3, 7, AddressMode::AbIdxY),
            0xFC => (false, OpType::NOP, 3, 4, AddressMode::AbIdxX),
            0xFD => (true , OpType::SBC, 3, 4, AddressMode::AbIdxX),
            0xFE => (true , OpType::INC, 3, 7, AddressMode::AbIdxX),
            0xFF => (false, OpType::ISC, 3, 7, AddressMode::AbIdxX),
            _ => panic!("none opcode:{:x}", opcode)
        };
        let (official, optype, bytes, cycle, address_mode) = r;
        r
    }

    fn decode_address(&mut self, address_mode: &AddressMode, pc: &mut u16) -> u16 {
        let result = match *address_mode {
            AddressMode::Immedt => { // Immediate : #value
                let offset = self.read(pc) as i8 as i16 as u16;
                pc.overflowing_add(offset).0
            },
            AddressMode::Implid => { // Implied : no operand
                0x00u16
            },
            AddressMode::Accumu => { // Accumulator : no operand
                self.a as u16
            },
            AddressMode::Relatv => { // Relative : $addr8 used with branch instructions
                let offset = self.read16(pc) as i16;
                let mut addr = (*pc as i16 + offset) as u16;
                self.read16(&mut addr)
            },
            AddressMode::ZeroPg => {    // Zero Page : $addr8
                let mut addr = self.read(pc) as u16;
                // self.read(&mut addr) as u16
                addr
            }
            AddressMode::ZPIdxX => {    // Zero Page Indexed with X : $addr8 + X
                let mut addr = (*pc + self.x as u16) | 0xFF;
                self.read(&mut addr) as u16
            },
            AddressMode::ZPIdxY => { // Zero Page Indexed with Y : $addr8 + Y
                let mut addr = (*pc + self.y as u16) | 0xFF;
                self.read(&mut addr) as u16
            },
            AddressMode::Absolu => { // Absolute : $addr16
                let mut addr = self.read16(pc);
                // self.read(&addr) as u16
                addr
            },
            AddressMode::AbIdxX => { // Absolute Indexed with X : $addr16 + X
                let (mut addr, overflow) = self.read16(pc).overflowing_add(self.x as u16);
                self.read(&mut addr) as u16
            },
            AddressMode::AbIdxY => { // Absolute Indexed with Y : $addr16 + Y
                let (mut addr, overflow) = self.read16(pc).overflowing_add(self.y as u16);
                self.read(&mut addr) as u16
            },
            AddressMode::Indrct => { // Indirect : ($addr8) used only with JMP
                let addr = self.read16(pc);
                self.read16bug(&addr)
            },
            AddressMode::IdxInd => { // Indexed with X Indirect : ($addr8 + X)
                let mut zp_addr = self.read(pc) as u16 + self.x as u16;
                let mut addr = self.read16bug(&mut zp_addr);
                self.read16(&mut addr)
            },
            AddressMode::IndIdx => { // Indirect Indexed with Y : ($addr8) + Y
                let mut arg = self.read(pc) as u16;
                let mut addr = self.read16bug(&mut arg) + self.y as u16;
                addr
            },
        };

        result
    }

    fn set_flag(&mut self, flag: u8, value:bool) {
        if value {
            self.p |= flag;
        } else {
            self.p &= !flag;
        }
    }

    fn get_flag(&mut self, flag: u8) -> bool {
        (self.p & flag) != 0
    }

    pub fn reset(&mut self) {
        self.pc = self.vector("reset");
        println!("reset vector:{:x}", self.pc);

        self.s = 0xFF;
    }

    fn vector(&mut self, name: &str) -> u16 {
        let mut addr = match name {
            "nmi"   => {0xFFFAu16}
            "reset" => {0xFFFCu16}
            "irq"   => {0xFFFEu16}
            _       => {panic!("invalid vector name:{}", name)}
        };
        self.read16(&mut addr)
    }
}

