use std::env;
use std::env::args;

pub mod bus;
pub mod cartridge;
pub mod cpu;
pub mod opcodes;
pub mod trace;
pub mod ppu;
pub mod render;
pub mod tiles_viewer;
pub mod controller;

use std::collections::HashMap;

use crate::trace::trace;
use crate::controller::Controller;
use bus::Bus;
use cartridge::Rom;
use cpu::Memory;
use cpu::CPU;
use ppu::NesPPU;
use render::frame::Frame;
use tiles_viewer::main1;

extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::EventPump;



fn main() {

    let mut key_map = HashMap::new();
    key_map.insert(Keycode::Down, controller::ControllerButtons::DOWN);
    key_map.insert(Keycode::Up, controller::ControllerButtons::UP);
    key_map.insert(Keycode::Left, controller::ControllerButtons::LEFT);
    key_map.insert(Keycode::Right, controller::ControllerButtons::RIGHT);
    key_map.insert(Keycode::Space, controller::ControllerButtons::SELECT);
    key_map.insert(Keycode::Return, controller::ControllerButtons::START);
    key_map.insert(Keycode::A, controller::ControllerButtons::BUTTON_A);
    key_map.insert(Keycode::S, controller::ControllerButtons::BUTTON_B);

    let sdl_context = sdl2::
        init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("NES", (256.0 * 2.0) as u32, (240.0 * 2.0) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(2.0, 2.0).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();

    let args: Vec<String> = env::args().collect();
    print!("This is the args debug print {:?}\n", args);
    if args.len() != 2 {
        panic!("Usage: 'cargo run -- `path_to_rom`'");
    }
    let rom_path = args[1].clone();

    let bytes: Vec<u8> = std::fs::read(rom_path).unwrap();
    let rom = Rom::new(&bytes).unwrap();

    let mut frame = Frame::new();

    let bus = Bus::new(rom, move |ppu: &NesPPU, controller: &mut controller::Controller| {
        render::render(ppu, &mut frame);
        texture.update(None, &frame.data, 256 * 3).unwrap();
 
        canvas.copy(&texture, None, None).unwrap();

        canvas.present();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => std::process::exit(0),
                Event::KeyDown { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        controller.set_button_pressed_status(*key, true);
                    }
                    println!("Key down, controller status: {:b}", controller.button_status)
                }
                Event::KeyUp { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        controller.set_button_pressed_status(*key, false);
                    }
                    println!("Key up, controller status: {:b}", controller.button_status)
                }
                _ => { /* do nothing */ }
            }
        }
    });

    let mut cpu = CPU::new(bus);

    cpu.reset();
    cpu.run();
}

// a helper function that helps read and respond to user inputs
fn handle_user_input(cpu: &mut CPU, event_pump: &mut EventPump) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => std::process::exit(0),
            Event::KeyDown {
                keycode: Some(Keycode::W),
                ..
            } => {
                cpu.mem_write(0xFF, 0x77) // write to the address that stores the last user input
                                          // 0x77 is the hex value for 'w'
            }
            Event::KeyDown {
                keycode: Some(Keycode::S),
                ..
            } => cpu.mem_write(0xFF, 0x73),
            Event::KeyDown {
                keycode: Some(Keycode::A),
                ..
            } => cpu.mem_write(0xFF, 0x61),
            Event::KeyDown {
                keycode: Some(Keycode::D),
                ..
            } => cpu.mem_write(0xFF, 0x64),
            _ => { /*No other keycodes added yet, even these are specific to snake*/ }
        }
    }
}

// helper for drawing on screen
fn color(byte: u8) -> Color {
    match byte {
        0 => sdl2::pixels::Color::BLACK,
        1 => sdl2::pixels::Color::WHITE,
        2 | 9 => sdl2::pixels::Color::GREY,
        3 | 10 => sdl2::pixels::Color::RED,
        4 | 11 => sdl2::pixels::Color::GREEN,
        5 | 12 => sdl2::pixels::Color::BLUE,
        6 | 13 => sdl2::pixels::Color::MAGENTA,
        7 | 14 => sdl2::pixels::Color::YELLOW,
        _ => sdl2::pixels::Color::CYAN,
    }
}

// helper to read the screen state
fn read_screen_state(cpu: &mut CPU, frame: &mut [u8; 32 * 3 * 32]) -> bool {
    let mut frame_idx = 0;
    let mut update = false;
    for i in 0x0200..0x0600 {
        let color_idx = cpu.mem_read(i as u16);
        let (b1, b2, b3) = color(color_idx).rgb();
        if frame[frame_idx] != b1 || frame[frame_idx + 1] != b2 || frame[frame_idx + 2] != b3 {
            frame[frame_idx] = b1;
            frame[frame_idx + 1] = b2;
            frame[frame_idx + 2] = b3;
            update = true;
        }
        frame_idx += 3;
    }
    update
}
