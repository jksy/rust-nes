extern crate bytes;

use std;
use std::fs::File;
use std::io::prelude::*;
use self::bytes::{BufMut, Bytes, BytesMut};
use std::mem;
use std::slice;

#[derive(Clone)]
struct RomHeader {
    magic_number: [u8; 4],
    prg_page_count: u8,
    chr_page_count: u8,
    flags6: u8,
    flags7: u8,
    flags: [u8; 2],
    zero: [u8; 6],
}

impl RomHeader {
    fn validate_magic_number(&self) -> bool {
        self.magic_number[0] == 0x4e && // 'N'
        self.magic_number[1] == 0x45 && // 'E'
        self.magic_number[2] == 0x53 && // 'S'
        self.magic_number[3] == 0x1a // EOF(DOS)
    }

    fn mapper_no(&self) -> u16 {
        let lower = (self.flags6 & 0xF0) as u16;
        let upper = (self.flags7 & 0xF0) as u16;
        lower | upper << 4
    }

    fn is_horizontal(&self) -> bool {
        (self.flags6 & 0x01) != 0
    }

    fn has_trainer(&self) -> bool {
        (self.flags6 & 0x04) != 0
    }
}

#[derive(Clone)]
pub struct Rom {
    header: RomHeader,
    prg: Bytes,
    chr: Bytes,
}

const PRG_BLOCK_SIZE: usize = 16 * 1024;
const CHR_BLOCK_SIZE: usize = 8 * 1024;

impl Rom {
    pub fn load<S: Into<String>>(filename: S) -> Result<(Box<Rom>), std::io::Error> {
        let mut file = File::open(filename.into())?;
        let header = Rom::load_header(&mut file)?;

        let mut prg = BytesMut::with_capacity(PRG_BLOCK_SIZE * header.prg_page_count as usize);
        Rom::read_file(&mut file, &mut prg, 16 * header.prg_page_count as usize)?;

        let mut chr = BytesMut::with_capacity(CHR_BLOCK_SIZE * header.chr_page_count as usize);
        Rom::read_file(&mut file, &mut chr, 8 * header.chr_page_count as usize)?;

        let rom = Rom {
            header: header,
            prg: prg.freeze(),
            chr: chr.freeze(),
        };
        Ok(Box::new(rom))
    }

    pub fn empty() -> Box<Rom> {
        let header: RomHeader = unsafe { mem::zeroed() };
        let prg = BytesMut::with_capacity(0);
        let chr = BytesMut::with_capacity(0);
        let rom = Rom {
            header: header,
            prg: prg.freeze(),
            chr: chr.freeze(),
        };
        Box::new(rom)
    }

    pub fn print(&self) {
        let magic_number = self.header.magic_number;
        info!("=======ROM Information=======");
        info!(
            "magic_number:[{}{}{}{}]",
            magic_number[0] as char,
            magic_number[1] as char,
            magic_number[2] as char,
            magic_number[3] as char,
        );
        info!(
            "validate_magic_number:{}",
            self.header.validate_magic_number()
        );
        info!("prg_page_count:{}", self.header.prg_page_count);
        info!("chr_page_count:{}", self.header.chr_page_count);
        info!("mapper_no:{}", self.header.mapper_no());
        info!("flags6:{}", self.header.flags6);
        info!("is_horizontal:{}", self.is_horizontal());
        info!("has_trainer:{}", self.has_trainer());
        info!("PRG Len:{}", self.prg.len());
        info!("CHR Len:{}", self.chr.len());
    }

    pub fn read_prg(&self, addr: u16) -> u8 {
        if self.header.prg_page_count == 1 {
            let x = addr & 0x3FFF;
            self.prg[x as usize]
        } else {
            self.prg[addr as usize]
        }
    }

    pub fn chr(&self) -> &[u8] {
        &self.chr
    }

    pub fn initial_pc(&self) -> u16 {
        let mut pc = 0x8000;
        let header = &self.header;
        if header.prg_page_count == 2 {
            let head = &self.prg[0..PRG_BLOCK_SIZE];
            let tail = &self.prg[PRG_BLOCK_SIZE..(PRG_BLOCK_SIZE * 2)];
            if header.mapper_no() == 0 && head == tail {
                pc = 0xc000
            }
        } else if header.prg_page_count == 1 {
            pc = 0xc000
        }

        pc
    }

    pub fn is_horizontal(&self) -> bool {
        self.header.is_horizontal()
    }

    pub fn has_trainer(&self) -> bool {
        self.header.has_trainer()
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

    fn read_file(
        file: &mut File,
        bytes: &mut BytesMut,
        read_kbyte: usize,
    ) -> Result<(), std::io::Error> {
        let mut buf = [0 as u8; 1024];
        for _ in 0..read_kbyte {
            file.read_exact(&mut buf)?;
            bytes.put_slice(&mut buf);
        }
        Ok(())
    }
}
