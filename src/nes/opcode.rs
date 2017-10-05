//
// copy from https://github.com/amaiorano/nes-disasm
//
//
use std::collections::HashMap;

enum AddressMode {
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

enum OpType {
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


struct OpCodeEntry {
    opcode: u8,
    optype: OpType,
    bytes: u8,
    cycle: u8,
    address_mode: AddressMode,
}

struct Opcodes {
    table: HashMap<u16, OpCodeEntry>,
}

impl Opcodes {
    fn new() -> Self {
        let mut opcode_table = [
            OpCodeEntry{opcode: 0x69, optype: OpType::ADC, bytes: 2, cycle: 2, address_mode: AddressMode::Immedt},
            OpCodeEntry{opcode: 0x65, optype: OpType::ADC, bytes: 2, cycle: 3, address_mode: AddressMode::ZeroPg},
            OpCodeEntry{opcode: 0x75, optype: OpType::ADC, bytes: 2, cycle: 4, address_mode: AddressMode::ZPIdxX},
            OpCodeEntry{opcode: 0x6D, optype: OpType::ADC, bytes: 3, cycle: 4, address_mode: AddressMode::Absolu},
            OpCodeEntry{opcode: 0x7D, optype: OpType::ADC, bytes: 3, cycle: 4, address_mode: AddressMode::AbIdxX},
            OpCodeEntry{opcode: 0x79, optype: OpType::ADC, bytes: 3, cycle: 4, address_mode: AddressMode::AbIdxY},
            OpCodeEntry{opcode: 0x61, optype: OpType::ADC, bytes: 2, cycle: 6, address_mode: AddressMode::IdxInd},
            OpCodeEntry{opcode: 0x71, optype: OpType::ADC, bytes: 2, cycle: 5, address_mode: AddressMode::IndIdx},

            OpCodeEntry{opcode: 0x29, optype: OpType::AND, bytes: 2, cycle: 2, address_mode: AddressMode::Immedt},
            OpCodeEntry{opcode: 0x25, optype: OpType::AND, bytes: 2, cycle: 3, address_mode: AddressMode::ZeroPg},
            OpCodeEntry{opcode: 0x35, optype: OpType::AND, bytes: 2, cycle: 4, address_mode: AddressMode::ZPIdxX},
            OpCodeEntry{opcode: 0x2D, optype: OpType::AND, bytes: 3, cycle: 4, address_mode: AddressMode::Absolu},
            OpCodeEntry{opcode: 0x3D, optype: OpType::AND, bytes: 3, cycle: 4, address_mode: AddressMode::AbIdxX},
            OpCodeEntry{opcode: 0x39, optype: OpType::AND, bytes: 3, cycle: 4, address_mode: AddressMode::AbIdxY},
            OpCodeEntry{opcode: 0x21, optype: OpType::AND, bytes: 2, cycle: 6, address_mode: AddressMode::IdxInd},
            OpCodeEntry{opcode: 0x31, optype: OpType::AND, bytes: 2, cycle: 5, address_mode: AddressMode::IndIdx},

            OpCodeEntry{opcode: 0x0A, optype: OpType::ASL, bytes: 1, cycle: 2, address_mode: AddressMode::Accumu},
            OpCodeEntry{opcode: 0x06, optype: OpType::ASL, bytes: 2, cycle: 5, address_mode: AddressMode::ZeroPg},
            OpCodeEntry{opcode: 0x16, optype: OpType::ASL, bytes: 2, cycle: 6, address_mode: AddressMode::ZPIdxX},
            OpCodeEntry{opcode: 0x0E, optype: OpType::ASL, bytes: 3, cycle: 6, address_mode: AddressMode::Absolu},
            OpCodeEntry{opcode: 0x1E, optype: OpType::ASL, bytes: 3, cycle: 7, address_mode: AddressMode::AbIdxX},

            OpCodeEntry{opcode: 0x90, optype: OpType::BCC, bytes: 2, cycle: 2, address_mode: AddressMode::Relatv},

            OpCodeEntry{opcode: 0xB0, optype: OpType::BCS, bytes: 2, cycle: 2, address_mode: AddressMode::Relatv},

            OpCodeEntry{opcode: 0xF0, optype: OpType::BEQ, bytes: 2, cycle: 2, address_mode: AddressMode::Relatv},

            OpCodeEntry{opcode: 0x24, optype: OpType::BIT, bytes: 2, cycle: 3, address_mode: AddressMode::ZeroPg},
            OpCodeEntry{opcode: 0x2C, optype: OpType::BIT, bytes: 3, cycle: 4, address_mode: AddressMode::Absolu},

            OpCodeEntry{opcode: 0x30, optype: OpType::BMI, bytes: 2, cycle: 2, address_mode: AddressMode::Relatv},

            OpCodeEntry{opcode: 0xD0, optype: OpType::BNE, bytes: 2, cycle: 2, address_mode: AddressMode::Relatv},

            OpCodeEntry{opcode: 0x10, optype: OpType::BPL, bytes: 2, cycle: 2, address_mode: AddressMode::Relatv},

            OpCodeEntry{opcode: 0x00, optype: OpType::BRK, bytes: 1, cycle: 7, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0x50, optype: OpType::BVC, bytes: 2, cycle: 2, address_mode: AddressMode::Relatv},

            OpCodeEntry{opcode: 0x70, optype: OpType::BVS, bytes: 2, cycle: 2, address_mode: AddressMode::Relatv},

            OpCodeEntry{opcode: 0x18, optype: OpType::CLC, bytes: 1, cycle: 2, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0xD8, optype: OpType::CLD, bytes: 1, cycle: 2, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0x58, optype: OpType::CLI, bytes: 1, cycle: 2, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0xB8, optype: OpType::CLV, bytes: 1, cycle: 2, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0xC9, optype: OpType::CMP, bytes: 2, cycle: 2, address_mode: AddressMode::Immedt},
            OpCodeEntry{opcode: 0xC5, optype: OpType::CMP, bytes: 2, cycle: 3, address_mode: AddressMode::ZeroPg},
            OpCodeEntry{opcode: 0xD5, optype: OpType::CMP, bytes: 2, cycle: 4, address_mode: AddressMode::ZPIdxX},
            OpCodeEntry{opcode: 0xCD, optype: OpType::CMP, bytes: 3, cycle: 4, address_mode: AddressMode::Absolu},
            OpCodeEntry{opcode: 0xDD, optype: OpType::CMP, bytes: 3, cycle: 4, address_mode: AddressMode::AbIdxX},
            OpCodeEntry{opcode: 0xD9, optype: OpType::CMP, bytes: 3, cycle: 4, address_mode: AddressMode::AbIdxY},
            OpCodeEntry{opcode: 0xC1, optype: OpType::CMP, bytes: 2, cycle: 6, address_mode: AddressMode::IdxInd},
            OpCodeEntry{opcode: 0xD1, optype: OpType::CMP, bytes: 2, cycle: 5, address_mode: AddressMode::IndIdx},

            OpCodeEntry{opcode: 0xE0, optype: OpType::CPX, bytes: 2, cycle: 2, address_mode: AddressMode::Immedt},
            OpCodeEntry{opcode: 0xE4, optype: OpType::CPX, bytes: 2, cycle: 3, address_mode: AddressMode::ZeroPg},
            OpCodeEntry{opcode: 0xEC, optype: OpType::CPX, bytes: 3, cycle: 4, address_mode: AddressMode::Absolu},

            OpCodeEntry{opcode: 0xC0, optype: OpType::CPY, bytes: 2, cycle: 2, address_mode: AddressMode::Immedt},
            OpCodeEntry{opcode: 0xC4, optype: OpType::CPY, bytes: 2, cycle: 3, address_mode: AddressMode::ZeroPg},
            OpCodeEntry{opcode: 0xCC, optype: OpType::CPY, bytes: 3, cycle: 4, address_mode: AddressMode::Absolu},

            OpCodeEntry{opcode: 0xC6, optype: OpType::DEC, bytes: 2, cycle: 5, address_mode: AddressMode::ZeroPg},
            OpCodeEntry{opcode: 0xD6, optype: OpType::DEC, bytes: 2, cycle: 6, address_mode: AddressMode::ZPIdxX},
            OpCodeEntry{opcode: 0xCE, optype: OpType::DEC, bytes: 3, cycle: 6, address_mode: AddressMode::Absolu},
            OpCodeEntry{opcode: 0xDE, optype: OpType::DEC, bytes: 3, cycle: 7, address_mode: AddressMode::AbIdxX},

            OpCodeEntry{opcode: 0xCA, optype: OpType::DEX, bytes: 1, cycle: 2, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0x88, optype: OpType::DEY, bytes: 1, cycle: 2, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0x49, optype: OpType::EOR, bytes: 2, cycle: 2, address_mode: AddressMode::Immedt},
            OpCodeEntry{opcode: 0x45, optype: OpType::EOR, bytes: 2, cycle: 3, address_mode: AddressMode::ZeroPg},
            OpCodeEntry{opcode: 0x55, optype: OpType::EOR, bytes: 2, cycle: 4, address_mode: AddressMode::ZPIdxX},
            OpCodeEntry{opcode: 0x4D, optype: OpType::EOR, bytes: 3, cycle: 4, address_mode: AddressMode::Absolu},
            OpCodeEntry{opcode: 0x5D, optype: OpType::EOR, bytes: 3, cycle: 4, address_mode: AddressMode::AbIdxX},
            OpCodeEntry{opcode: 0x59, optype: OpType::EOR, bytes: 3, cycle: 4, address_mode: AddressMode::AbIdxY},
            OpCodeEntry{opcode: 0x41, optype: OpType::EOR, bytes: 2, cycle: 6, address_mode: AddressMode::IdxInd},
            OpCodeEntry{opcode: 0x51, optype: OpType::EOR, bytes: 2, cycle: 5, address_mode: AddressMode::IndIdx},

            OpCodeEntry{opcode: 0xE6, optype: OpType::INC, bytes: 2, cycle: 5, address_mode: AddressMode::ZeroPg},
            OpCodeEntry{opcode: 0xF6, optype: OpType::INC, bytes: 2, cycle: 6, address_mode: AddressMode::ZPIdxX},
            OpCodeEntry{opcode: 0xEE, optype: OpType::INC, bytes: 3, cycle: 6, address_mode: AddressMode::Absolu},
            OpCodeEntry{opcode: 0xFE, optype: OpType::INC, bytes: 3, cycle: 7, address_mode: AddressMode::AbIdxX},

            OpCodeEntry{opcode: 0xE8, optype: OpType::INX, bytes: 1, cycle: 2, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0xC8, optype: OpType::INY, bytes: 1, cycle: 2, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0x4C, optype: OpType::JMP, bytes: 3, cycle: 3, address_mode: AddressMode::Absolu},
            OpCodeEntry{opcode: 0x6C, optype: OpType::JMP, bytes: 3, cycle: 5, address_mode: AddressMode::Indrct},

            OpCodeEntry{opcode: 0x20, optype: OpType::JSR, bytes: 3, cycle: 6, address_mode: AddressMode::Absolu},

            OpCodeEntry{opcode: 0xA9, optype: OpType::LDA, bytes: 2, cycle: 2, address_mode: AddressMode::Immedt},
            OpCodeEntry{opcode: 0xA5, optype: OpType::LDA, bytes: 2, cycle: 3, address_mode: AddressMode::ZeroPg},
            OpCodeEntry{opcode: 0xB5, optype: OpType::LDA, bytes: 2, cycle: 4, address_mode: AddressMode::ZPIdxX},
            OpCodeEntry{opcode: 0xAD, optype: OpType::LDA, bytes: 3, cycle: 4, address_mode: AddressMode::Absolu},
            OpCodeEntry{opcode: 0xBD, optype: OpType::LDA, bytes: 3, cycle: 4, address_mode: AddressMode::AbIdxX},
            OpCodeEntry{opcode: 0xB9, optype: OpType::LDA, bytes: 3, cycle: 4, address_mode: AddressMode::AbIdxY},
            OpCodeEntry{opcode: 0xA1, optype: OpType::LDA, bytes: 2, cycle: 6, address_mode: AddressMode::IdxInd},
            OpCodeEntry{opcode: 0xB1, optype: OpType::LDA, bytes: 2, cycle: 5, address_mode: AddressMode::IndIdx},

            OpCodeEntry{opcode: 0xA2, optype: OpType::LDX, bytes: 2, cycle: 2, address_mode: AddressMode::Immedt},
            OpCodeEntry{opcode: 0xA6, optype: OpType::LDX, bytes: 2, cycle: 3, address_mode: AddressMode::ZeroPg},
            OpCodeEntry{opcode: 0xB6, optype: OpType::LDX, bytes: 2, cycle: 4, address_mode: AddressMode::ZPIdxY},
            OpCodeEntry{opcode: 0xAE, optype: OpType::LDX, bytes: 3, cycle: 4, address_mode: AddressMode::Absolu},
            OpCodeEntry{opcode: 0xBE, optype: OpType::LDX, bytes: 3, cycle: 4, address_mode: AddressMode::AbIdxY},

            OpCodeEntry{opcode: 0xA0, optype: OpType::LDY, bytes: 2, cycle: 2, address_mode: AddressMode::Immedt},
            OpCodeEntry{opcode: 0xA4, optype: OpType::LDY, bytes: 2, cycle: 3, address_mode: AddressMode::ZeroPg},
            OpCodeEntry{opcode: 0xB4, optype: OpType::LDY, bytes: 2, cycle: 4, address_mode: AddressMode::ZPIdxX},
            OpCodeEntry{opcode: 0xAC, optype: OpType::LDY, bytes: 3, cycle: 4, address_mode: AddressMode::Absolu},
            OpCodeEntry{opcode: 0xBC, optype: OpType::LDY, bytes: 3, cycle: 4, address_mode: AddressMode::AbIdxX},

            OpCodeEntry{opcode: 0x4A, optype: OpType::LSR, bytes: 1, cycle: 2, address_mode: AddressMode::Accumu},
            OpCodeEntry{opcode: 0x46, optype: OpType::LSR, bytes: 2, cycle: 5, address_mode: AddressMode::ZeroPg},
            OpCodeEntry{opcode: 0x56, optype: OpType::LSR, bytes: 2, cycle: 6, address_mode: AddressMode::ZPIdxX},
            OpCodeEntry{opcode: 0x4E, optype: OpType::LSR, bytes: 3, cycle: 6, address_mode: AddressMode::Absolu},
            OpCodeEntry{opcode: 0x5E, optype: OpType::LSR, bytes: 3, cycle: 7, address_mode: AddressMode::AbIdxX},

            OpCodeEntry{opcode: 0xEA, optype: OpType::NOP, bytes: 1, cycle: 2, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0x09, optype: OpType::ORA, bytes: 2, cycle: 2, address_mode: AddressMode::Immedt},
            OpCodeEntry{opcode: 0x05, optype: OpType::ORA, bytes: 2, cycle: 3, address_mode: AddressMode::ZeroPg},
            OpCodeEntry{opcode: 0x15, optype: OpType::ORA, bytes: 2, cycle: 4, address_mode: AddressMode::ZPIdxX},
            OpCodeEntry{opcode: 0x0D, optype: OpType::ORA, bytes: 3, cycle: 4, address_mode: AddressMode::Absolu},
            OpCodeEntry{opcode: 0x1D, optype: OpType::ORA, bytes: 3, cycle: 4, address_mode: AddressMode::AbIdxX},
            OpCodeEntry{opcode: 0x19, optype: OpType::ORA, bytes: 3, cycle: 4, address_mode: AddressMode::AbIdxY},
            OpCodeEntry{opcode: 0x01, optype: OpType::ORA, bytes: 2, cycle: 6, address_mode: AddressMode::IdxInd},
            OpCodeEntry{opcode: 0x11, optype: OpType::ORA, bytes: 2, cycle: 5, address_mode: AddressMode::IndIdx},

            OpCodeEntry{opcode: 0x48, optype: OpType::PHA, bytes: 1, cycle: 3, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0x08, optype: OpType::PHP, bytes: 1, cycle: 3, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0x68, optype: OpType::PLA, bytes: 1, cycle: 4, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0x28, optype: OpType::PLP, bytes: 1, cycle: 4, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0x2A, optype: OpType::ROL, bytes: 1, cycle: 2, address_mode: AddressMode::Accumu},
            OpCodeEntry{opcode: 0x26, optype: OpType::ROL, bytes: 2, cycle: 5, address_mode: AddressMode::ZeroPg},
            OpCodeEntry{opcode: 0x36, optype: OpType::ROL, bytes: 2, cycle: 6, address_mode: AddressMode::ZPIdxX},
            OpCodeEntry{opcode: 0x2E, optype: OpType::ROL, bytes: 3, cycle: 6, address_mode: AddressMode::Absolu},
            OpCodeEntry{opcode: 0x3E, optype: OpType::ROL, bytes: 3, cycle: 7, address_mode: AddressMode::AbIdxX},

            OpCodeEntry{opcode: 0x6A, optype: OpType::ROR, bytes: 1, cycle: 2, address_mode: AddressMode::Accumu},
            OpCodeEntry{opcode: 0x66, optype: OpType::ROR, bytes: 2, cycle: 5, address_mode: AddressMode::ZeroPg},
            OpCodeEntry{opcode: 0x76, optype: OpType::ROR, bytes: 2, cycle: 6, address_mode: AddressMode::ZPIdxX},
            OpCodeEntry{opcode: 0x6E, optype: OpType::ROR, bytes: 3, cycle: 6, address_mode: AddressMode::Absolu},
            OpCodeEntry{opcode: 0x7E, optype: OpType::ROR, bytes: 3, cycle: 7, address_mode: AddressMode::AbIdxX},

            OpCodeEntry{opcode: 0x40, optype: OpType::RTI, bytes: 1, cycle: 6, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0x60, optype: OpType::RTS, bytes: 1, cycle: 6, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0xE9, optype: OpType::SBC, bytes: 2, cycle: 2, address_mode: AddressMode::Immedt},
            OpCodeEntry{opcode: 0xE5, optype: OpType::SBC, bytes: 2, cycle: 3, address_mode: AddressMode::ZeroPg},
            OpCodeEntry{opcode: 0xF5, optype: OpType::SBC, bytes: 2, cycle: 4, address_mode: AddressMode::ZPIdxX},
            OpCodeEntry{opcode: 0xED, optype: OpType::SBC, bytes: 3, cycle: 4, address_mode: AddressMode::Absolu},
            OpCodeEntry{opcode: 0xFD, optype: OpType::SBC, bytes: 3, cycle: 4, address_mode: AddressMode::AbIdxX},
            OpCodeEntry{opcode: 0xF9, optype: OpType::SBC, bytes: 3, cycle: 4, address_mode: AddressMode::AbIdxY},
            OpCodeEntry{opcode: 0xE1, optype: OpType::SBC, bytes: 2, cycle: 6, address_mode: AddressMode::IdxInd},
            OpCodeEntry{opcode: 0xF1, optype: OpType::SBC, bytes: 2, cycle: 5, address_mode: AddressMode::IndIdx},

            OpCodeEntry{opcode: 0x38, optype: OpType::SEC, bytes: 1, cycle: 2, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0xF8, optype: OpType::SED, bytes: 1, cycle: 2, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0x78, optype: OpType::SEI, bytes: 1, cycle: 2, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0x85, optype: OpType::STA, bytes: 2, cycle: 3, address_mode: AddressMode::ZeroPg},
            OpCodeEntry{opcode: 0x95, optype: OpType::STA, bytes: 2, cycle: 4, address_mode: AddressMode::ZPIdxX},
            OpCodeEntry{opcode: 0x8D, optype: OpType::STA, bytes: 3, cycle: 4, address_mode: AddressMode::Absolu},
            OpCodeEntry{opcode: 0x9D, optype: OpType::STA, bytes: 3, cycle: 5, address_mode: AddressMode::AbIdxX},
            OpCodeEntry{opcode: 0x99, optype: OpType::STA, bytes: 3, cycle: 5, address_mode: AddressMode::AbIdxY},
            OpCodeEntry{opcode: 0x81, optype: OpType::STA, bytes: 2, cycle: 6, address_mode: AddressMode::IdxInd},
            OpCodeEntry{opcode: 0x91, optype: OpType::STA, bytes: 2, cycle: 6, address_mode: AddressMode::IndIdx},

            OpCodeEntry{opcode: 0x86, optype: OpType::STX, bytes: 2, cycle: 3, address_mode: AddressMode::ZeroPg},
            OpCodeEntry{opcode: 0x96, optype: OpType::STX, bytes: 2, cycle: 4, address_mode: AddressMode::ZPIdxY},
            OpCodeEntry{opcode: 0x8E, optype: OpType::STX, bytes: 3, cycle: 4, address_mode: AddressMode::Absolu},

            OpCodeEntry{opcode: 0x84, optype: OpType::STY, bytes: 2, cycle: 3, address_mode: AddressMode::ZeroPg},
            OpCodeEntry{opcode: 0x94, optype: OpType::STY, bytes: 2, cycle: 4, address_mode: AddressMode::ZPIdxX},
            OpCodeEntry{opcode: 0x8C, optype: OpType::STY, bytes: 3, cycle: 4, address_mode: AddressMode::Absolu},

            OpCodeEntry{opcode: 0xAA, optype: OpType::TAX, bytes: 1, cycle: 2, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0xA8, optype: OpType::TAY, bytes: 1, cycle: 2, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0xBA, optype: OpType::TSX, bytes: 1, cycle: 2, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0x8A, optype: OpType::TXA, bytes: 1, cycle: 2, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0x9A, optype: OpType::TXS, bytes: 1, cycle: 2, address_mode: AddressMode::Implid},

            OpCodeEntry{opcode: 0x98, optype: OpType::TYA, bytes: 1, cycle: 2, address_mode: AddressMode::Implid},
        ];

        let mut table = HashMap::new();
        for entry in opcode_table.into_iter() {
            table[&entry.opcode] = entry;
        }

        Opcodes{table: table}
    }

    fn entry(&self, opcode: u8) -> OpCodeEntry {
        self.table[opcode as usize]
    }
}
