//
// copy from https://github.com/amaiorano/nes-disasm
//
//
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Copy, Clone)]
pub enum AddressMode {
    Immedt,    // Immediate : #value
    Implid,    // Implied : no operand
    Accumu,    // Accumulator : no operand
    Relatv, // Relative : $addr8 used with branch instructions
    ZeroPg,    // Zero Page : $addr8
    ZPIdxX,    // Zero Page Indexed with X : $addr8 + X
    ZPIdxY, // Zero Page Indexed with Y : $addr8 + Y
    Absolu, // Absolute : $addr16
    AbIdxX, // Absolute Indexed with X : $addr16 + X
    AbIdxY, // Absolute Indexed with Y : $addr16 + Y
    Indrct, // Indirect : ($addr8) used only with JMP
    IdxInd, // Indexed with X Indirect : ($addr8 + X)
    IndIdx, // Indirect Indexed with Y : ($addr8) + Y
}

#[derive(Debug, Copy, Clone)]
pub enum OpType {
    ADC, AND, ASL,
    BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS,
    CLC, CLD, CLI, CLV, CMP, CPX, CPY, DEC, DEX, DEY,
    EOR, INC, INX, INY,
    JMP, JSR,
    LDA, LDX, LDY, LSR,
    NOP,
    ORA,
    PHA, PHP, PLA, PLP,
    ROL, ROR, RTI, RTS,
    SBC, SEC, SED, SEI, STA, STX, STY,
    TAX, TAY, TSX, TXA, TXS, TYA,
}

// struct OpCodeEntry<'a> {
//     opcode: &'a u8,
//     optype: &'a OpType,
//     bytes:  &'a u8,
//     cycle:  &'a u8,
//     address_mode: &'a AddressMode,
// }
//
// impl<'a> OpCodeEntry<'a> {
//     fn new(opcode: u8,
//            optype: OpType,
//            bytes: u8,
//            cycle: u8,
//            address_mode: AddressMode) -> Self {
//         OpCodeEntry{opcode: &opcode,
//                     optype: &optype,
//                     bytes: &bytes,
//                     cycle: &cycle,
//                     address_mode: &address_mode}
//     }
// }

