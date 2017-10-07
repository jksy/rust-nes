use nes::rom::Rom;
use nes::mbc::Mbc;
use nes::opcode::{OpType, AddressMode};
use std;
use std::ptr::null;

pub struct Cpu {
    pub a: u8,      // accumulator
    pub x: u8,      // index register(X)
    pub y: u8,      // index register(Y)
    pub pc: u16,    // program counter
    pub s: u8,      // stack pointer
    pub p: u8,      // processor status register
    pub f: u8,      // flag register
    pub mbc: Mbc,
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

impl Cpu {
    pub fn new() -> Self {
        Cpu {a: 0, x: 0, y: 0, pc: 0, s: 0, p: 0, f:0, mbc: Mbc::new(Rom::empty())}
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.mbc.read(&self.pc);
            let (official, optype, bytes, cycle, addr_mode) = self.decode(opcode);
            let value = self.decode_address(&addr_mode);

            match optype {
                OpType::SEI => { self.f |= FLAG_IRQ; },
                OpType::CLI => { self.f |= !FLAG_IRQ; },

                // copy operators
                OpType::LDA => { self.a = self.mbc.read(&value) },
                OpType::LDX => { self.x = self.mbc.read(&value) },
                OpType::LDY => { self.y = self.mbc.read(&value) },

                // OpType::STA => { self.mbc.write(self.a) },
                // OpType::STX => { self.mbc.write(self.x) },
                // OpType::STY => { self.mbc.write(self.y) },

                OpType::TAX => { self.x = self.a; },
                OpType::TAY => { self.y = self.a; },
                OpType::TSX => { self.x = self.s; },
                OpType::TXA => { self.a = self.x; },
                OpType::TXS => { self.s = self.x; },
                OpType::TYA => { self.a = self.y; },

                // caluculate oprators
                OpType::ADC => { self.a = self.a + value as u8; },
                OpType::AND => { self.a = self.a & value as u8; },
                OpType::ASL => { },
                OpType::BIT => { },
                OpType::CMP => { },
                OpType::CPX => { },
                OpType::CPY => { },
                OpType::DEC => { },
                OpType::DEX => { self.x -= 1; },
                OpType::DEY => { self.y -= 1; },
                OpType::EOR => { self.a ^= value as u8; },
                OpType::INC => { },
                OpType::INX => { self.x += 1; },
                OpType::INY => { self.y += 1; },
                OpType::LSR => { },
                OpType::ORA => { self.a |= value as u8; },
                OpType::ROL => { },
                OpType::ROR => { },
                OpType::SBC => { },

                // STACK
                OpType::PHA => { let a = self.a; self.push(&a) },
                OpType::PHP => { let p = self.p; self.push(&p) },
                OpType::PLA => { self.a = self.pop() },
                OpType::PLP => { self.p = self.pop() },

                // JMP
                OpType::JMP => { self.pc = value; continue; },
                OpType::JSR => { self.pc = value; continue; },
                OpType::RTS => { self.pc = value; continue; },
                _ => {},
            }
            println!("decoding value:{:x}, opcode:{:x}, optype:{:?}, byte:{}, addr_mode:{:?}, value:{:x}", value, opcode, optype, bytes, addr_mode, value);

            self.pc += bytes
        }
    }

    fn push(&mut self, data: &u8) {
        let addr = self.s as u16;
        self.mbc.write(&addr, data);
        self.s += 1;
    }

    fn pushw(&mut self, data: &u16) {
        let low = (*data | 0xFF) as u8;
        let high = (*data >> 8) as u8;
        self.push(&low);
        self.push(&high);
    }

    fn pop(&mut self) -> u8 {
        let addr = self.s as u16;
        let data = self.mbc.read(&addr);
        self.s -= 1;
        data
    }

    fn popw(&mut self) -> u16 {
        let high = self.pop() as u16;
        let low = self.pop() as u16;
        (high << 8) | low
    }

    pub fn set_rom(&mut self, rom: Box<Rom>) {
        self.mbc.set_rom(rom);
    }

    pub fn disasm(&mut self) {
        self.pc = 0;
        while self.pc < self.mbc.prg_len() && self.pc + 0x8000 < 0xFFFA {
            println!("pc:{:x}, self.mbc.prg_len():{:x}", self.pc, self.mbc.prg_len());
            let addr = self.pc + 0x8000u16;
            let opcode = self.mbc.read(&addr);
            // if opcode != 0 {
            //     println!("{:x} => opcode:{:x}", self.pc, opcode);
            // }
            // self.pc += 1;
            let (official, optype, bytes, cycle, addr_mode) = self.decode(opcode);
            let value = self.decode_address(&addr_mode);
            println!("decoding addr:{:x}, opcode:{:x}, optype:{:?}, byte:{}, addr_mode:{:?}, value:{:x}", addr, opcode, optype, bytes, addr_mode, value);

            self.pc += bytes;
        }

        for name in &["nmi", "reset", "irq"] {
            let address = self.mbc.vector(&name);
            println!("vector:{}, {:x}", name, address);
        }
    }

