mod addressing_mode;

use nes::cpu::addressing_mode::*;
use nes::mbc::Mbc;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct Cpu {
    pub a: u8,      // accumulator
    pub x: u8,      // index register(X)
    pub y: u8,      // index register(Y)
    pub pc: u16,    // program counter
    pub s: u8,      // stack pointer
    pub p: u8,      // processor status register
    pub mbc: Rc<RefCell<Box<Mbc>>>,
    pub cycle: u64,
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

macro_rules !inc {
    ($self:ident, $target:expr) => {
        {
            let target = $target as u16;
            let result = target.wrapping_add(1);

            $self.set_negative_flag(result as u8);
            $self.set_zero_flag(result as u8);
            $target = result as u8;
        }

    }
}

macro_rules !dec {
    ($self:ident, $target:expr) => {
        {
            let target = $target as u16;
            let result = target.wrapping_sub(1);

            $self.set_negative_flag(result as u8);
            $self.set_zero_flag(result as u8);
            $target = result as u8;
        }

    }
}


macro_rules !cmp {
    ($self:ident, $target:expr, $value:expr) => {
        {
            let target = $target;
            let value = $value;
            let (result, _) = target.overflowing_sub(value);
            $self.set_flag(FLAG_CRY, target >= value);
            $self.set_zero_flag(result);
            $self.set_negative_flag(result);
        }

    }
}

macro_rules !branch {
    ($self:ident, $name:expr, $flag:expr, $addr:expr, $result:expr) => {
        {
            info!("opcode:{}", $name);
            if $self.get_flag($flag) == $result {
                let offset = $addr.read($self) as i8 as i32;
                let jump_addr = (($self.pc as i32) + offset) as u16 + 1;
                info!("{} Jump pc:{:x} -> {:x}", $name, $self.pc, jump_addr);
                $self.pc = jump_addr;
                true
            } else {
                $self.pc += $addr.length();
                false
            }
        }
    }
}

macro_rules !instruction {
    ($self:ident,
     $addressing_mode:ident,
     $action:ident,
     $cycle:expr,
     $page_cycle:expr) => {
         {
            let m = $self.$addressing_mode();
            $self.cycle = $self.cycle.wrapping_add($cycle);
            if $page_cycle != 0 && m.is_page_crossed() {
                $self.cycle = $self.cycle.wrapping_add($page_cycle);
            }

            $self.$action(m)
         }
     }
}


impl Cpu {
    pub fn new(mbc: Rc<RefCell<Box<Mbc>>>) -> Self {
        Cpu {
            a: 0, x: 0, y: 0,
            pc: 0x8000, s: 0xFF, p: 0,
            mbc: mbc,
            cycle: 0,
            }
    }

    pub fn setup(&mut self) {
        let pc = self.mbc.borrow().initial_pc();
        info!("setup() pc:{:x} -> {:x}", self.pc, pc);
        self.set_flag(FLAG_CRY, false);
        self.set_flag(FLAG_ZER, false);
        self.set_flag(FLAG_IRQ, false);
        self.set_flag(FLAG_DEC, false);
        self.set_flag(FLAG_BRK, false);
        self.set_flag(FLAG_RSV, false);
        self.set_flag(FLAG_OVF, false);
        self.set_flag(FLAG_NEG, false);
        self.pc = pc;
    }

    fn brk<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:BRK");
        self.do_irq("irq");
        false
    }

    fn kil<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:KIL");
        self.pc += addr.length();
        unimplemented!();
    }

    fn slo<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:SLO");
        self.pc += addr.length();
        unimplemented!();
    }
    fn nop<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:NOP");
        self.pc += addr.length();
        true
    }
    fn anc<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:ANC");
        self.pc += addr.length();
        unimplemented!();
    }
    fn clc<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:CLC");
        self.set_flag(FLAG_CRY, false);
        self.pc += addr.length();
        true
    }
    fn sec<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:SEC");
        self.set_flag(FLAG_CRY, true);
        self.pc += addr.length();
        true
    }
    fn cli<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:CLI");
        self.set_flag(FLAG_IRQ, false);
        self.pc += addr.length();
        true
    }
    fn sei<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:SEI");
        self.set_flag(FLAG_IRQ, true);
        self.pc += addr.length();
        true
    }
    fn clv<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:CLV");
        self.set_flag(FLAG_OVF, false);
        self.pc += addr.length();
        true
    }
    fn cld<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:CLD");
        self.set_flag(FLAG_DEC, false);
        self.pc += addr.length();
        true
    }
    fn sed<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:SED");
        self.set_flag(FLAG_DEC, true);
        self.pc += addr.length();
        true
    }

    // subrouting
    fn jmp<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:JMP");
        let jump_addr = addr.read16_addr(self);
        info!("jump_addr:{:x}", jump_addr);
        self.pc = jump_addr;
        false
    }
    fn jsr<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:JSR");
        self.set_flag(FLAG_DEC, true);
        let pc = self.pc;
        self.push16(pc + addr.length() - 1);
        self.pc = addr.read16_addr(self);
        false
    }
    fn rts<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:RTS");
        let return_addr = self.pop16();
        info!("self.pc({:x}) => {:x}", self.pc, return_addr);
        self.pc = return_addr + 1;
        false
    }
    fn rti<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:RTI");
        self.p = self.pop();
        let return_addr = self.pop16();
        info!("self.pc({:x}) => {:x}", self.pc, return_addr);
        self.pc = return_addr;
        false
    }

    fn do_irq(&mut self, irq_name: &str) {
        let pc = self.pc;
        self.push16(pc);
        let p = self.p;
        self.push(p);
        self.set_flag(FLAG_IRQ, true);
        self.pc = self.vector(irq_name);
    }


    // copy operator
    fn sta<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:STA");
        let a = self.a;
        addr.write(self, a);
        self.pc += addr.length();
        true
    }
    fn stx<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:STX");
        let x = self.x;
        addr.write(self, x);
        self.pc += addr.length();
        true
    }
    fn sty<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:STY");
        let y = self.y;
        addr.write(self, y);
        self.pc += addr.length();
        true
    }
    fn lda<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:LDA");
        let value = addr.read(self);
        self.a = value;
        self.set_negative_flag(value);
        self.set_zero_flag(value);
        self.pc += addr.length();
        true
    }
    fn ldx<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:LDX");
        let value = addr.read(self);
        self.x = value;
        self.set_negative_flag(value);
        self.set_zero_flag(value);
        self.pc += addr.length();
        true
    }
    fn ldy<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:LDY");
        let value = addr.read(self);
        self.y = value;
        self.set_negative_flag(value);
        self.set_zero_flag(value);
        self.pc += addr.length();
        true
    }
    fn tax<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:TAX");
        let value = self.a;
        self.x = value;
        self.set_negative_flag(value);
        self.set_zero_flag(value);
        self.pc += addr.length();
        true
    }
    fn tay<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:TAY");
        let value = self.a;
        self.y = value;
        self.set_negative_flag(value);
        self.set_zero_flag(value);
        self.pc += addr.length();
        true
    }
    fn tsx<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:TSX");
        let value = self.s;
        self.x = value;
        self.set_negative_flag(value);
        self.set_zero_flag(value);
        self.pc += addr.length();
        true
    }
    fn txa<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:TXA");
        let value = self.x;
        self.a = value;
        self.set_negative_flag(value);
        self.set_zero_flag(value);
        self.pc += addr.length();
        true
    }
    fn txs<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:TXS");
        let value = self.x;
        self.s = value;
        self.pc += addr.length();
        true
    }
    fn tya<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:TYA");
        let value = self.y;
        self.a = value;
        self.set_negative_flag(value);
        self.set_zero_flag(value);
        self.pc += addr.length();
        true
    }

    // caluculate oprators
    fn adc<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:ADC");
        let value = addr.read(self) as u16;
        let target = self.a as u16;
        let mut result = target.wrapping_add(value);
        if self.get_flag(FLAG_CRY) {
            result = result.wrapping_add(1);
        }
        self.set_negative_flag(result as u8);
        self.set_zero_flag(result as u8);
        self.set_flag(FLAG_CRY, (result & 0x0100) != 0);
        self.set_flag(FLAG_OVF,
                      (target ^ value) & 0x80 == 0 &&
                      (target ^ result) & 0x80 == 0x80);
        self.a = result as u8;

        self.pc += addr.length();
        true
    }
    fn and<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:AND");
        let result = self.a & addr.read(self);
        self.set_negative_flag(result);
        self.set_zero_flag(result);
        self.a = result;
        self.pc += addr.length();
        true
    }
    fn asl<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:ASL");
        let target = addr.read(self);
        let carry = (target & 0x80) == 0x80;
        let result = target.wrapping_shl(1);
        self.set_flag(FLAG_CRY, carry);
        self.set_negative_flag(result);
        self.set_zero_flag(result);
        addr.write(self, result);
        self.pc += addr.length();
        true
    }
    fn bit<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:BIT");
        let operand = addr.read(self);
        let result = self.a & operand;
        self.set_zero_flag(result);
        self.set_negative_flag(operand);
        self.set_flag(FLAG_OVF, (operand & 0x40) == 0x40);
        self.pc += addr.length();
        true
    }
    fn cmp<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:CMP");
        let v = addr.read(self);
        info!("self.a={:x}, addr.read={:x}", self.a, v);
        cmp!(self, self.a, v);
        self.pc += addr.length();
        true
    }
    fn cpx<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:CPX");
        cmp!(self, self.x, addr.read(self));
        self.pc += addr.length();
        true
    }
    fn cpy<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:CPY");
        cmp!(self, self.y, addr.read(self));
        self.pc += addr.length();
        true
    }
    fn dec<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:DEC");
        let mut value = addr.read(self);
        dec!(self, value);
        addr.write(self, value);
        self.pc += addr.length();
        true
    }
    fn dex<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:DEX");
        dec!(self, self.x);
        self.pc += addr.length();
        true
    }
    fn dey<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:DEY");
        dec!(self, self.y);
        self.pc += addr.length();
        true
    }
    fn eor<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:EOR");
        let operand = addr.read(self);
        let result = operand ^ self.a;
        self.set_zero_flag(result);
        self.set_negative_flag(result);
        self.a = result;
        self.pc += addr.length();
        true
    }
    fn inc<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:INC");
        let mut value = addr.read(self);
        inc!(self, value);
        addr.write(self, value);
        self.pc += addr.length();
        true
    }
    fn inx<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:INX");
        inc!(self, self.x);
        self.pc += addr.length();
        true
    }
    fn iny<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:INY");
        inc!(self, self.y);
        self.pc += addr.length();
        true
    }
    fn lsr<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:LSR");
        let target = addr.read(self);
        let carry = (target & 0x01) == 0x01;
        let result = target.wrapping_shr(1);
        self.set_flag(FLAG_CRY, carry);
        self.set_zero_flag(result);
        self.set_negative_flag(result);
        addr.write(self, result);
        self.pc += addr.length();
        true
    }
    fn ora<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:ORA");
        let result = self.a | addr.read(self);
        self.set_negative_flag(result);
        self.set_zero_flag(result);
        self.a = result;
        self.pc += addr.length();
        true
    }
    fn rol<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:ROL");
        let data = addr.read(self);
        let carry = (data & 0x80) == 0x80;
        let mut result = data << 1;
        if self.get_flag(FLAG_CRY) {
            result |= 0x01;
        }

        self.set_flag(FLAG_CRY, carry);
        self.set_negative_flag(result);
        self.set_zero_flag(result);

        addr.write(self, result);
        self.pc += addr.length();
        true
    }
    fn ror<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:ROR");
        let data = addr.read(self);
        let carry = (data & 0x01) == 0x01;
        let mut result = data >> 1;
        if self.get_flag(FLAG_CRY) {
            result |= 0x80;
        }

        self.set_flag(FLAG_CRY, carry);
        self.set_negative_flag(result);
        self.set_zero_flag(result);

        addr.write(self, result);
        self.pc += addr.length();
        true
    }
    fn sre<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:SRE");
        let data = addr.read(self);
        self.set_flag(FLAG_CRY, (data & 0x01) != 0);
        addr.write(self, data >> 1);
        self.pc += addr.length();
        true
    }
    fn sbc<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:SBC");
        let value = addr.read(self) as u16;
        let target = self.a as u16;
        let mut result = target.wrapping_sub(value);
        if !self.get_flag(FLAG_CRY) {
            result = result.wrapping_sub(1);
        }
        self.set_zero_flag(result as u8);
        self.set_negative_flag(result as u8);
        self.set_flag(FLAG_CRY, (result & 0x0100) == 0);
        self.set_flag(FLAG_OVF,
                      (target ^ value) & 0x80 != 0 &&
                      (target ^ result) & 0x80 == 0x80);
        self.a = result as u8;
        self.pc += addr.length();
        true
    }

    // stack
    fn pha<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:PHA");
        let a = self.a;
        self.push(a);
        self.set_negative_flag(a);
        self.set_zero_flag(a);
        self.pc += addr.length();
        true
    }
    fn php<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:PHP");
        let p = self.p | FLAG_BRK | FLAG_RSV;
        self.push(p);
        self.pc += addr.length();
        true
    }
    fn pla<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:PLA");
        let a = self.pop();
        self.a = a;
        self.set_negative_flag(a);
        self.set_zero_flag(a);
        self.pc += addr.length();
        true
    }
    fn plp<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:PLP");
        self.p = self.pop();
        self.pc += addr.length();
        true
    }

    // branch
    fn bcc<T:AddressingMode>(&mut self, addr: T) -> bool {
        branch!(self, "BCC", FLAG_CRY, addr, false)
    }
    fn bcs<T:AddressingMode>(&mut self, addr: T) -> bool {
        branch!(self, "BCS", FLAG_CRY, addr, true)
    }
    fn beq<T:AddressingMode>(&mut self, addr: T) -> bool {
        branch!(self, "BEQ", FLAG_ZER, addr, true)
    }
    fn bmi<T:AddressingMode>(&mut self, addr: T) -> bool {
        branch!(self, "BMI", FLAG_NEG, addr, true)
    }
    fn bne<T:AddressingMode>(&mut self, addr: T) -> bool {
        branch!(self, "BNE", FLAG_ZER, addr, false)
    }
    fn bpl<T:AddressingMode>(&mut self, addr: T) -> bool {
        branch!(self, "BPL", FLAG_NEG, addr, false)
    }
    fn bvc<T:AddressingMode>(&mut self, addr: T) -> bool {
        branch!(self, "BVC", FLAG_OVF, addr, false)
    }
    fn bvs<T:AddressingMode>(&mut self, addr: T) -> bool {
        branch!(self, "BVS", FLAG_OVF, addr, true)
    }

    fn rla<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:RLA");
        unimplemented!();
    }
    fn rra<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:RRA");
        unimplemented!();
    }
    fn alr<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:ALR");
        unimplemented!();
    }
    fn arr<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:ARR");
        unimplemented!();
    }
    fn sax<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:SAX");
        unimplemented!();
    }
    fn say<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:SAY");
        unimplemented!();
    }
    fn xaa<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:XAA");
        unimplemented!();
    }
    fn ahx<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:AHX");
        unimplemented!();
    }
    fn tas<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:TAS");
        unimplemented!();
    }
    fn shx<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:SHX");
        unimplemented!();
    }
    fn shy<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:SHY");
        unimplemented!();
    }
    fn lax<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:LAX");
        unimplemented!();
    }
    fn las<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:LAS");
        unimplemented!();
    }
    fn dcp<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:DCP");
        unimplemented!();
    }
    fn axs<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:AXS");
        unimplemented!();
    }
    fn isc<T:AddressingMode>(&mut self, addr: T) -> bool {
        info!("opcode:ISC");
        unimplemented!();
    }

    // addressing mode
    fn implicit(&mut self) -> NoAccessAddressingMode {
        NoAccessAddressingMode{}
    }

    fn accumurator(&mut self) -> AccumuratorAddressingMode {
        AccumuratorAddressingMode::new()
    }

    fn indirect(&mut self) -> MemoryAddressingMode {
        let addr = self.read16(self.pc);
        let addr = self.read16(addr);
        MemoryAddressingMode::new(addr, 2)
    }

    fn indirectx(&mut self) -> MemoryAddressingMode {
        let operand = self.read(self.pc) as u16;
        let x = self.x as u16;
        let low_addr   = (operand + x) & 0xFF;
        let high_addr  = (operand + x + 1) & 0xFF;
        let low = self.read(low_addr) as u16;
        let high = (self.read(high_addr) as u16) << 8;

        MemoryAddressingMode::new(low + high, 1)
    }

    fn indirecty(&mut self) -> MemoryAddressingMode {
        let operand = self.read(self.pc) as u16;
        let y = self.y as u16;
        let low_addr  = operand;
        let high_addr = (operand + 1) & 0xFF;
        let low  = self.read(low_addr) as u16;
        let high = (self.read(high_addr) as u16) << 8;
        MemoryAddressingMode::new(low.wrapping_add(high).wrapping_add(y), 1)
    }

    fn zeropage(&mut self) -> MemoryAddressingMode {
        let addr = self.read(self.pc) as u16;
        MemoryAddressingMode::new(addr, 1)
    }

    fn zeropagex(&mut self) -> MemoryAddressingMode {
        let operand = self.read(self.pc);
        let addr = operand.wrapping_add(self.x) as u16;
        MemoryAddressingMode::new(addr, 1)
    }

    fn zeropagey(&mut self) -> MemoryAddressingMode {
        let operand = self.read(self.pc);
        let addr = operand.wrapping_add(self.y) as u16;
        MemoryAddressingMode::new(addr, 1)
    }

    fn immediate(&mut self) -> ImmediateAddressingMode {
        ImmediateAddressingMode::new(self.read(self.pc), 1)
    }

    fn absolute(&mut self) -> MemoryAddressingMode {
        let addr = self.read16(self.pc);
        MemoryAddressingMode::new(addr, 2)
    }

    fn absolutex(&mut self) -> MemoryAddressingMode {
        let addr = self.read16(self.pc).wrapping_add(self.x as u16);
        MemoryAddressingMode::new(addr, 2)
    }

    fn absolutey(&mut self) -> MemoryAddressingMode {
        let addr = self.read16(self.pc).wrapping_add(self.y as u16);
        MemoryAddressingMode::new(addr, 2)
    }

    #[inline]
    pub fn cycle(&self) -> u64 {
        self.cycle
    }

    pub fn tick(&mut self) {
        self.debug();

        if self.process_nmi() {
            return;
        }

        let before_status = self.clone();

        let opcode = self.read(self.pc);
        self.pc += 1;

        self.process_opcode(opcode);

        self.print_diff(before_status);
    }

    fn process_opcode(&mut self, opcode: u8) {
        let cont = match opcode {
            // 0x00 => {let m = self.implicit();    self.brk(m) },
            0x00 => { instruction!(self, implicit,    brk, 7, 0) },
            0x01 => { instruction!(self, indirectx,   ora, 6, 0) },
            0x02 => { instruction!(self, immediate,   kil, 2, 0) },
            0x03 => { instruction!(self, indirectx,   slo, 8, 0) },
            0x04 => { instruction!(self, zeropage,    nop, 3, 0) },
            0x05 => { instruction!(self, zeropage,    ora, 3, 0) },
            0x06 => { instruction!(self, zeropage,    asl, 5, 0) },
            0x07 => { instruction!(self, zeropage,    slo, 5, 0) },
            0x08 => { instruction!(self, implicit,    php, 3, 0) },
            0x09 => { instruction!(self, immediate,   ora, 2, 0) },
            0x0A => { instruction!(self, accumurator, asl, 2, 0) },
            0x0B => { instruction!(self, immediate,   anc, 2, 0) },
            0x0C => { instruction!(self, absolute,    nop, 4, 0) },
            0x0D => { instruction!(self, absolute,    ora, 4, 0) },
            0x0E => { instruction!(self, absolute,    asl, 6, 0) },
            0x0F => { instruction!(self, absolute,    slo, 6, 0) },
            0x10 => { instruction!(self, immediate,   bpl, 2, 1) },
            0x11 => { instruction!(self, indirecty,   ora, 5, 1) },
            0x12 => { instruction!(self, immediate,   kil, 2, 0) },
            0x13 => { instruction!(self, indirecty,   slo, 8, 0) },
            0x14 => { instruction!(self, zeropagex,   nop, 4, 0) },
            0x15 => { instruction!(self, zeropagex,   ora, 4, 0) },
            0x16 => { instruction!(self, zeropagex,   asl, 6, 0) },
            0x17 => { instruction!(self, zeropagex,   slo, 6, 0) },
            0x18 => { instruction!(self, implicit,    clc, 2, 0) },
            0x19 => { instruction!(self, absolutey,   ora, 4, 1) },
            0x1A => { instruction!(self, immediate,   nop, 2, 0) },
            0x1B => { instruction!(self, absolutey,   slo, 7, 0) },
            0x1C => { instruction!(self, absolutex,   nop, 4, 1) },
            0x1D => { instruction!(self, absolutex,   ora, 4, 1) },
            0x1E => { instruction!(self, absolutex,   asl, 7, 0) },
            0x1F => { instruction!(self, absolutex,   slo, 7, 0) },
            0x20 => { instruction!(self, absolute,    jsr, 6, 0) },
            0x21 => { instruction!(self, indirectx,   and, 6, 0) },
            0x22 => { instruction!(self, immediate,   kil, 2, 0) },
            0x23 => { instruction!(self, indirectx,   rla, 8, 0) },
            0x24 => { instruction!(self, zeropage,    bit, 3, 0) },
            0x25 => { instruction!(self, zeropage,    and, 3, 0) },
            0x26 => { instruction!(self, zeropage,    rol, 5, 0) },
            0x27 => { instruction!(self, zeropage,    rla, 5, 0) },
            0x28 => { instruction!(self, implicit,    plp, 4, 0) },
            0x29 => { instruction!(self, immediate,   and, 2, 0) },
            0x2A => { instruction!(self, accumurator, rol, 2, 0) },
            0x2B => { instruction!(self, immediate,   anc, 2, 0) },
            0x2C => { instruction!(self, absolute,    bit, 4, 0) },
            0x2D => { instruction!(self, absolute,    and, 4, 0) },
            0x2E => { instruction!(self, absolute,    rol, 6, 0) },
            0x2F => { instruction!(self, absolute,    rla, 6, 0) },
            0x30 => { instruction!(self, immediate,   bmi, 2, 1) },
            0x31 => { instruction!(self, indirecty,   and, 5, 1) },
            0x32 => { instruction!(self, immediate,   kil, 2, 0) },
            0x33 => { instruction!(self, indirecty,   rla, 8, 0) },
            0x34 => { instruction!(self, zeropagex,   nop, 4, 0) },
            0x35 => { instruction!(self, zeropagex,   and, 4, 0) },
            0x36 => { instruction!(self, zeropagex,   rol, 6, 0) },
            0x37 => { instruction!(self, zeropagex,   rla, 6, 0) },
            0x38 => { instruction!(self, implicit,    sec, 2, 0) },
            0x39 => { instruction!(self, absolutey,   and, 4, 1) },
            0x3A => { instruction!(self, immediate,   nop, 2, 0) },
            0x3B => { instruction!(self, absolutey,   rla, 7, 0) },
            0x3C => { instruction!(self, absolutex,   nop, 4, 1) },
            0x3D => { instruction!(self, absolutex,   and, 4, 1) },
            0x3E => { instruction!(self, absolutex,   rol, 7, 0) },
            0x3F => { instruction!(self, absolutex,   rla, 7, 0) },
            0x40 => { instruction!(self, implicit,    rti, 6, 0) },
            0x41 => { instruction!(self, indirectx,   eor, 6, 0) },
            0x42 => { instruction!(self, immediate,   kil, 2, 0) },
            0x43 => { instruction!(self, indirectx,   sre, 8, 0) },
            0x44 => { instruction!(self, zeropage,    nop, 3, 0) },
            0x45 => { instruction!(self, zeropage,    eor, 3, 0) },
            0x46 => { instruction!(self, zeropage,    lsr, 5, 0) },
            0x47 => { instruction!(self, zeropage,    sre, 5, 0) },
            0x48 => { instruction!(self, implicit,    pha, 3, 0) },
            0x49 => { instruction!(self, immediate,   eor, 2, 0) },
            0x4A => { instruction!(self, accumurator, lsr, 2, 0) },
            0x4B => { instruction!(self, immediate,   alr, 2, 0) },
            0x4C => { instruction!(self, absolute,    jmp, 3, 0) },
            0x4D => { instruction!(self, absolute,    eor, 4, 0) },
            0x4E => { instruction!(self, absolute,    lsr, 6, 0) },
            0x4F => { instruction!(self, absolute,    sre, 6, 0) },
            0x50 => { instruction!(self, immediate,   bvc, 2, 1) },
            0x51 => { instruction!(self, indirecty,   eor, 5, 1) },
            0x52 => { instruction!(self, immediate,   kil, 2, 0) },
            0x53 => { instruction!(self, indirecty,   sre, 8, 0) },
            0x54 => { instruction!(self, zeropagex,   nop, 4, 0) },
            0x55 => { instruction!(self, zeropagex,   eor, 4, 0) },
            0x56 => { instruction!(self, zeropagex,   lsr, 6, 0) },
            0x57 => { instruction!(self, zeropagex,   sre, 6, 0) },
            0x58 => { instruction!(self, implicit,    cli, 2, 0) },
            0x59 => { instruction!(self, absolutey,   eor, 4, 1) },
            0x5A => { instruction!(self, immediate,   nop, 2, 0) },
            0x5B => { instruction!(self, absolutey,   sre, 7, 0) },
            0x5C => { instruction!(self, absolutex,   nop, 4, 1) },
            0x5D => { instruction!(self, absolutex,   eor, 4, 1) },
            0x5E => { instruction!(self, absolutex,   lsr, 7, 0) },
            0x5F => { instruction!(self, absolutex,   sre, 7, 0) },
            0x60 => { instruction!(self, implicit,    rts, 6, 0) },
            0x61 => { instruction!(self, indirectx,   adc, 6, 0) },
            0x62 => { instruction!(self, immediate,   kil, 2, 0) },
            0x63 => { instruction!(self, indirectx,   rra, 8, 0) },
            0x64 => { instruction!(self, zeropage,    nop, 3, 0) },
            0x65 => { instruction!(self, zeropage,    adc, 3, 0) },
            0x66 => { instruction!(self, zeropage,    ror, 5, 0) },
            0x67 => { instruction!(self, zeropage,    rra, 5, 0) },
            0x68 => { instruction!(self, implicit,    pla, 4, 0) },
            0x69 => { instruction!(self, immediate,   adc, 2, 0) },
            0x6A => { instruction!(self, accumurator, ror, 2, 0) },
            0x6B => { instruction!(self, immediate,   arr, 2, 0) },
            0x6C => { instruction!(self, indirect,    jmp, 5, 0) },
            0x6D => { instruction!(self, absolute,    adc, 4, 0) },
            0x6E => { instruction!(self, absolute,    ror, 6, 0) },
            0x6F => { instruction!(self, absolute,    rra, 6, 0) },
            0x70 => { instruction!(self, immediate,   bvs, 2, 1) },
            0x71 => { instruction!(self, indirecty,   adc, 5, 1) },
            0x72 => { instruction!(self, immediate,   kil, 2, 0) },
            0x73 => { instruction!(self, indirecty,   rra, 8, 0) },
            0x74 => { instruction!(self, zeropagex,   nop, 4, 0) },
            0x75 => { instruction!(self, zeropagex,   adc, 4, 0) },
            0x76 => { instruction!(self, zeropagex,   ror, 6, 0) },
            0x77 => { instruction!(self, zeropagex,   rra, 6, 0) },
            0x78 => { instruction!(self, implicit,    sei, 2, 0) },
            0x79 => { instruction!(self, absolutey,   adc, 4, 1) },
            0x7A => { instruction!(self, immediate,   nop, 2, 0) },
            0x7B => { instruction!(self, absolutey,   rra, 7, 0) },
            0x7C => { instruction!(self, absolutex,   nop, 4, 1) },
            0x7D => { instruction!(self, absolutex,   adc, 4, 1) },
            0x7E => { instruction!(self, absolutex,   ror, 7, 0) },
            0x7F => { instruction!(self, absolutex,   rra, 7, 0) },
            0x80 => { instruction!(self, immediate,   nop, 2, 0) },
            0x81 => { instruction!(self, indirectx,   sta, 6, 0) },
            0x82 => { instruction!(self, immediate,   nop, 2, 0) },
            0x83 => { instruction!(self, indirectx,   sax, 6, 0) },
            0x84 => { instruction!(self, zeropage,    sty, 3, 0) },
            0x85 => { instruction!(self, zeropage,    sta, 3, 0) },
            0x86 => { instruction!(self, zeropage,    stx, 3, 0) },
            0x87 => { instruction!(self, zeropage,    sax, 3, 0) },
            0x88 => { instruction!(self, implicit,    dey, 2, 0) },
            0x89 => { instruction!(self, immediate,   nop, 2, 0) },
            0x8A => { instruction!(self, implicit,    txa, 2, 0) },
            0x8B => { instruction!(self, immediate,   xaa, 2, 0) },
            0x8C => { instruction!(self, absolute,    sty, 4, 0) },
            0x8D => { instruction!(self, absolute,    sta, 4, 0) },
            0x8E => { instruction!(self, absolute,    stx, 4, 0) },
            0x8F => { instruction!(self, absolute,    sax, 4, 0) },
            0x90 => { instruction!(self, immediate,   bcc, 2, 1) },
            0x91 => { instruction!(self, indirecty,   sta, 6, 0) },
            0x92 => { instruction!(self, immediate,   kil, 2, 0) },
            0x93 => { instruction!(self, indirecty,   ahx, 6, 0) },
            0x94 => { instruction!(self, zeropagex,   sty, 4, 0) },
            0x95 => { instruction!(self, zeropagex,   sta, 4, 0) },
            0x96 => { instruction!(self, zeropagey,   stx, 4, 0) },
            0x97 => { instruction!(self, zeropagey,   sax, 4, 0) },
            0x98 => { instruction!(self, implicit,    tya, 2, 0) },
            0x99 => { instruction!(self, absolutey,   sta, 5, 0) },
            0x9A => { instruction!(self, implicit,    txs, 2, 0) },
            0x9B => { instruction!(self, immediate,   tas, 5, 0) },
            0x9C => { instruction!(self, absolutex,   shy, 5, 0) },
            0x9D => { instruction!(self, absolutex,   sta, 5, 0) },
            0x9E => { instruction!(self, absolutey,   shx, 5, 0) },
            0x9F => { instruction!(self, absolutey,   ahx, 5, 0) },
            0xA0 => { instruction!(self, immediate,   ldy, 2, 0) },
            0xA1 => { instruction!(self, indirectx,   lda, 6, 0) },
            0xA2 => { instruction!(self, immediate,   ldx, 2, 0) },
            0xA3 => { instruction!(self, indirectx,   lax, 6, 0) },
            0xA4 => { instruction!(self, zeropage,    ldy, 3, 0) },
            0xA5 => { instruction!(self, zeropage,    lda, 3, 0) },
            0xA6 => { instruction!(self, zeropage,    ldx, 3, 0) },
            0xA7 => { instruction!(self, zeropage,    lax, 3, 0) },
            0xA8 => { instruction!(self, implicit,    tay, 2, 0) },
            0xA9 => { instruction!(self, immediate,   lda, 2, 0) },
            0xAA => { instruction!(self, implicit,    tax, 2, 0) },
            0xAB => { instruction!(self, immediate,   lax, 2, 0) },
            0xAC => { instruction!(self, absolute,    ldy, 4, 0) },
            0xAD => { instruction!(self, absolute,    lda, 4, 0) },
            0xAE => { instruction!(self, absolute,    ldx, 4, 0) },
            0xAF => { instruction!(self, absolute,    lax, 4, 0) },
            0xB0 => { instruction!(self, immediate,   bcs, 2, 1) },
            0xB1 => { instruction!(self, indirecty,   lda, 5, 1) },
            0xB2 => { instruction!(self, immediate,   kil, 2, 0) },
            0xB3 => { instruction!(self, indirecty,   lax, 5, 1) },
            0xB4 => { instruction!(self, zeropagex,   ldy, 4, 0) },
            0xB5 => { instruction!(self, zeropagex,   lda, 4, 0) },
            0xB6 => { instruction!(self, zeropagey,   ldx, 4, 0) },
            0xB7 => { instruction!(self, zeropagey,   lax, 4, 0) },
            0xB8 => { instruction!(self, implicit,    clv, 2, 0) },
            0xB9 => { instruction!(self, absolutey,   lda, 4, 1) },
            0xBA => { instruction!(self, implicit,    tsx, 2, 0) },
            0xBB => { instruction!(self, absolutey,   las, 4, 1) },
            0xBC => { instruction!(self, absolutex,   ldy, 4, 1) },
            0xBD => { instruction!(self, absolutex,   lda, 4, 1) },
            0xBE => { instruction!(self, absolutey,   ldx, 4, 1) },
            0xBF => { instruction!(self, absolutey,   lax, 4, 1) },
            0xC0 => { instruction!(self, immediate,   cpy, 2, 0) },
            0xC1 => { instruction!(self, indirectx,   cmp, 6, 0) },
            0xC2 => { instruction!(self, immediate,   nop, 2, 0) },
            0xC3 => { instruction!(self, indirectx,   dcp, 8, 0) },
            0xC4 => { instruction!(self, zeropage,    cpy, 3, 0) },
            0xC5 => { instruction!(self, zeropage,    cmp, 3, 0) },
            0xC6 => { instruction!(self, zeropage,    dec, 5, 0) },
            0xC7 => { instruction!(self, zeropage,    dcp, 5, 0) },
            0xC8 => { instruction!(self, implicit,    iny, 2, 0) },
            0xC9 => { instruction!(self, immediate,   cmp, 2, 0) },
            0xCA => { instruction!(self, implicit,    dex, 2, 0) },
            0xCB => { instruction!(self, immediate,   axs, 2, 0) },
            0xCC => { instruction!(self, absolute,    cpy, 4, 0) },
            0xCD => { instruction!(self, absolute,    cmp, 4, 0) },
            0xCE => { instruction!(self, absolute,    dec, 6, 0) },
            0xCF => { instruction!(self, absolute,    dcp, 6, 0) },
            0xD0 => { instruction!(self, immediate,   bne, 2, 1) },
            0xD1 => { instruction!(self, indirecty,   cmp, 5, 1) },
            0xD2 => { instruction!(self, immediate,   kil, 2, 0) },
            0xD3 => { instruction!(self, indirecty,   dcp, 8, 0) },
            0xD4 => { instruction!(self, zeropagex,   nop, 4, 0) },
            0xD5 => { instruction!(self, zeropagex,   cmp, 4, 0) },
            0xD6 => { instruction!(self, zeropagex,   dec, 6, 0) },
            0xD7 => { instruction!(self, zeropagex,   dcp, 6, 0) },
            0xD8 => { instruction!(self, implicit,    cld, 2, 0) },
            0xD9 => { instruction!(self, absolutey,   cmp, 4, 1) },
            0xDA => { instruction!(self, immediate,   nop, 2, 0) },
            0xDB => { instruction!(self, absolutey,   dcp, 7, 0) },
            0xDC => { instruction!(self, absolutex,   nop, 4, 1) },
            0xDD => { instruction!(self, absolutex,   cmp, 4, 1) },
            0xDE => { instruction!(self, absolutex,   dec, 7, 0) },
            0xDF => { instruction!(self, absolutex,   dcp, 7, 0) },
            0xE0 => { instruction!(self, immediate,   cpx, 2, 0) },
            0xE1 => { instruction!(self, indirectx,   sbc, 6, 0) },
            0xE2 => { instruction!(self, immediate,   nop, 2, 0) },
            0xE3 => { instruction!(self, indirectx,   isc, 8, 0) },
            0xE4 => { instruction!(self, zeropage,    cpx, 3, 0) },
            0xE5 => { instruction!(self, zeropage,    sbc, 3, 0) },
            0xE6 => { instruction!(self, zeropage,    inc, 5, 0) },
            0xE7 => { instruction!(self, zeropage,    isc, 5, 0) },
            0xE8 => { instruction!(self, implicit,    inx, 2, 0) },
            0xE9 => { instruction!(self, immediate,   sbc, 2, 0) },
            0xEA => { instruction!(self, implicit,    nop, 2, 0) },
            0xEB => { instruction!(self, immediate,   sbc, 2, 0) },
            0xEC => { instruction!(self, absolute,    cpx, 4, 0) },
            0xED => { instruction!(self, absolute,    sbc, 4, 0) },
            0xEE => { instruction!(self, absolute,    inc, 6, 0) },
            0xEF => { instruction!(self, absolute,    isc, 6, 0) },
            0xF0 => { instruction!(self, immediate,   beq, 2, 1) },
            0xF1 => { instruction!(self, indirecty,   sbc, 5, 1) },
            0xF2 => { instruction!(self, immediate,   kil, 2, 0) },
            0xF3 => { instruction!(self, indirecty,   isc, 8, 0) },
            0xF4 => { instruction!(self, zeropagex,   nop, 4, 0) },
            0xF5 => { instruction!(self, zeropagex,   sbc, 4, 0) },
            0xF6 => { instruction!(self, zeropagex,   inc, 6, 0) },
            0xF7 => { instruction!(self, zeropagex,   isc, 6, 0) },
            0xF8 => { instruction!(self, implicit,    sed, 2, 0) },
            0xF9 => { instruction!(self, absolutey,   sbc, 4, 1) },
            0xFA => { instruction!(self, immediate,   nop, 2, 0) },
            0xFB => { instruction!(self, absolutey,   isc, 7, 0) },
            0xFC => { instruction!(self, absolutex,   nop, 4, 1) },
            0xFD => { instruction!(self, absolutex,   sbc, 4, 1) },
            0xFE => { instruction!(self, absolutex,   inc, 7, 0) },
            0xFF => { instruction!(self, absolutex,   isc, 7, 0) },
            _ => panic!("none opcode:{:x}", opcode)
        };
    }

    fn print_diff(&self, before: Cpu) {
        if self.a != before.a { info!("a: {:x} -> {:x}", before.a, self.a) }
        if self.x != before.x { info!("x: {:x} -> {:x}", before.x, self.x) }
        if self.y != before.y { info!("y: {:x} -> {:x}", before.y, self.y) }
        if self.pc != before.pc { info!("pc: {:x} -> {:x}", before.pc, self.pc) }
        if self.s != before.s { info!("s: {:x} -> {:x}", before.s, self.s) }
        if self.p != before.p {
            info!("p: 0b{:08b} -> 0b{:08b}", before.p, self.p)
        }
    }

    fn debug(&mut self) {
        let time = self.cycle as f32 * 1.0 / 22_000_000.0;
        info!("=====CPU(cycle:[{:07}({}s)],pc:[{:02x}]====", self.cycle, time,self.pc);
        info!("a:{:02x}  x:{:02x}  y:{:02x}  s:{:02x}",
              self.a,
              self.x,
              self.y,
              self.s);
        info!("p[{:x}] CRY:{}, ZER:{}, IRQ:{}, DEC:{}, BRK:{}, RSV:{}, OVF:{}, NEG:{}",
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
    }

    fn read(&self, addr: u16) -> u8 {
        self.mbc.borrow_mut().read(addr)
    }

    fn read16(&self, addr: u16) -> u16 {
        self.mbc.borrow_mut().read16(addr)
    }

    // not used
    // pub fn write(&mut self, addr: u16, data: u8) {
    //     self.mbc.borrow_mut().write(addr, data)
    // }

    fn push(&mut self, data: u8) {
        let addr = self.s as u16 + 0x0100;
        info!("push(addr => {:x}, data => {:x})", addr, data);
        self.mbc.borrow_mut().write(addr, data);
        self.s = self.s.wrapping_sub(1);
    }

    fn push16(&mut self, data: u16) {
        info!("push16(self.s:{:x}, {:x})", self.s, data);
        let low = (data & 0xFF) as u8;
        let high = (data >> 8) as u8;
        self.push(high);
        self.push(low);
    }

    fn pop(&mut self) -> u8 {
        self.s = self.s.wrapping_add(1);
        let addr = (self.s as u16) + 0x0100;
        let data = self.mbc.borrow_mut().read(addr);
        // info!("pop(addr => {:x}, data => {:x})", addr, data);
        data
    }

    fn pop16(&mut self) -> u16 {
        // info!("pop() => low({:x})", low);
        let low = self.pop() as u16;
        let high = self.pop() as u16;
        // info!("pop() => high({:x})", high);
        let data = (high << 8) | low;
        // info!("pop16(self.s:{:x}, {:x})", self.s, data);
        data
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

    fn set_negative_flag(&mut self, value: u8) {
        self.set_flag(FLAG_NEG, (value & 0x80) != 0);
    }

    fn set_zero_flag(&mut self, value: u8) {
        self.set_flag(FLAG_ZER, value == 0);
    }

    fn set_overflow_flag(&mut self, before: u8, after: u8) {
        self.set_flag(FLAG_OVF, (before & 0x80) == 0 && (after & 0x80) == 0x80);
    }

    fn process_nmi(&mut self) -> bool {
        let need_irq = {
            let mbc = self.mbc.borrow_mut();
            let enable = mbc.is_enable_nmi();
            let raised = mbc.is_raise_nmi();
            // info!("enable_nmi:{}, raise_nmi:{}", raised, raised);
            enable && raised
        };
        // info!("need_irq:{}", need_irq);

        if need_irq {
            info!("do_irq");
            self.do_irq("nmi");
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.pc = self.vector("reset");
        info!("reset vector:{:x}", self.pc);

        self.s = 0xFF;
    }

    fn vector(&self, name: &str) -> u16 {
        let addr = match name {
            "nmi"   => {0xFFFAu16}
            "reset" => {0xFFFCu16}
            "irq"   => {0xFFFEu16}
            _       => {panic!("invalid vector name:{}", name)}
        };
        self.read16(addr)
    }
}


