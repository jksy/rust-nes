use nes::mbc::Mbc;
use std::cell::RefCell;
use std::rc::Rc;
use nes::addressing_mode::*;

#[derive(Clone)]
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
            println!("opcode:{}", $name);
            if $self.get_flag($flag) == $result {
                let offset = $addr.read($self) as i8 as i32;
                let jump_addr = (($self.pc as i32) + offset) as u16 + 1;
                println!("{} Jump pc:{:x} -> {:x}", $name, $self.pc, jump_addr);
                $self.pc = jump_addr;
                true
            } else {
                $self.pc += $addr.length();
                false
            }
        }
    }
}


impl Cpu {
    pub fn new(mbc: Rc<RefCell<Box<Mbc>>>) -> Self {
        Cpu {
            a: 0, x: 0, y: 0,
            pc: 0x8000, s: 0xFF, p: 0,
            mbc: mbc,
            step: 0,
            }
    }

    pub fn setup(&mut self) {
        let pc = self.mbc.borrow().initial_pc();
        println!("setup() pc:{:x} -> {:x}", self.pc, pc);
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
        println!("opcode:BRK");
        self.do_irq("irq");
        false
    }

    fn kil<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:KIL");
        self.pc += addr.length();
        unimplemented!();
    }

    fn slo<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:SLO");
        self.pc += addr.length();
        unimplemented!();
    }
    fn nop<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:NOP");
        self.pc += addr.length();
        true
    }
    fn anc<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:ANC");
        self.pc += addr.length();
        unimplemented!();
    }
    fn clc<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:CLC");
        self.set_flag(FLAG_CRY, false);
        self.pc += addr.length();
        true
    }
    fn sec<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:SEC");
        self.set_flag(FLAG_CRY, true);
        self.pc += addr.length();
        true
    }
    fn cli<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:CLI");
        self.set_flag(FLAG_IRQ, false);
        self.pc += addr.length();
        true
    }
    fn sei<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:SEI");
        self.set_flag(FLAG_IRQ, true);
        self.pc += addr.length();
        true
    }
    fn clv<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:CLV");
        self.set_flag(FLAG_OVF, false);
        self.pc += addr.length();
        true
    }
    fn cld<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:CLD");
        self.set_flag(FLAG_DEC, false);
        self.pc += addr.length();
        true
    }
    fn sed<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:SED");
        self.set_flag(FLAG_DEC, true);
        self.pc += addr.length();
        true
    }

    // subrouting
    fn jmp<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:JMP");
        let jump_addr = addr.read16_addr(self);
        println!("jump_addr:{:x}", jump_addr);
        self.pc = jump_addr;
        false
    }
    fn jsr<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:JSR");
        self.set_flag(FLAG_DEC, true);
        let pc = self.pc;
        self.push16(pc + addr.length() - 1);
        self.pc = addr.read16_addr(self);
        false
    }
    fn rts<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:RTS");
        let return_addr = self.pop16();
        println!("self.pc({:x}) => {:x}", self.pc, return_addr);
        self.pc = return_addr + 1;
        false
    }
    fn rti<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:RTI");
        self.p = self.pop();
        let return_addr = self.pop16();
        println!("self.pc({:x}) => {:x}", self.pc, return_addr);
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
        println!("opcode:STA");
        let a = self.a;
        addr.write(self, a);
        self.pc += addr.length();
        true
    }
    fn stx<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:STX");
        let x = self.x;
        addr.write(self, x);
        self.pc += addr.length();
        true
    }
    fn sty<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:STY");
        let y = self.y;
        addr.write(self, y);
        self.pc += addr.length();
        true
    }
    fn lda<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:LDA");
        let value = addr.read(self);
        self.a = value;
        self.set_negative_flag(value);
        self.set_zero_flag(value);
        self.pc += addr.length();
        true
    }
    fn ldx<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:LDX");
        let value = addr.read(self);
        self.x = value;
        self.set_negative_flag(value);
        self.set_zero_flag(value);
        self.pc += addr.length();
        true
    }
    fn ldy<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:LDY");
        let value = addr.read(self);
        self.y = value;
        self.set_negative_flag(value);
        self.set_zero_flag(value);
        self.pc += addr.length();
        true
    }
    fn tax<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:TAX");
        let value = self.a;
        self.x = value;
        self.set_negative_flag(value);
        self.set_zero_flag(value);
        self.pc += addr.length();
        true
    }
    fn tay<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:TAY");
        let value = self.a;
        self.y = value;
        self.set_negative_flag(value);
        self.set_zero_flag(value);
        self.pc += addr.length();
        true
    }
    fn tsx<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:TSX");
        let value = self.s;
        self.x = value;
        self.set_negative_flag(value);
        self.set_zero_flag(value);
        self.pc += addr.length();
        true
    }
    fn txa<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:TXA");
        let value = self.x;
        self.a = value;
        self.set_negative_flag(value);
        self.set_zero_flag(value);
        self.pc += addr.length();
        true
    }
    fn txs<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:TXS");
        let value = self.x;
        self.s = value;
        self.pc += addr.length();
        true
    }
    fn tya<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:TYA");
        let value = self.y;
        self.a = value;
        self.set_negative_flag(value);
        self.set_zero_flag(value);
        self.pc += addr.length();
        true
    }

    // caluculate oprators
    fn adc<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:ADC");
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
        println!("opcode:AND");
        let result = self.a & addr.read(self);
        self.set_negative_flag(result);
        self.set_zero_flag(result);
        self.a = result;
        self.pc += addr.length();
        true
    }
    fn asl<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:ASL");
        let carry = (self.a & 0x80) == 0x80;
        let result = self.a.wrapping_shl(1);
        self.set_flag(FLAG_CRY, carry);
        self.set_negative_flag(result);
        self.set_zero_flag(result);

        self.a = result;
        self.pc += addr.length();
        true
    }
    fn bit<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:BIT");
        let operand = addr.read(self);
        let result = self.a & operand;
        self.set_zero_flag(result);
        self.set_negative_flag(operand);
        self.set_flag(FLAG_OVF, (operand & 0x40) == 0x40);
        self.a = result;
        self.pc += addr.length();
        true
    }
    fn cmp<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:CMP");
        let v = addr.read(self);
        println!("self.a={:x}, addr.read={:x}", self.a, v);
        cmp!(self, self.a, v);
        self.pc += addr.length();
        true
    }
    fn cpx<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:CPX");
        cmp!(self, self.x, addr.read(self));
        self.pc += addr.length();
        true
    }
    fn cpy<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:CPY");
        cmp!(self, self.y, addr.read(self));
        self.pc += addr.length();
        true
    }
    fn dec<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:DEC");
        let mut value = addr.read(self);
        dec!(self, value);
        addr.write(self, value);
        self.pc += addr.length();
        true
    }
    fn dex<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:DEX");
        dec!(self, self.x);
        self.pc += addr.length();
        true
    }
    fn dey<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:DEY");
        dec!(self, self.y);
        self.pc += addr.length();
        true
    }
    fn eor<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:EOR");
        let operand = addr.read(self);
        let result = operand ^ self.a;
        self.set_zero_flag(result);
        self.set_negative_flag(result);
        self.a = result;
        self.pc += addr.length();
        true
    }
    fn inc<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:INC");
        let mut value = addr.read(self);
        inc!(self, value);
        addr.write(self, value);
        self.pc += addr.length();
        true
    }
    fn inx<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:INX");
        inc!(self, self.x);
        self.pc += addr.length();
        true
    }
    fn iny<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:INY");
        inc!(self, self.y);
        self.pc += addr.length();
        true
    }
    fn lsr<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:LSR");
        let carry = (self.a & 0x01) != 0;
        self.set_flag(FLAG_CRY, carry);
        let result = self.a >> 1;
        self.set_flag(FLAG_ZER, result == 0);
        self.set_flag(FLAG_NEG, (result & 0x80) != 0);
        self.a = result;
        self.pc += addr.length();
        true
    }
    fn ora<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:ORA");
        let result = self.a | addr.read(self);
        self.set_negative_flag(result);
        self.set_zero_flag(result);
        self.a = result;
        self.pc += addr.length();
        true
    }
    fn rol<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:ROL");
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
        println!("opcode:ROR");
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
        println!("opcode:SRE");
        let data = addr.read(self);
        self.set_flag(FLAG_CRY, (data & 0x01) != 0);
        addr.write(self, data >> 1);
        self.pc += addr.length();
        true
    }
    fn sbc<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:SBC");
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
        println!("opcode:PHA");
        let a = self.a;
        self.push(a);
        self.set_negative_flag(a);
        self.set_zero_flag(a);
        self.pc += addr.length();
        true
    }
    fn php<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:PHP");
        let p = self.p | FLAG_BRK | FLAG_RSV;
        self.push(p);
        self.pc += addr.length();
        true
    }
    fn pla<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:PLA");
        let a = self.pop();
        self.a = a;
        self.set_negative_flag(a);
        self.set_zero_flag(a);
        self.pc += addr.length();
        true
    }
    fn plp<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:PLP");
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
        println!("opcode:RLA");
        unimplemented!();
    }
    fn rra<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:RRA");
        unimplemented!();
    }
    fn alr<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:ALR");
        unimplemented!();
    }
    fn arr<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:ARR");
        unimplemented!();
    }
    fn sax<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:SAX");
        unimplemented!();
    }
    fn say<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:SAY");
        unimplemented!();
    }
    fn xaa<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:XAA");
        unimplemented!();
    }
    fn ahx<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:AHX");
        unimplemented!();
    }
    fn tas<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:TAS");
        unimplemented!();
    }
    fn shx<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:SHX");
        unimplemented!();
    }
    fn shy<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:SHY");
        unimplemented!();
    }
    fn lax<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:LAX");
        unimplemented!();
    }
    fn las<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:LAS");
        unimplemented!();
    }
    fn dcp<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:DCP");
        unimplemented!();
    }
    fn axs<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:AXS");
        unimplemented!();
    }
    fn isc<T:AddressingMode>(&mut self, addr: T) -> bool {
        println!("opcode:ISC");
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
        let arg = self.read(self.pc) as u16;
        let addr = self.read16(arg).wrapping_add(self.y as u16);
        MemoryAddressingMode::new(addr, 1)
    }

    fn zeropage(&mut self) -> MemoryAddressingMode {
        let addr = self.read(self.pc) as u16;
        MemoryAddressingMode::new(addr, 1)
    }

    fn zeropagex(&mut self) -> MemoryAddressingMode {
        let addr = ((self.pc + self.x as u16) & 0xFF) as u16;
        MemoryAddressingMode::new(addr, 1)
    }

    fn zeropagey(&mut self) -> MemoryAddressingMode {
        let addr = ((self.pc + self.y as u16) & 0xFF) as u16;
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

    pub fn tick(&mut self) {
        self.step += 1;
        self.debug();

        if self.process_nmi() {
            return;
        }

        let before_status = self.clone();

        // if self.pc < 0xc000 {
        //     panic!("== invalidate pc:${:x}", self.pc);
        // }
        let opcode = self.read(self.pc);
        self.pc += 1;

        let cont = match opcode {
            0x00 => {let m = self.implicit();    self.brk(m) },
            0x01 => {let m = self.indirectx();   self.ora(m) },
            0x02 => {let m = self.immediate();   self.kil(m) },
            0x03 => {let m = self.indirectx();   self.slo(m) },
            0x04 => {let m = self.zeropage();    self.nop(m) },
            0x05 => {let m = self.zeropage();    self.ora(m) },
            0x06 => {let m = self.zeropage();    self.asl(m) },
            0x07 => {let m = self.zeropage();    self.slo(m) },
            0x08 => {let m = self.implicit();    self.php(m) },
            0x09 => {let m = self.immediate();   self.ora(m) },
            0x0A => {let m = self.accumurator(); self.asl(m) },
            0x0B => {let m = self.immediate();   self.anc(m) },
            0x0C => {let m = self.absolute();    self.nop(m) },
            0x0D => {let m = self.absolute();    self.ora(m) },
            0x0E => {let m = self.absolute();    self.asl(m) },
            0x0F => {let m = self.absolute();    self.slo(m) },
            0x10 => {let m = self.immediate();   self.bpl(m) },
            0x11 => {let m = self.indirecty();   self.ora(m) },
            0x12 => {let m = self.immediate();   self.kil(m) },
            0x13 => {let m = self.indirecty();   self.slo(m) },
            0x14 => {let m = self.zeropagex();   self.nop(m) },
            0x15 => {let m = self.zeropagex();   self.ora(m) },
            0x16 => {let m = self.zeropagex();   self.asl(m) },
            0x17 => {let m = self.zeropagex();   self.slo(m) },
            0x18 => {let m = self.implicit();    self.clc(m) },
            0x19 => {let m = self.absolutey();   self.ora(m) },
            0x1A => {let m = self.immediate();   self.nop(m) },
            0x1B => {let m = self.absolutey();   self.slo(m) },
            0x1C => {let m = self.absolutex();   self.nop(m) },
            0x1D => {let m = self.absolutex();   self.ora(m) },
            0x1E => {let m = self.absolutex();   self.asl(m) },
            0x1F => {let m = self.absolutex();   self.slo(m) },
            0x20 => {let m = self.absolute();    self.jsr(m) },
            0x21 => {let m = self.indirectx();   self.and(m) },
            0x22 => {let m = self.immediate();   self.kil(m) },
            0x23 => {let m = self.indirectx();   self.rla(m) },
            0x24 => {let m = self.zeropage();    self.bit(m) },
            0x25 => {let m = self.zeropage();    self.and(m) },
            0x26 => {let m = self.zeropage();    self.rol(m) },
            0x27 => {let m = self.zeropage();    self.rla(m) },
            0x28 => {let m = self.implicit();    self.plp(m) },
            0x29 => {let m = self.immediate();   self.and(m) },
            0x2A => {let m = self.accumurator(); self.rol(m) },
            0x2B => {let m = self.immediate();   self.anc(m) },
            0x2C => {let m = self.absolute();    self.bit(m) },
            0x2D => {let m = self.absolute();    self.and(m) },
            0x2E => {let m = self.absolute();    self.rol(m) },
            0x2F => {let m = self.absolute();    self.rla(m) },
            0x30 => {let m = self.immediate();   self.bmi(m) },
            0x31 => {let m = self.indirecty();   self.and(m) },
            0x32 => {let m = self.immediate();   self.kil(m) },
            0x33 => {let m = self.indirecty();   self.rla(m) },
            0x34 => {let m = self.zeropagex();   self.nop(m) },
            0x35 => {let m = self.zeropagex();   self.and(m) },
            0x36 => {let m = self.zeropagex();   self.rol(m) },
            0x37 => {let m = self.zeropagex();   self.rla(m) },
            0x38 => {let m = self.implicit();    self.sec(m) },
            0x39 => {let m = self.absolutey();   self.and(m) },
            0x3A => {let m = self.immediate();   self.nop(m) },
            0x3B => {let m = self.absolutey();   self.rla(m) },
            0x3C => {let m = self.absolutex();   self.nop(m) },
            0x3D => {let m = self.absolutex();   self.and(m) },
            0x3E => {let m = self.absolutex();   self.rol(m) },
            0x3F => {let m = self.absolutex();   self.rla(m) },
            0x40 => {let m = self.implicit();    self.rti(m) },
            0x41 => {let m = self.indirectx();   self.eor(m) },
            0x42 => {let m = self.immediate();   self.kil(m) },
            0x43 => {let m = self.indirectx();   self.sre(m) },
            0x44 => {let m = self.zeropage();    self.nop(m) },
            0x45 => {let m = self.zeropage();    self.eor(m) },
            0x46 => {let m = self.zeropage();    self.lsr(m) },
            0x47 => {let m = self.zeropage();    self.sre(m) },
            0x48 => {let m = self.implicit();    self.pha(m) },
            0x49 => {let m = self.immediate();   self.eor(m) },
            0x4A => {let m = self.accumurator(); self.lsr(m) },
            0x4B => {let m = self.immediate();   self.alr(m) },
            0x4C => {let m = self.absolute();    self.jmp(m) },
            0x4D => {let m = self.absolute();    self.eor(m) },
            0x4E => {let m = self.absolute();    self.lsr(m) },
            0x4F => {let m = self.absolute();    self.sre(m) },
            0x50 => {let m = self.immediate();   self.bvc(m) },
            0x51 => {let m = self.indirecty();   self.eor(m) },
            0x52 => {let m = self.immediate();   self.kil(m) },
            0x53 => {let m = self.indirecty();   self.sre(m) },
            0x54 => {let m = self.zeropagex();   self.nop(m) },
            0x55 => {let m = self.zeropagex();   self.eor(m) },
            0x56 => {let m = self.zeropagex();   self.lsr(m) },
            0x57 => {let m = self.zeropagex();   self.sre(m) },
            0x58 => {let m = self.implicit();    self.cli(m) },
            0x59 => {let m = self.absolutey();   self.eor(m) },
            0x5A => {let m = self.immediate();   self.nop(m) },
            0x5B => {let m = self.absolutey();   self.sre(m) },
            0x5C => {let m = self.absolutex();   self.nop(m) },
            0x5D => {let m = self.absolutex();   self.eor(m) },
            0x5E => {let m = self.absolutex();   self.lsr(m) },
            0x5F => {let m = self.absolutex();   self.sre(m) },
            0x60 => {let m = self.implicit();    self.rts(m) },
            0x61 => {let m = self.indirectx();   self.adc(m) },
            0x62 => {let m = self.immediate();   self.kil(m) },
            0x63 => {let m = self.indirectx();   self.rra(m) },
            0x64 => {let m = self.zeropage();    self.nop(m) },
            0x65 => {let m = self.zeropage();    self.adc(m) },
            0x66 => {let m = self.zeropage();    self.ror(m) },
            0x67 => {let m = self.zeropage();    self.rra(m) },
            0x68 => {let m = self.implicit();    self.pla(m) },
            0x69 => {let m = self.immediate();   self.adc(m) },
            0x6A => {let m = self.accumurator(); self.ror(m) },
            0x6B => {let m = self.immediate();   self.arr(m) },
            0x6C => {let m = self.indirect();    self.jmp(m) },
            0x6D => {let m = self.absolute();    self.adc(m) },
            0x6E => {let m = self.absolute();    self.ror(m) },
            0x6F => {let m = self.absolute();    self.rra(m) },
            0x70 => {let m = self.immediate();   self.bvs(m) },
            0x71 => {let m = self.indirecty();   self.adc(m) },
            0x72 => {let m = self.immediate();   self.kil(m) },
            0x73 => {let m = self.indirecty();   self.rra(m) },
            0x74 => {let m = self.zeropagex();   self.nop(m) },
            0x75 => {let m = self.zeropagex();   self.adc(m) },
            0x76 => {let m = self.zeropagex();   self.ror(m) },
            0x77 => {let m = self.zeropagex();   self.rra(m) },
            0x78 => {let m = self.implicit();    self.sei(m) },
            0x79 => {let m = self.absolutey();   self.adc(m) },
            0x7A => {let m = self.immediate();   self.nop(m) },
            0x7B => {let m = self.absolutey();   self.rra(m) },
            0x7C => {let m = self.absolutex();   self.nop(m) },
            0x7D => {let m = self.absolutex();   self.adc(m) },
            0x7E => {let m = self.absolutex();   self.ror(m) },
            0x7F => {let m = self.absolutex();   self.rra(m) },
            0x80 => {let m = self.immediate();   self.nop(m) },
            0x81 => {let m = self.indirectx();   self.sta(m) },
            0x82 => {let m = self.immediate();   self.nop(m) },
            0x83 => {let m = self.indirectx();   self.sax(m) },
            0x84 => {let m = self.zeropage();    self.sty(m) },
            0x85 => {let m = self.zeropage();    self.sta(m) },
            0x86 => {let m = self.zeropage();    self.stx(m) },
            0x87 => {let m = self.zeropage();    self.sax(m) },
            0x88 => {let m = self.implicit();    self.dey(m) },
            0x89 => {let m = self.immediate();   self.nop(m) },
            0x8A => {let m = self.implicit();    self.txa(m) },
            0x8B => {let m = self.immediate();   self.xaa(m) },
            0x8C => {let m = self.absolute();    self.sty(m) },
            0x8D => {let m = self.absolute();    self.sta(m) },
            0x8E => {let m = self.absolute();    self.stx(m) },
            0x8F => {let m = self.absolute();    self.sax(m) },
            0x90 => {let m = self.immediate();   self.bcc(m) },
            0x91 => {let m = self.indirecty();   self.sta(m) },
            0x92 => {let m = self.immediate();   self.kil(m) },
            0x93 => {let m = self.indirecty();   self.ahx(m) },
            0x94 => {let m = self.zeropagex();   self.sty(m) },
            0x95 => {let m = self.zeropagex();   self.sta(m) },
            0x96 => {let m = self.zeropagey();   self.stx(m) },
            0x97 => {let m = self.zeropagey();   self.sax(m) },
            0x98 => {let m = self.implicit();    self.tya(m) },
            0x99 => {let m = self.absolutey();   self.sta(m) },
            0x9A => {let m = self.implicit();    self.txs(m) },
            0x9B => {let m = self.immediate();   self.tas(m) },
            0x9C => {let m = self.absolutex();   self.shy(m) },
            0x9D => {let m = self.absolutex();   self.sta(m) },
            0x9E => {let m = self.absolutey();   self.shx(m) },
            0x9F => {let m = self.absolutey();   self.ahx(m) },
            0xA0 => {let m = self.immediate();   self.ldy(m) },
            0xA1 => {let m = self.indirectx();   self.lda(m) },
            0xA2 => {let m = self.immediate();   self.ldx(m) },
            0xA3 => {let m = self.indirectx();   self.lax(m) },
            0xA4 => {let m = self.zeropage();    self.ldy(m) },
            0xA5 => {let m = self.zeropage();    self.lda(m) },
            0xA6 => {let m = self.zeropage();    self.ldx(m) },
            0xA7 => {let m = self.zeropage();    self.lax(m) },
            0xA8 => {let m = self.implicit();    self.tay(m) },
            0xA9 => {let m = self.immediate();   self.lda(m) },
            0xAA => {let m = self.implicit();    self.tax(m) },
            0xAB => {let m = self.immediate();   self.lax(m) },
            0xAC => {let m = self.absolute();    self.ldy(m) },
            0xAD => {let m = self.absolute();    self.lda(m) },
            0xAE => {let m = self.absolute();    self.ldx(m) },
            0xAF => {let m = self.absolute();    self.lax(m) },
            0xB0 => {let m = self.immediate();   self.bcs(m) },
            0xB1 => {let m = self.indirecty();   self.lda(m) },
            0xB2 => {let m = self.immediate();   self.kil(m) },
            0xB3 => {let m = self.indirecty();   self.lax(m) },
            0xB4 => {let m = self.zeropagex();   self.ldy(m) },
            0xB5 => {let m = self.zeropagex();   self.lda(m) },
            0xB6 => {let m = self.zeropagey();   self.ldx(m) },
            0xB7 => {let m = self.zeropagey();   self.lax(m) },
            0xB8 => {let m = self.implicit();    self.clv(m) },
            0xB9 => {let m = self.absolutey();   self.lda(m) },
            0xBA => {let m = self.implicit();    self.tsx(m) },
            0xBB => {let m = self.absolutey();   self.las(m) },
            0xBC => {let m = self.absolutex();   self.ldy(m) },
            0xBD => {let m = self.absolutex();   self.lda(m) },
            0xBE => {let m = self.absolutey();   self.ldx(m) },
            0xBF => {let m = self.absolutey();   self.lax(m) },
            0xC0 => {let m = self.immediate();   self.cpy(m) },
            0xC1 => {let m = self.indirectx();   self.cmp(m) },
            0xC2 => {let m = self.immediate();   self.nop(m) },
            0xC3 => {let m = self.indirectx();   self.dcp(m) },
            0xC4 => {let m = self.zeropage();    self.cpy(m) },
            0xC5 => {let m = self.zeropage();    self.cmp(m) },
            0xC6 => {let m = self.zeropage();    self.dec(m) },
            0xC7 => {let m = self.zeropage();    self.dcp(m) },
            0xC8 => {let m = self.implicit();    self.iny(m) },
            0xC9 => {let m = self.immediate();   self.cmp(m) },
            0xCA => {let m = self.implicit();    self.dex(m) },
            0xCB => {let m = self.immediate();   self.axs(m) },
            0xCC => {let m = self.absolute();    self.cpy(m) },
            0xCD => {let m = self.absolute();    self.cmp(m) },
            0xCE => {let m = self.absolute();    self.dec(m) },
            0xCF => {let m = self.absolute();    self.dcp(m) },
            0xD0 => {let m = self.immediate();   self.bne(m) },
            0xD1 => {let m = self.indirecty();   self.cmp(m) },
            0xD2 => {let m = self.immediate();   self.kil(m) },
            0xD3 => {let m = self.indirecty();   self.dcp(m) },
            0xD4 => {let m = self.zeropagex();   self.nop(m) },
            0xD5 => {let m = self.zeropagex();   self.cmp(m) },
            0xD6 => {let m = self.zeropagex();   self.dec(m) },
            0xD7 => {let m = self.zeropagex();   self.dcp(m) },
            0xD8 => {let m = self.implicit();    self.cld(m) },
            0xD9 => {let m = self.absolutey();   self.cmp(m) },
            0xDA => {let m = self.immediate();   self.nop(m) },
            0xDB => {let m = self.absolutey();   self.dcp(m) },
            0xDC => {let m = self.absolutex();   self.nop(m) },
            0xDD => {let m = self.absolutex();   self.cmp(m) },
            0xDE => {let m = self.absolutex();   self.dec(m) },
            0xDF => {let m = self.absolutex();   self.dcp(m) },
            0xE0 => {let m = self.immediate();   self.cpx(m) },
            0xE1 => {let m = self.indirectx();   self.sbc(m) },
            0xE2 => {let m = self.immediate();   self.nop(m) },
            0xE3 => {let m = self.indirectx();   self.isc(m) },
            0xE4 => {let m = self.zeropage();    self.cpx(m) },
            0xE5 => {let m = self.zeropage();    self.sbc(m) },
            0xE6 => {let m = self.zeropage();    self.inc(m) },
            0xE7 => {let m = self.zeropage();    self.isc(m) },
            0xE8 => {let m = self.implicit();    self.inx(m) },
            0xE9 => {let m = self.immediate();   self.sbc(m) },
            0xEA => {let m = self.implicit();    self.nop(m) },
            0xEB => {let m = self.immediate();   self.sbc(m) },
            0xEC => {let m = self.absolute();    self.cpx(m) },
            0xED => {let m = self.absolute();    self.sbc(m) },
            0xEE => {let m = self.absolute();    self.inc(m) },
            0xEF => {let m = self.absolute();    self.isc(m) },
            0xF0 => {let m = self.immediate();   self.beq(m) },
            0xF1 => {let m = self.indirecty();   self.sbc(m) },
            0xF2 => {let m = self.immediate();   self.kil(m) },
            0xF3 => {let m = self.indirecty();   self.isc(m) },
            0xF4 => {let m = self.zeropagex();   self.nop(m) },
            0xF5 => {let m = self.zeropagex();   self.sbc(m) },
            0xF6 => {let m = self.zeropagex();   self.inc(m) },
            0xF7 => {let m = self.zeropagex();   self.isc(m) },
            0xF8 => {let m = self.implicit();    self.sed(m) },
            0xF9 => {let m = self.absolutey();   self.sbc(m) },
            0xFA => {let m = self.immediate();   self.nop(m) },
            0xFB => {let m = self.absolutey();   self.isc(m) },
            0xFC => {let m = self.absolutex();   self.nop(m) },
            0xFD => {let m = self.absolutex();   self.sbc(m) },
            0xFE => {let m = self.absolutex();   self.inc(m) },
            0xFF => {let m = self.absolutex();   self.isc(m) },
            _ => panic!("none opcode:{:x}", opcode)
        };

        self.print_diff(before_status);

    }

    fn print_diff(&self, before: Cpu) {
        if self.a != before.a { println!("a: {:x} -> {:x}", before.a, self.a) }
        if self.x != before.x { println!("x: {:x} -> {:x}", before.x, self.x) }
        if self.y != before.y { println!("y: {:x} -> {:x}", before.y, self.y) }
        if self.pc != before.pc { println!("pc: {:x} -> {:x}", before.pc, self.pc) }
        if self.s != before.s { println!("s: {:x} -> {:x}", before.s, self.s) }
        if self.p != before.p {
            println!("p: 0b{:08b} -> 0b{:08b}", before.p, self.p)
        }
    }

    fn debug(&mut self) {
        let time = self.step as f32 * 1.0 / 22_000_000.0;
        println!("=====CPU(step:[{:07}({}s)],pc:[{:02x}]====", self.step, time,self.pc);
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
        let addr = 0x00u16;
        let test_result = self.read(addr);
        println!("test_result:0x{:x}", test_result);
    }

    fn read(&self, addr: u16) -> u8 {
        self.mbc.borrow_mut().read(addr)
    }

    fn read16(&self, addr: u16) -> u16 {
        self.mbc.borrow_mut().read16(addr)
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.mbc.borrow_mut().write(addr, data)
    }

    fn push(&mut self, data: u8) {
        let addr = self.s as u16 + 0x0100;
        println!("push(addr => {:x}, data => {:x})", addr, data);
        self.mbc.borrow_mut().write(addr, data);
        self.s -= 1;
    }

    fn push16(&mut self, data: u16) {
        println!("push16(self.s:{:x}, {:x})", self.s, data);
        let low = (data & 0xFF) as u8;
        let high = (data >> 8) as u8;
        self.push(high);
        self.push(low);
    }

    fn pop(&mut self) -> u8 {
        self.s += 1;
        let addr = (self.s as u16) + 0x0100;
        let data = self.mbc.borrow_mut().read(addr);
        // println!("pop(addr => {:x}, data => {:x})", addr, data);
        data
    }

    fn pop16(&mut self) -> u16 {
        // println!("pop() => low({:x})", low);
        let low = self.pop() as u16;
        let high = self.pop() as u16;
        // println!("pop() => high({:x})", high);
        let data = (high << 8) | low;
        // println!("pop16(self.s:{:x}, {:x})", self.s, data);
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
            // println!("enable_nmi:{}, raise_nmi:{}", raised, raised);
            enable && raised
        };
        // println!("need_irq:{}", need_irq);

        if need_irq {
            println!("do_irq");
            self.do_irq("nmi");
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.pc = self.vector("reset");
        println!("reset vector:{:x}", self.pc);

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

