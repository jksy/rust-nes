use nes::rom::Rom;
use nes::mbc::Mbc;
use nes::opcode::{OpType, AddressMode};
use std;
use std::ptr::null;

pub struct Cpu<'a> {
    pub a: u8,      // accumulator
    pub x: u8,      // index register(X)
    pub y: u8,      // index register(Y)
    pub pc: u16,    // program counter
    pub s: u8,      // stack pointer
    pub p: u8,      // processor status register
    // pub ram: [u8; 2048],
    pub mbc: Mbc<'a>,
}

impl<'a> Cpu<'a> {
    pub fn new() -> Self {
        Cpu {a: 0, x: 0, y: 0, pc: 0, s: 0, p: 0, mbc: Mbc::new(None)}
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.mbc.read(&self.pc);
            match opcode {
                0x00 => self.lda(),
                0x01 => self.ldx(),
                0x02 => self.ldy(),
                _ => {println!("invalid opcode:{}", self.pc); break}
            }
            self.pc += 1
        }
    }

    pub fn set_rom(&mut self, rom: &'a Rom) {
        self.mbc.set_rom(&rom);
    }

    pub fn disasm(&mut self, rom: Rom) {
        self.pc = 0;
        while self.pc < rom.prg_len() {
            let opcode = rom.prg(self.pc);
            // if opcode != 0 {
            //     println!("{:x} => opcode:{:x}", self.pc, opcode);
            // }
            // self.pc += 1;
            let (optype, bytes, cycle, address) = self.decode_op(opcode);
            self.pc += bytes;
        }
    }

    fn decode_op(&mut self, opcode: u8) -> (OpType, u16, u16, AddressMode) {
        println!("decoding pc:{:x}, opcode:{:x}", self.pc, opcode);

        let r = match opcode {
            0x69 => (OpType::ADC, 2, 2, AddressMode::Immedt),
            0x65 => (OpType::ADC, 2, 3, AddressMode::ZeroPg),
            0x75 => (OpType::ADC, 2, 4, AddressMode::ZPIdxX),
            0x6D => (OpType::ADC, 3, 4, AddressMode::Absolu),
            0x7D => (OpType::ADC, 3, 4, AddressMode::AbIdxX),
            0x79 => (OpType::ADC, 3, 4, AddressMode::AbIdxY),
            0x61 => (OpType::ADC, 2, 6, AddressMode::IdxInd),
            0x71 => (OpType::ADC, 2, 5, AddressMode::IndIdx),

            0x29 => (OpType::AND, 2, 2, AddressMode::Immedt),
            0x25 => (OpType::AND, 2, 3, AddressMode::ZeroPg),
            0x35 => (OpType::AND, 2, 4, AddressMode::ZPIdxX),
            0x2D => (OpType::AND, 3, 4, AddressMode::Absolu),
            0x3D => (OpType::AND, 3, 4, AddressMode::AbIdxX),
            0x39 => (OpType::AND, 3, 4, AddressMode::AbIdxY),
            0x21 => (OpType::AND, 2, 6, AddressMode::IdxInd),
            0x31 => (OpType::AND, 2, 5, AddressMode::IndIdx),

            0x0A => (OpType::ASL, 1, 2, AddressMode::Accumu),
            0x06 => (OpType::ASL, 2, 5, AddressMode::ZeroPg),
            0x16 => (OpType::ASL, 2, 6, AddressMode::ZPIdxX),
            0x0E => (OpType::ASL, 3, 6, AddressMode::Absolu),
            0x1E => (OpType::ASL, 3, 7, AddressMode::AbIdxX),

            0x90 => (OpType::BCC, 2, 2, AddressMode::Relatv),

            0xB0 => (OpType::BCS, 2, 2, AddressMode::Relatv),

            0xF0 => (OpType::BEQ, 2, 2, AddressMode::Relatv),

            0x24 => (OpType::BIT, 2, 3, AddressMode::ZeroPg),
            0x2C => (OpType::BIT, 3, 4, AddressMode::Absolu),

            0x30 => (OpType::BMI, 2, 2, AddressMode::Relatv),

            0xD0 => (OpType::BNE, 2, 2, AddressMode::Relatv),

            0x10 => (OpType::BPL, 2, 2, AddressMode::Relatv),

            0x00 => (OpType::BRK, 1, 7, AddressMode::Implid),

            0x50 => (OpType::BVC, 2, 2, AddressMode::Relatv),

            0x70 => (OpType::BVS, 2, 2, AddressMode::Relatv),

            0x18 => (OpType::CLC, 1, 2, AddressMode::Implid),

            0xD8 => (OpType::CLD, 1, 2, AddressMode::Implid),

            0x58 => (OpType::CLI, 1, 2, AddressMode::Implid),

            0xB8 => (OpType::CLV, 1, 2, AddressMode::Implid),

            0xC9 => (OpType::CMP, 2, 2, AddressMode::Immedt),
            0xC5 => (OpType::CMP, 2, 3, AddressMode::ZeroPg),
            0xD5 => (OpType::CMP, 2, 4, AddressMode::ZPIdxX),
            0xCD => (OpType::CMP, 3, 4, AddressMode::Absolu),
            0xDD => (OpType::CMP, 3, 4, AddressMode::AbIdxX),
            0xD9 => (OpType::CMP, 3, 4, AddressMode::AbIdxY),
            0xC1 => (OpType::CMP, 2, 6, AddressMode::IdxInd),
            0xD1 => (OpType::CMP, 2, 5, AddressMode::IndIdx),

            0xE0 => (OpType::CPX, 2, 2, AddressMode::Immedt),
            0xE4 => (OpType::CPX, 2, 3, AddressMode::ZeroPg),
            0xEC => (OpType::CPX, 3, 4, AddressMode::Absolu),

            0xC0 => (OpType::CPY, 2, 2, AddressMode::Immedt),
            0xC4 => (OpType::CPY, 2, 3, AddressMode::ZeroPg),
            0xCC => (OpType::CPY, 3, 4, AddressMode::Absolu),

            0xC6 => (OpType::DEC, 2, 5, AddressMode::ZeroPg),
            0xD6 => (OpType::DEC, 2, 6, AddressMode::ZPIdxX),
            0xCE => (OpType::DEC, 3, 6, AddressMode::Absolu),
            0xDE => (OpType::DEC, 3, 7, AddressMode::AbIdxX),

            0xCA => (OpType::DEX, 1, 2, AddressMode::Implid),

            0x88 => (OpType::DEY, 1, 2, AddressMode::Implid),

            0x49 => (OpType::EOR, 2, 2, AddressMode::Immedt),
            0x45 => (OpType::EOR, 2, 3, AddressMode::ZeroPg),
            0x55 => (OpType::EOR, 2, 4, AddressMode::ZPIdxX),
            0x4D => (OpType::EOR, 3, 4, AddressMode::Absolu),
            0x5D => (OpType::EOR, 3, 4, AddressMode::AbIdxX),
            0x59 => (OpType::EOR, 3, 4, AddressMode::AbIdxY),
            0x41 => (OpType::EOR, 2, 6, AddressMode::IdxInd),
            0x51 => (OpType::EOR, 2, 5, AddressMode::IndIdx),

            0xE6 => (OpType::INC, 2, 5, AddressMode::ZeroPg),
            0xF6 => (OpType::INC, 2, 6, AddressMode::ZPIdxX),
            0xEE => (OpType::INC, 3, 6, AddressMode::Absolu),
            0xFE => (OpType::INC, 3, 7, AddressMode::AbIdxX),

            0xE8 => (OpType::INX, 1, 2, AddressMode::Implid),

            0xC8 => (OpType::INY, 1, 2, AddressMode::Implid),

            0x4C => (OpType::JMP, 3, 3, AddressMode::Absolu),
            0x6C => (OpType::JMP, 3, 5, AddressMode::Indrct),

            0x20 => (OpType::JSR, 3, 6, AddressMode::Absolu),

            0xA9 => (OpType::LDA, 2, 2, AddressMode::Immedt),
            0xA5 => (OpType::LDA, 2, 3, AddressMode::ZeroPg),
            0xB5 => (OpType::LDA, 2, 4, AddressMode::ZPIdxX),
            0xAD => (OpType::LDA, 3, 4, AddressMode::Absolu),
            0xBD => (OpType::LDA, 3, 4, AddressMode::AbIdxX),
            0xB9 => (OpType::LDA, 3, 4, AddressMode::AbIdxY),
            0xA1 => (OpType::LDA, 2, 6, AddressMode::IdxInd),
            0xB1 => (OpType::LDA, 2, 5, AddressMode::IndIdx),

            0xA2 => (OpType::LDX, 2, 2, AddressMode::Immedt),
            0xA6 => (OpType::LDX, 2, 3, AddressMode::ZeroPg),
            0xB6 => (OpType::LDX, 2, 4, AddressMode::ZPIdxY),
            0xAE => (OpType::LDX, 3, 4, AddressMode::Absolu),
            0xBE => (OpType::LDX, 3, 4, AddressMode::AbIdxY),

            0xA0 => (OpType::LDY, 2, 2, AddressMode::Immedt),
            0xA4 => (OpType::LDY, 2, 3, AddressMode::ZeroPg),
            0xB4 => (OpType::LDY, 2, 4, AddressMode::ZPIdxX),
            0xAC => (OpType::LDY, 3, 4, AddressMode::Absolu),
            0xBC => (OpType::LDY, 3, 4, AddressMode::AbIdxX),

            0x4A => (OpType::LSR, 1, 2, AddressMode::Accumu),
            0x46 => (OpType::LSR, 2, 5, AddressMode::ZeroPg),
            0x56 => (OpType::LSR, 2, 6, AddressMode::ZPIdxX),
            0x4E => (OpType::LSR, 3, 6, AddressMode::Absolu),
            0x5E => (OpType::LSR, 3, 7, AddressMode::AbIdxX),

            0xEA => (OpType::NOP, 1, 2, AddressMode::Implid),

            0x09 => (OpType::ORA, 2, 2, AddressMode::Immedt),
            0x05 => (OpType::ORA, 2, 3, AddressMode::ZeroPg),
            0x15 => (OpType::ORA, 2, 4, AddressMode::ZPIdxX),
            0x0D => (OpType::ORA, 3, 4, AddressMode::Absolu),
            0x1D => (OpType::ORA, 3, 4, AddressMode::AbIdxX),
            0x19 => (OpType::ORA, 3, 4, AddressMode::AbIdxY),
            0x01 => (OpType::ORA, 2, 6, AddressMode::IdxInd),
            0x11 => (OpType::ORA, 2, 5, AddressMode::IndIdx),

            0x48 => (OpType::PHA, 1, 3, AddressMode::Implid),

            0x08 => (OpType::PHP, 1, 3, AddressMode::Implid),

            0x68 => (OpType::PLA, 1, 4, AddressMode::Implid),

            0x28 => (OpType::PLP, 1, 4, AddressMode::Implid),

            0x2A => (OpType::ROL, 1, 2, AddressMode::Accumu),
            0x26 => (OpType::ROL, 2, 5, AddressMode::ZeroPg),
            0x36 => (OpType::ROL, 2, 6, AddressMode::ZPIdxX),
            0x2E => (OpType::ROL, 3, 6, AddressMode::Absolu),
            0x3E => (OpType::ROL, 3, 7, AddressMode::AbIdxX),

            0x6A => (OpType::ROR, 1, 2, AddressMode::Accumu),
            0x66 => (OpType::ROR, 2, 5, AddressMode::ZeroPg),
            0x76 => (OpType::ROR, 2, 6, AddressMode::ZPIdxX),
            0x6E => (OpType::ROR, 3, 6, AddressMode::Absolu),
            0x7E => (OpType::ROR, 3, 7, AddressMode::AbIdxX),

            0x40 => (OpType::RTI, 1, 6, AddressMode::Implid),

            0x60 => (OpType::RTS, 1, 6, AddressMode::Implid),

            0xE9 => (OpType::SBC, 2, 2, AddressMode::Immedt),
            0xE5 => (OpType::SBC, 2, 3, AddressMode::ZeroPg),
            0xF5 => (OpType::SBC, 2, 4, AddressMode::ZPIdxX),
            0xED => (OpType::SBC, 3, 4, AddressMode::Absolu),
            0xFD => (OpType::SBC, 3, 4, AddressMode::AbIdxX),
            0xF9 => (OpType::SBC, 3, 4, AddressMode::AbIdxY),
            0xE1 => (OpType::SBC, 2, 6, AddressMode::IdxInd),
            0xF1 => (OpType::SBC, 2, 5, AddressMode::IndIdx),

            0x38 => (OpType::SEC, 1, 2, AddressMode::Implid),

            0xF8 => (OpType::SED, 1, 2, AddressMode::Implid),

            0x78 => (OpType::SEI, 1, 2, AddressMode::Implid),

            0x85 => (OpType::STA, 2, 3, AddressMode::ZeroPg),
            0x95 => (OpType::STA, 2, 4, AddressMode::ZPIdxX),
            0x8D => (OpType::STA, 3, 4, AddressMode::Absolu),
            0x9D => (OpType::STA, 3, 5, AddressMode::AbIdxX),
            0x99 => (OpType::STA, 3, 5, AddressMode::AbIdxY),
            0x81 => (OpType::STA, 2, 6, AddressMode::IdxInd),
            0x91 => (OpType::STA, 2, 6, AddressMode::IndIdx),

            0x86 => (OpType::STX, 2, 3, AddressMode::ZeroPg),
            0x96 => (OpType::STX, 2, 4, AddressMode::ZPIdxY),
            0x8E => (OpType::STX, 3, 4, AddressMode::Absolu),

            0x84 => (OpType::STY, 2, 3, AddressMode::ZeroPg),
            0x94 => (OpType::STY, 2, 4, AddressMode::ZPIdxX),
            0x8C => (OpType::STY, 3, 4, AddressMode::Absolu),

            0xAA => (OpType::TAX, 1, 2, AddressMode::Implid),

            0xA8 => (OpType::TAY, 1, 2, AddressMode::Implid),

            0xBA => (OpType::TSX, 1, 2, AddressMode::Implid),

            0x8A => (OpType::TXA, 1, 2, AddressMode::Implid),

            0x9A => (OpType::TXS, 1, 2, AddressMode::Implid),

            0x98 => (OpType::TYA, 1, 2, AddressMode::Implid),
            _ => panic!("none opcode:{:x}", opcode)
        };
        let (optype, bytes, cycle, address_mode) = r;
        println!("{:?}",r);
        r
    }

    fn decode_address(&self, address_mode: &AddressMode, arg: &u16) -> u16 {
        let mode = *address_mode;
        let result = match *address_mode {
            AddressMode::Immedt => self.pc, //?    // Immediate : #value
            AddressMode::Implid => 0u16, //?    // Implied : no operand
            AddressMode::Accumu => self.a as u16, //?    // Accumulator : no operand
            AddressMode::Relatv => (self.pc + arg) as u16, // Relative : $addr8 used with branch instructions
            AddressMode::ZeroPg => self.mbc.read(&self.pc) as u16,    // Zero Page : $addr8
            AddressMode::ZPIdxX => (self.mbc.read(&self.pc) + self.x) as u16,    // Zero Page Indexed with X : $addr8 + X
            AddressMode::ZPIdxY => (self.mbc.read(&self.pc) + self.y) as u16, // Zero Page Indexed with Y : $addr8 + Y
            AddressMode::Absolu => 0u16, // Absolute : $addr16
            AddressMode::AbIdxX => 0u16, // Absolute Indexed with X : $addr16 + X
            AddressMode::AbIdxY => 0u16, // Absolute Indexed with Y : $addr16 + Y
            AddressMode::Indrct => 0u16, // Indirect : ($addr8) used only with JMP
            AddressMode::IdxInd => 0u16, // Indexed with X Indirect : ($addr8 + X)
            AddressMode::IndIdx => 0u16, // Indirect Indexed with Y : ($addr8) + Y
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
        let high = self.mbc.read(&0xFFFC);
        let low = self.mbc.read(&0xFFFD);
        println!("reset vector:{},{}", high, low);

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

