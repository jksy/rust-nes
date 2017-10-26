mod nes;

extern crate sdl2;
extern crate bmp;

use nes::rom::Rom;
use nes::Nes;
use std::thread;
use std::sync::mpsc::channel;
use bmp::Image;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-nes", 341, 261)
                                .position_centered()
                                .build()
                                .unwrap();

    let mut canvas = window.into_canvas()
                            .target_texture()
                            .present_vsync()
                            .build()
                            .unwrap();

    let creator = canvas.texture_creator();


    let mut nes = Nes::new();
    let rom = Rom::load("nestest.nes").unwrap();
    rom.print();
    nes.set_rom(rom.clone());
    nes.reset();

    let mut counter = 0;
    loop {
        nes.tick();

        counter += 1;
        if counter < 256 * 60 {
            continue;
        }
        counter = 0;

        let mut img = Image::new(256, 240);
        nes.render_image(&mut img);
        img.save("x.bmp");

        let mut texture = creator.create_texture_streaming(PixelFormatEnum::RGB888, 256, 240).unwrap();
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0u32..240u32 {
                for x in 0u32..256u32  {
                    let pixel = img.get_pixel(x, y);
                    let offset = (y * 256 * 4 + x * 4) as usize;
                    buffer[offset+1] = pixel.g;
                    buffer[offset+2] = pixel.r;
                    buffer[offset] = pixel.b;
                }
            }
        }).unwrap();
        canvas.copy(&texture, None, Some(Rect::new(0, 0, 255, 239))).unwrap();
        canvas.present();
    }

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


