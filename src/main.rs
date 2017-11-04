mod nes;

extern crate sdl2;
extern crate bmp;
#[macro_use]
extern crate log;
extern crate env_logger;

use nes::rom::Rom;
use nes::Nes;
use nes::joypad;
use bmp::Image;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Texture;
use sdl2::video::Window;
use sdl2::render::Canvas;
use std::collections::HashSet;
use std::{thread, time};
use std::time::SystemTime;
use std::rc::Rc;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    env_logger::init().unwrap();

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
    // let rom = Rom::load("roms/scanline.nes").unwrap();
    // let rom = Rom::load("roms/branch_timing_tests/1.Branch_Basics.nes").unwrap();
    // let rom = Rom::load("roms/dk.nes").unwrap();
    let rom = Rom::load("roms/full_palette.nes").unwrap();
    // let rom = Rom::load("roms/power_up_palette.nes").unwrap();
    // let rom = Rom::load("roms/color_test.nes").unwrap();
    // let rom = Rom::load("roms/nestest.nes").unwrap();
    // let rom = Rom::load("roms/ram_retain.nes").unwrap();
    // let rom = Rom::load("roms/cpu_dummy_reads.nes").unwrap();
    rom.print();
    nes.set_rom(rom.clone());
    nes.reset();

    let mut texture = creator.
        create_texture_streaming(PixelFormatEnum::RGB888, 256, 240).unwrap();

    let mut slow = false;
    let mut prev_render_time = SystemTime::now();
    let mut button_state = 0u8;
    let mut button_state_changed = false;
    let mut img = Image::new(256, 240);

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
                Event::KeyDown {..} |
                Event::KeyUp {..} => {
                    button_state_changed = true;
                },
                _ => {}
            }
            info!("event:{:?}", event);
        }

        if button_state_changed {
            button_state = get_button_state(&events);
            button_state_changed = false;
        }

        nes.set_joypad_button_state(button_state);
        nes.tick();

        if slow {
            thread::sleep(time::Duration::from_millis(100));
        }

        // update canvas if display changed
        if nes.is_display_changed() == false {
            continue;
        }
        // TODO:
        let elapsed = prev_render_time.elapsed().unwrap();
        if elapsed.as_secs() < 1 {
            continue;
        }
        prev_render_time = SystemTime::now();

        info!("========== draw image ===============");
        render_nes_display(&nes, &mut img, &mut canvas, &mut texture);

        // draw nes display
        nes.clear_display_changed();
        prev_render_time = SystemTime::now();
        // dumping ram & ppu
        nes.dump();
    }
}

fn get_button_state(events: &sdl2::EventPump) -> u8 {
  let keys:HashSet<Keycode> = events.
      keyboard_state().
      pressed_scancodes().
      filter_map(Keycode::from_scancode).
      collect();

  let mut button_state = 0x0u8;
  {
    for key in keys {
        match key {
            Keycode::Up     => {button_state |= joypad::BUTTON_UP},
            Keycode::Down   => {button_state |= joypad::BUTTON_DOWN},
            Keycode::Left   => {button_state |= joypad::BUTTON_LEFT},
            Keycode::Right  => {button_state |= joypad::BUTTON_RIGHT},
            Keycode::Space  => {button_state |= joypad::BUTTON_SELECT},
            Keycode::Return => {button_state |= joypad::BUTTON_START},
            Keycode::A      => {button_state |= joypad::BUTTON_A},
            Keycode::B      => {button_state |= joypad::BUTTON_B},
            _ => {},
        }
    }
  }
  return button_state;
}

fn render_nes_display(nes: &Nes, img: &mut Image, canvas: &mut Canvas<Window>, texture: &mut Texture) {
    nes.render_image(img);
    let _ = img.save("x.bmp");

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

