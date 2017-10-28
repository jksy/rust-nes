mod nes;

extern crate sdl2;
extern crate bmp;

use nes::rom::Rom;
use nes::Nes;
use nes::joypad;
use std::sync::mpsc::channel;
use bmp::Image;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use std::collections::HashSet;
use std::{thread, time};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let sdl_context = sdl2::init().unwrap();


    // window & canvas
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

    // event for input device
    let mut events = sdl_context.event_pump().unwrap();

    let mut nes = Nes::new();
    let rom = Rom::load("nestest.nes").unwrap();
    rom.print();
    nes.set_rom(rom.clone());
    nes.reset();

    let mut counter = 0;
    let mut texture = creator.
        create_texture_streaming(PixelFormatEnum::RGB888, 256, 240).unwrap();

    let mut slow = true;
    let mut prev_render_time = SystemTime::now();

    'running: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                    break 'running;
                },
                Event::KeyDown { keycode: Some(Keycode::S), ..} => {
                    slow = !slow;
                },
                _ => {}
            }
            println!("event:{:?}", event);
        }

        let keys:HashSet<Keycode> = events.
            keyboard_state().
            pressed_scancodes().
            filter_map(Keycode::from_scancode).
            collect();

        let mut button = 0u8;
        {
            for key in keys {
                match key {
                    Keycode::Up     => {button |= joypad::BUTTON_UP},
                    Keycode::Down   => {button |= joypad::BUTTON_DOWN},
                    Keycode::Left   => {button |= joypad::BUTTON_LEFT},
                    Keycode::Right  => {button |= joypad::BUTTON_RIGHT},
                    Keycode::Space  => {button |= joypad::BUTTON_SELECT},
                    Keycode::Return => {button |= joypad::BUTTON_START},
                    Keycode::A      => {button |= joypad::BUTTON_A},
                    Keycode::B      => {button |= joypad::BUTTON_B},
                    _ => {},
                }
            }
        }
        nes.set_joypad_button_state(button);


        nes.tick();

        if slow {
            thread::sleep(time::Duration::from_millis(100));
        }

        // update canvas if display changed
        let result = nes.is_display_changed();
        if nes.is_display_changed() == false {
            continue;
        }
        // TODO:
        let elapsed = prev_render_time.elapsed().unwrap();
        if elapsed.as_secs() < 1 {
            continue;
        }
        prev_render_time = SystemTime::now();

        println!("========== draw image ===============");

        // draw nes display
        let mut img = Image::new(256, 240);
        nes.render_image(&mut img);
        img.save("x.bmp");

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

        nes.clear_display_changed();
        prev_render_time = SystemTime::now();
    }
}


