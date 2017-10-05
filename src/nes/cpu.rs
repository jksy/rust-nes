use nes::rom::Rom;

pub struct Cpu {
    pub a: u8,      // accumulator
    pub x: u8,      // index register(X)
    pub y: u8,      // index register(Y)
    pub pc: u16,    // program counter
    pub s: u8,      // stack pointer
    pub p: u8,      // processor status register
    pub ram: [u8; 2048],
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {a: 0, x: 0, y: 0, pc: 0, s: 0, p: 0, ram: [0; 2048]}
    }

    pub fn run(&mut self) {
        loop {
            match self.ram[self.pc as usize] {
                0x00 => self.lda(),
                0x01 => self.ldx(),
                0x02 => self.ldy(),
                _ => {println!("invalid opcode:{}", self.pc); break}
            }
            self.pc += 1
        }
    }

    pub fn disasm(&mut self, rom: Rom) {
        self.pc = 0;
        while self.pc < rom.prg_len() {
            let opcode = rom.prg(self.pc);
            if opcode != 0 {
                println!("{:x} => opcode:{:x}", self.pc, opcode);
            }
            self.pc += 1;

        }
    }

    pub fn reset(&mut self) {
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
    fn pop(&mut self) -> u8 {
        let r = self.ram[self.s as usize];
        self.s -= 1;
        r
    }
    pub fn read(&mut self, addr: u16) -> u8{
        self.ram[addr as usize]
    }
    pub fn write(&mut self, addr: u16, v: u8) {
        self.ram[addr as usize] = v
    }
}