// let opcode_table  = [
//     [0x69, OpType::ADC, 2, 2, AddressMode::Immedt],
//     [0x65, OpType::ADC, 2, 3, AddressMode::ZeroPg],
//     [0x75, OpType::ADC, 2, 4, AddressMode::ZPIdxX],
//     [0x6D, OpType::ADC, 3, 4, AddressMode::Absolu],
//     [0x7D, OpType::ADC, 3, 4, AddressMode::AbIdxX],
//     [0x79, OpType::ADC, 3, 4, AddressMode::AbIdxY],
//     [0x61, OpType::ADC, 2, 6, AddressMode::IdxInd],
//     [0x71, OpType::ADC, 2, 5, AddressMode::IndIdx],
//
//     [0x29, OpType::AND, 2, 2, AddressMode::Immedt],
//     [0x25, OpType::AND, 2, 3, AddressMode::ZeroPg],
//     [0x35, OpType::AND, 2, 4, AddressMode::ZPIdxX],
//     [0x2D, OpType::AND, 3, 4, AddressMode::Absolu],
//     [0x3D, OpType::AND, 3, 4, AddressMode::AbIdxX],
//     [0x39, OpType::AND, 3, 4, AddressMode::AbIdxY],
//     [0x21, OpType::AND, 2, 6, AddressMode::IdxInd],
//     [0x31, OpType::AND, 2, 5, AddressMode::IndIdx],
//
//     [0x0A, OpType::ASL, 1, 2, AddressMode::Accumu],
//     [0x06, OpType::ASL, 2, 5, AddressMode::ZeroPg],
//     [0x16, OpType::ASL, 2, 6, AddressMode::ZPIdxX],
//     [0x0E, OpType::ASL, 3, 6, AddressMode::Absolu],
//     [0x1E, OpType::ASL, 3, 7, AddressMode::AbIdxX],
//
//     [0x90, OpType::BCC, 2, 2, AddressMode::Relatv],
//
//     [0xB0, OpType::BCS, 2, 2, AddressMode::Relatv],
//
//     [0xF0, OpType::BEQ, 2, 2, AddressMode::Relatv],
//
//     [0x24, OpType::BIT, 2, 3, AddressMode::ZeroPg],
//     [0x2C, OpType::BIT, 3, 4, AddressMode::Absolu],
//
//     [0x30, OpType::BMI, 2, 2, AddressMode::Relatv],
//
//     [0xD0, OpType::BNE, 2, 2, AddressMode::Relatv],
//
//     [0x10, OpType::BPL, 2, 2, AddressMode::Relatv],
//
//     [0x00, OpType::BRK, 1, 7, AddressMode::Implid],
//
//     [0x50, OpType::BVC, 2, 2, AddressMode::Relatv],
//
//     [0x70, OpType::BVS, 2, 2, AddressMode::Relatv],
//
//     [0x18, OpType::CLC, 1, 2, AddressMode::Implid],
//
//     [0xD8, OpType::CLD, 1, 2, AddressMode::Implid],
//
//     [0x58, OpType::CLI, 1, 2, AddressMode::Implid],
//
//     [0xB8, OpType::CLV, 1, 2, AddressMode::Implid],
//
//     [0xC9, OpType::CMP, 2, 2, AddressMode::Immedt],
//     [0xC5, OpType::CMP, 2, 3, AddressMode::ZeroPg],
//     [0xD5, OpType::CMP, 2, 4, AddressMode::ZPIdxX],
//     [0xCD, OpType::CMP, 3, 4, AddressMode::Absolu],
//     [0xDD, OpType::CMP, 3, 4, AddressMode::AbIdxX],
//     [0xD9, OpType::CMP, 3, 4, AddressMode::AbIdxY],
//     [0xC1, OpType::CMP, 2, 6, AddressMode::IdxInd],
//     [0xD1, OpType::CMP, 2, 5, AddressMode::IndIdx],
//
//     [0xE0, OpType::CPX, 2, 2, AddressMode::Immedt],
//     [0xE4, OpType::CPX, 2, 3, AddressMode::ZeroPg],
//     [0xEC, OpType::CPX, 3, 4, AddressMode::Absolu],
//
//     [0xC0, OpType::CPY, 2, 2, AddressMode::Immedt],
//     [0xC4, OpType::CPY, 2, 3, AddressMode::ZeroPg],
//     [0xCC, OpType::CPY, 3, 4, AddressMode::Absolu],
//
//     [0xC6, OpType::DEC, 2, 5, AddressMode::ZeroPg],
//     [0xD6, OpType::DEC, 2, 6, AddressMode::ZPIdxX],
//     [0xCE, OpType::DEC, 3, 6, AddressMode::Absolu],
//     [0xDE, OpType::DEC, 3, 7, AddressMode::AbIdxX],
//
//     [0xCA, OpType::DEX, 1, 2, AddressMode::Implid],
//
//     [0x88, OpType::DEY, 1, 2, AddressMode::Implid],
//
//     [0x49, OpType::EOR, 2, 2, AddressMode::Immedt],
//     [0x45, OpType::EOR, 2, 3, AddressMode::ZeroPg],
//     [0x55, OpType::EOR, 2, 4, AddressMode::ZPIdxX],
//     [0x4D, OpType::EOR, 3, 4, AddressMode::Absolu],
//     [0x5D, OpType::EOR, 3, 4, AddressMode::AbIdxX],
//     [0x59, OpType::EOR, 3, 4, AddressMode::AbIdxY],
//     [0x41, OpType::EOR, 2, 6, AddressMode::IdxInd],
//     [0x51, OpType::EOR, 2, 5, AddressMode::IndIdx],
//
//     [0xE6, OpType::INC, 2, 5, AddressMode::ZeroPg],
//     [0xF6, OpType::INC, 2, 6, AddressMode::ZPIdxX],
//     [0xEE, OpType::INC, 3, 6, AddressMode::Absolu],
//     [0xFE, OpType::INC, 3, 7, AddressMode::AbIdxX],
//
//     [0xE8, OpType::INX, 1, 2, AddressMode::Implid],
//
//     [0xC8, OpType::INY, 1, 2, AddressMode::Implid],
//
//     [0x4C, OpType::JMP, 3, 3, AddressMode::Absolu],
//     [0x6C, OpType::JMP, 3, 5, AddressMode::Indrct],
//
//     [0x20, OpType::JSR, 3, 6, AddressMode::Absolu],
//
//     [0xA9, OpType::LDA, 2, 2, AddressMode::Immedt],
//     [0xA5, OpType::LDA, 2, 3, AddressMode::ZeroPg],
//     [0xB5, OpType::LDA, 2, 4, AddressMode::ZPIdxX],
//     [0xAD, OpType::LDA, 3, 4, AddressMode::Absolu],
//     [0xBD, OpType::LDA, 3, 4, AddressMode::AbIdxX],
//     [0xB9, OpType::LDA, 3, 4, AddressMode::AbIdxY],
//     [0xA1, OpType::LDA, 2, 6, AddressMode::IdxInd],
//     [0xB1, OpType::LDA, 2, 5, AddressMode::IndIdx],
//
//     [0xA2, OpType::LDX, 2, 2, AddressMode::Immedt],
//     [0xA6, OpType::LDX, 2, 3, AddressMode::ZeroPg],
//     [0xB6, OpType::LDX, 2, 4, AddressMode::ZPIdxY],
//     [0xAE, OpType::LDX, 3, 4, AddressMode::Absolu],
//     [0xBE, OpType::LDX, 3, 4, AddressMode::AbIdxY],
//
//     [0xA0, OpType::LDY, 2, 2, AddressMode::Immedt],
//     [0xA4, OpType::LDY, 2, 3, AddressMode::ZeroPg],
//     [0xB4, OpType::LDY, 2, 4, AddressMode::ZPIdxX],
//     [0xAC, OpType::LDY, 3, 4, AddressMode::Absolu],
//     [0xBC, OpType::LDY, 3, 4, AddressMode::AbIdxX],
//
//     [0x4A, OpType::LSR, 1, 2, AddressMode::Accumu],
//     [0x46, OpType::LSR, 2, 5, AddressMode::ZeroPg],
//     [0x56, OpType::LSR, 2, 6, AddressMode::ZPIdxX],
//     [0x4E, OpType::LSR, 3, 6, AddressMode::Absolu],
//     [0x5E, OpType::LSR, 3, 7, AddressMode::AbIdxX],
//
//     [0xEA, OpType::NOP, 1, 2, AddressMode::Implid],
//
//     [0x09, OpType::ORA, 2, 2, AddressMode::Immedt],
//     [0x05, OpType::ORA, 2, 3, AddressMode::ZeroPg],
//     [0x15, OpType::ORA, 2, 4, AddressMode::ZPIdxX],
//     [0x0D, OpType::ORA, 3, 4, AddressMode::Absolu],
//     [0x1D, OpType::ORA, 3, 4, AddressMode::AbIdxX],
//     [0x19, OpType::ORA, 3, 4, AddressMode::AbIdxY],
//     [0x01, OpType::ORA, 2, 6, AddressMode::IdxInd],
//     [0x11, OpType::ORA, 2, 5, AddressMode::IndIdx],
//
//     [0x48, OpType::PHA, 1, 3, AddressMode::Implid],
//
//     [0x08, OpType::PHP, 1, 3, AddressMode::Implid],
//
//     [0x68, OpType::PLA, 1, 4, AddressMode::Implid],
//
//     [0x28, OpType::PLP, 1, 4, AddressMode::Implid],
//
//     [0x2A, OpType::ROL, 1, 2, AddressMode::Accumu],
//     [0x26, OpType::ROL, 2, 5, AddressMode::ZeroPg],
//     [0x36, OpType::ROL, 2, 6, AddressMode::ZPIdxX],
//     [0x2E, OpType::ROL, 3, 6, AddressMode::Absolu],
//     [0x3E, OpType::ROL, 3, 7, AddressMode::AbIdxX],
//
//     [0x6A, OpType::ROR, 1, 2, AddressMode::Accumu],
//     [0x66, OpType::ROR, 2, 5, AddressMode::ZeroPg],
//     [0x76, OpType::ROR, 2, 6, AddressMode::ZPIdxX],
//     [0x6E, OpType::ROR, 3, 6, AddressMode::Absolu],
//     [0x7E, OpType::ROR, 3, 7, AddressMode::AbIdxX],
//
//     [0x40, OpType::RTI, 1, 6, AddressMode::Implid],
//
//     [0x60, OpType::RTS, 1, 6, AddressMode::Implid],
//
//     [0xE9, OpType::SBC, 2, 2, AddressMode::Immedt],
//     [0xE5, OpType::SBC, 2, 3, AddressMode::ZeroPg],
//     [0xF5, OpType::SBC, 2, 4, AddressMode::ZPIdxX],
//     [0xED, OpType::SBC, 3, 4, AddressMode::Absolu],
//     [0xFD, OpType::SBC, 3, 4, AddressMode::AbIdxX],
//     [0xF9, OpType::SBC, 3, 4, AddressMode::AbIdxY],
//     [0xE1, OpType::SBC, 2, 6, AddressMode::IdxInd],
//     [0xF1, OpType::SBC, 2, 5, AddressMode::IndIdx],
//
//     [0x38, OpType::SEC, 1, 2, AddressMode::Implid],
//
//     [0xF8, OpType::SED, 1, 2, AddressMode::Implid],
//
//     [0x78, OpType::SEI, 1, 2, AddressMode::Implid],
//
//     [0x85, OpType::STA, 2, 3, AddressMode::ZeroPg],
//     [0x95, OpType::STA, 2, 4, AddressMode::ZPIdxX],
//     [0x8D, OpType::STA, 3, 4, AddressMode::Absolu],
//     [0x9D, OpType::STA, 3, 5, AddressMode::AbIdxX],
//     [0x99, OpType::STA, 3, 5, AddressMode::AbIdxY],
//     [0x81, OpType::STA, 2, 6, AddressMode::IdxInd],
//     [0x91, OpType::STA, 2, 6, AddressMode::IndIdx],
//
//     [0x86, OpType::STX, 2, 3, AddressMode::ZeroPg],
//     [0x96, OpType::STX, 2, 4, AddressMode::ZPIdxY],
//     [0x8E, OpType::STX, 3, 4, AddressMode::Absolu],
//
//     [0x84, OpType::STY, 2, 3, AddressMode::ZeroPg],
//     [0x94, OpType::STY, 2, 4, AddressMode::ZPIdxX],
//     [0x8C, OpType::STY, 3, 4, AddressMode::Absolu],
//
//     [0xAA, OpType::TAX, 1, 2, AddressMode::Implid],
//
//     [0xA8, OpType::TAY, 1, 2, AddressMode::Implid],
//
//     [0xBA, OpType::TSX, 1, 2, AddressMode::Implid],
//
//     [0x8A, OpType::TXA, 1, 2, AddressMode::Implid],
//
//     [0x9A, OpType::TXS, 1, 2, AddressMode::Implid],
//
//     [0x98, OpType::TYA, 1, 2, AddressMode::Implid],
// ];

// struct Opcodes<'a> {
//     table: HashMap<u8, &'a OpCodeEntry<'a>>,
// }
//
// impl<'a> Opcodes<'a> {
//     fn new() -> Self {
//
//         let table = HashMap::new();
//         for entry in opcode_table.into_iter() {
//             table[&entry.opcode] = entry;
//         }
//
//         Opcodes{table: table}
//     }
//
//     fn entry(&self, opcode: u8) -> OpCodeEntry {
//         self.table[opcode]
//     }
// }
