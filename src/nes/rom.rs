extern crate bytes;

use std;
use std::fs::File;
use std::io::prelude::*;
use self::bytes::{BytesMut, Bytes, BufMut, Buf};
use std::mem;
use std::slice;

fn upper_bits(byte: u8) -> u8 {
    byte & 0xF0
}

struct RomHeader {
    magic_number: [u8; 4],
    prg_page_count: u8,
    chr_page_count: u8,
    lower_mapper: u8,
    upper_mapper: u8,
    flags: [u8; 2],
    zero: [u8; 6]
}

impl RomHeader {
    fn validate_magic_number(&self) -> bool {
        self.magic_number[0] == 0x4e && // 'N'
        self.magic_number[1] == 0x45 && // 'E'
        self.magic_number[2] == 0x53 && // 'S'
        self.magic_number[3] == 0x1a    // EOF(DOS)
    }

    fn mapper(&self) -> u16 {
        let lower = upper_bits(self.lower_mapper) as u16;
        let upper = upper_bits(self.upper_mapper) as u16;
        lower | upper << 4
    }
}

pub struct Rom {
    header: RomHeader,
    prg: Bytes,
    chr: Bytes,
}

impl Rom {
    pub fn load(filename: &str) -> Result<(Rom), std::io::Error> {
        let mut file = File::open(filename)?;
        let header = Rom::load_header(&mut file)?;

        let mut prg = BytesMut::with_capacity(16 * 1024 * header.prg_page_count as usize);
        Rom::read_file(&mut file, &mut prg, 16 * header.prg_page_count as usize)?;

        let mut chr = BytesMut::with_capacity(8 * 1024 * header.chr_page_count as usize);
        Rom::read_file(&mut file, &mut chr, 8 * header.chr_page_count as usize)?;

        Ok(Rom{header: header, prg: prg.freeze(), chr: chr.freeze()})
    }

    pub fn print(&self) {
        let magic_number = self.header.magic_number;
        println!("=======ROM Information=======");
        println!("magic_number:[{}{}{}{}]",
                 magic_number[0] as char,
                 magic_number[1] as char,
                 magic_number[2] as char,
                 magic_number[3] as char,
                );
        println!("validate_magic_number:{}", self.header.validate_magic_number());
        println!("prg_page_count:{}", self.header.prg_page_count);
        println!("chr_page_count:{}", self.header.chr_page_count);
        println!("mapper:{}", self.header.mapper());
        println!("PRG Len:{}", self.prg.len());
        println!("CHR Len:{}", self.chr.len());
    }

    fn load_header(file: &mut File) -> Result<(RomHeader), std::io::Error> {
        let mut header: RomHeader = unsafe { mem::zeroed() };
        let header_size = mem::size_of::<RomHeader>();
        unsafe {
            let dst_ptr = &mut header as *mut RomHeader as *mut u8;
            let mut slice = slice::from_raw_parts_mut(dst_ptr, header_size);
            file.read_exact(slice)?;
        }
        Ok(header)
    }

    fn read_file(file: &mut File, bytes: &mut BytesMut, read_kbyte: usize) -> Result<(), std::io::Error> {
        let mut buf = [0 as u8; 1024];
        for _ in 0..read_kbyte {
            file.read_exact(&mut buf)?;
            bytes.put_slice(&mut buf);
        }
        Ok(())
    }
}


fn run() {
    let rom = Rom::load("cpu_dummy_reads.nes").unwrap();
    rom.print();
}