    fn decode(&mut self, opcode: u8) -> (bool, OpType, u16, u16, AddressMode) {
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
            0xCA => (true , OpType::DEX, 1, 2, AddressMode::Immedt),
            0xCB => (false, OpType::AXS, 2, 2, AddressMode::Immedt),
            0xCC => (true , OpType::CPY, 3, 4, AddressMode::Absolu),
            0xCD => (true , OpType::CMP, 3, 4, AddressMode::Absolu),
            0xCE => (true , OpType::DEC, 3, 6, AddressMode::Absolu),
            0xCF => (false, OpType::DCP, 3, 6, AddressMode::Absolu),
            0xD0 => (true , OpType::BNE, 2, 3, AddressMode::Immedt),
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
        println!("{:?}",r);
        r
    }

    fn decode_address(&self, address_mode: &AddressMode) -> u16 {
        println!("decode_address:{:?}, self.pc:{:x}", address_mode, self.pc);
        let start = self.pc as u16 + 1;
        let result = match *address_mode {
            AddressMode::Immedt => { // Immediate : #value
                self.mbc.read(&start) as u16
            },
            AddressMode::Implid => { // Implied : no operand
                0x00u16
            },
            AddressMode::Accumu => { // Accumulator : no operand
                self.a as u16
            },
            AddressMode::Relatv => { // Relative : $addr8 used with branch instructions
                let offset = self.mbc.read16(&start) as i16;
                let addr = (self.pc as i16 + offset) as u16;
                self.mbc.read16(&addr)
            },
            AddressMode::ZeroPg => {    // Zero Page : $addr8
                let addr = self.mbc.read(&start) as u16;
                self.mbc.read(&addr) as u16
            }
            AddressMode::ZPIdxX => {    // Zero Page Indexed with X : $addr8 + X
                let addr = (self.pc + self.x as u16) | 0xFF;
                self.mbc.read(&addr) as u16
            },
            AddressMode::ZPIdxY => { // Zero Page Indexed with Y : $addr8 + Y
                let addr = (self.pc + self.y as u16) | 0xFF;
                self.mbc.read(&addr) as u16
            },
            AddressMode::Absolu => { // Absolute : $addr16
                let addr = self.mbc.read16(&start);
                self.mbc.read(&addr) as u16
            },
            AddressMode::AbIdxX => { // Absolute Indexed with X : $addr16 + X
                let addr = self.mbc.read16(&start) + self.x as u16;
                self.mbc.read(&addr) as u16
            },
            AddressMode::AbIdxY => { // Absolute Indexed with Y : $addr16 + Y
                let addr = self.mbc.read16(&start) + self.y as u16;
                self.mbc.read(&addr) as u16
            },
            AddressMode::Indrct => { // Indirect : ($addr8) used only with JMP
                let addr = self.mbc.read16(&start);
                addr
            },
            AddressMode::IdxInd => { // Indexed with X Indirect : ($addr8 + X)
                let zp_addr = ((self.mbc.read(&start) + self.x) | 0xFFu8) as u16;
                let addr = self.mbc.read16(&zp_addr);
                self.mbc.read16(&addr)
            },
            AddressMode::IndIdx => { // Indirect Indexed with Y : ($addr8) + Y
                let zp_addr = (self.mbc.read(&start) | 0xFFu8) as u16;
                let addr = self.mbc.read16(&zp_addr) + self.y as u16;
                self.mbc.read16(&addr)
            },
        };

        result
    }

    fn read8(&self, addr: &u16) -> u8 {
        self.mbc.read(addr)
    }
    fn read16(&self, addr: &u16) -> u16 {
        let second = addr + 1;
        (self.mbc.read(addr) as u16) |
        ((self.mbc.read(&second) as u16) << 8)
    }

    pub fn reset(&mut self) {
        self.pc = self.mbc.vector("reset");
        println!("reset vector:{:x}", self.pc);

        self.s = 0xFF;
    }

    fn lda(&mut self) {
        self.a = 1
    }
    fn ldx(&mut self) {
        self.x = 2
    }
    fn ldy(&mut self) {
        self.y = 3
    }
}

