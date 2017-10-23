use nes::cpu::Cpu;

#[allow(unused_variables)]
pub trait AddressingMode {
    fn read(&self, cpu: &mut Cpu) -> u8 { unimplemented!() }
    fn write(&self, cpu: &mut Cpu, data: u8) { unimplemented!() }
    fn read16(&self, cpu: &mut Cpu) -> u16 { unimplemented!() }
    fn read16_addr(&self, cpu: &mut Cpu) -> u16 { unimplemented!() }

    fn length(&self) -> u16 { unimplemented!() }
}

pub struct NoAccessAddressingMode {}
impl AddressingMode for NoAccessAddressingMode {
    fn length(&self) -> u16 { 0u16 }
}

pub struct MemoryAddressingMode {
    addr: u16,
    size: u16,
}

impl MemoryAddressingMode {
    pub fn new(addr: u16, size: u16) -> Self {
        MemoryAddressingMode{addr: addr, size: size}
    }
}

impl AddressingMode for MemoryAddressingMode {
    fn read(&self, cpu: &mut Cpu) -> u8 {
        cpu.mbc.borrow_mut().read(self.addr)
    }
    fn read16(&self, cpu: &mut Cpu) -> u16 {
        let mbc = cpu.mbc.borrow_mut();
        let low = mbc.read(self.addr) as u16;
        let high = mbc.read(self.addr+1) as u16;
        high << 8 | low
    }
    fn read16_addr(&self, _: &mut Cpu) -> u16 {
        self.addr
    }
    fn write(&self, cpu: &mut Cpu, data: u8) {
        cpu.mbc.borrow_mut().write(self.addr, data)
    }
    fn length(&self) -> u16 {self.size}
}

pub struct ImmediateAddressingMode {
    value: u8,
    size: u16,
}

impl ImmediateAddressingMode {
    pub fn new(value: u8, size: u16) -> Self {
        ImmediateAddressingMode{value: value, size: size}
    }
}

impl AddressingMode for ImmediateAddressingMode {
    fn read(&self, _: &mut Cpu) -> u8 {
        self.value
    }
    fn read16(&self, _: &mut Cpu) -> u16 {
        unimplemented!()
    }
    fn write(&self, _: &mut Cpu, _: u8) {
        unimplemented!()
    }
    fn length(&self) -> u16 {self.size}
}

pub struct AccumuratorAddressingMode {
    size: u16,
}

impl AccumuratorAddressingMode {
    pub fn new() -> Self {
        AccumuratorAddressingMode{size: 0}
    }
}

impl AddressingMode for AccumuratorAddressingMode {
    fn read(&self, cpu: &mut Cpu) -> u8 {
        cpu.a
    }
    fn write(&self, cpu: &mut Cpu, data: u8) {
        cpu.a = data
    }

    fn length(&self) -> u16 {self.size}
}


