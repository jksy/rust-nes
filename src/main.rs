mod nes;

extern crate bmp;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate sdl2;
#[macro_use]
extern crate bitflags;

use bmp::Image;
use nes::Nes;
use nes::joypad;
use nes::rom::Rom;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::render::Texture;
use sdl2::video::Window;
use std::collections::HashSet;
use std::env;
use std::io::{self, Write};
use std::process::exit;
use std::rc::Rc;
use std::time::SystemTime;
use std::{thread, time};

fn get_rom_filename() -> Result<(String), (String)> {
    if env::args().count() != 2 {
        return Err("need only one argument".to_owned());
    }
    Ok(env::args().nth(1).unwrap())
}

fn run_nes() -> Result<(), (String)> {
    env_logger::init();

    if env::args().count() != 2 {
        return Err("need only one argument".to_owned());
    }
    let rom_filename = get_rom_filename().unwrap();

    let mut nes = Nes::new();

    let sdl_context = sdl2::init().unwrap();

    // window & canvas
    let video_subsystem = sdl_context.video().unwrap();

    let (screen_width, screen_height) = nes.screen_size();
    let (window_width, window_height) = (screen_width * 2, screen_height * 2);

    let window = video_subsystem
        .window("rust-nes", window_width, window_height)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window
        .into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .unwrap();

    let creator = canvas.texture_creator();

    // event for input device
    let mut events = sdl_context.event_pump().unwrap();

    let mut nes = Nes::new();
    let rom = Rom::load(rom_filename).unwrap();
    rom.print();
    nes.set_rom(rom.clone());
    nes.reset();

    let mut texture = creator
        .create_texture_streaming(PixelFormatEnum::RGB888, screen_width, screen_height)
        .unwrap();

    let mut slow = false;
    let mut prev_render_time = SystemTime::now();
    let mut prev_poll_event_time = SystemTime::now();
    let mut button_state = 0u8;
    let mut button_state_changed = false;
    let mut img = vec![0u8; (screen_width * screen_height * 4) as usize]; // RGBA

    'running: loop {
        let elapsed = prev_poll_event_time.elapsed().unwrap();
        if elapsed.subsec_nanos() > 50_000_000 { // every 50 msec
            for event in events.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => {
                        break 'running;
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::S),
                        ..
                    } => {
                        slow = !slow;
                    }
                    Event::KeyDown { .. } | Event::KeyUp { .. } => {
                        button_state_changed = true;
                    }
                    _ => {}
                }
                info!("event:{:?}", event);
            }
            prev_poll_event_time = SystemTime::now();
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

        if !nes.screen_rendered() {
            continue;
        }
        nes.reset_screen_rendered();

        // TODO:
        let elapsed = prev_render_time.elapsed().unwrap();
        if elapsed.subsec_nanos() < 500_000_000 {  // every 500ms
        // if elapsed.as_secs() < 1 {
            continue;
        }
        prev_render_time = SystemTime::now();

        info!("========== draw image ===============");
        render_nes_screen(&nes, &mut img, &mut canvas, &mut texture);

        // draw nes display
        prev_render_time = SystemTime::now();
        // dumping ram & ppu
        // nes.dump();
    }
    Ok(())
}

fn get_button_state(events: &sdl2::EventPump) -> u8 {
    let keys: HashSet<Keycode> = events
        .keyboard_state()
        .pressed_scancodes()
        .filter_map(Keycode::from_scancode)
        .collect();

    let mut button_state = 0x0u8;
    {
        for key in keys {
            match key {
                Keycode::Up => button_state |= joypad::BUTTON_UP,
                Keycode::Down => button_state |= joypad::BUTTON_DOWN,
                Keycode::Left => button_state |= joypad::BUTTON_LEFT,
                Keycode::Right => button_state |= joypad::BUTTON_RIGHT,
                Keycode::Space => button_state |= joypad::BUTTON_SELECT,
                Keycode::Return => button_state |= joypad::BUTTON_START,
                Keycode::A => button_state |= joypad::BUTTON_A,
                Keycode::B => button_state |= joypad::BUTTON_B,
                _ => {}
            }
        }
    }
    return button_state;
}

fn render_nes_screen(
    nes: &Nes,
    img: &mut Vec<u8>,
    canvas: &mut Canvas<Window>,
    texture: &mut Texture,
) {
    nes.render_image(img);

    texture
        .with_lock(None, |buffer: &mut [u8], pitch: usize| {
            buffer.copy_from_slice(img);
        })
        .unwrap();

    canvas
        .copy(&texture, None, None)
        .unwrap();

    canvas.present();
}

fn main() {
    ::std::process::exit(match run_nes() {
        Ok(_) => 0,
        Err(err) => {
            writeln!(io::stderr(), "error: {:?}", err).unwrap();
            1
        }
    });
}
