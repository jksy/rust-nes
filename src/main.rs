mod nes;

use nes::cpu::Cpu;
use nes::rom::Rom;
use nes::Nes;

extern crate sdl2;

use std::io::{self, Read};

pub fn init_sdl() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-nes", 100, 100)
                                .position_centered()
                                .build()
                                .unwrap();

    let mut canvas = window.into_canvas()
                            .target_texture()
                            .present_vsync()
                            .build()
                            .unwrap();
}

fn main() {
    // init_sdl();
    let mut nes = Nes::new();
    let rom = Rom::load("nestest.nes").unwrap();
    rom.print();
    nes.set_rom(rom.clone());
    nes.reset();
    nes.run();

    // let mut cpu = Cpu::new();
    // // println!("a:{}", a.a);
    // // println!("x:{}", a.x);
    // // println!("y:{}", a.y);

    // let rom = Rom::load("nestest.nes").unwrap();
    // rom.print();
    // cpu.set_rom(rom.clone());
    // cpu.reset();
    // cpu.run();
    // // cpu.disasm();
    // // cpu.run();

    // // let mut cpu = Cpu::new();
    // // cpu.disasm(rom);

}


