mod nes;

use nes::cpu::Cpu;
use nes::rom::Rom;

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
    init_sdl();

    let mut a = Cpu::new();
    a.reset();
    a.ram[0] = 0x01;
    a.ram[1] = 0x02;
    a.ram[2] = 0x03;
    a.ram[3] = 0x04;
    a.run();
    println!("a:{}", a.a);
    println!("x:{}", a.x);
    println!("y:{}", a.y);

    let rom = Rom::load("cpu_dummy_reads.nes").unwrap();
    rom.print();

}


