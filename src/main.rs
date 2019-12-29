extern crate minifb;

use std::error::Error;
use std::fs;

use minifb::{Key, Window, WindowOptions, Scale};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const RAM_SIZE: usize = 4096;
const GREEN: u32 = 0x81c784;
const BLACK: u32 = 0x29302a;

fn ram_dump(ram: &Vec<u8>) {
    for c in ram {
        print!("{:02x} ", c);
    }

    println!("");
}

fn main() {
    println!("chip8-rust: CHIP-8 emulator written in Rust");

    let filename = String::from("roms/maze.ch8");

    let rom = match fs::read(&filename) {
        Err(why) => panic!("Could not open file: {}", why.description()),
        Ok(file) => file
    };

    let mut ram: Vec<u8> = vec![0; RAM_SIZE];
    let mut display: Vec<u32> = vec![0; WIDTH * HEIGHT];

    for (i, c) in rom.into_iter().enumerate() {
        if i >= RAM_SIZE {
            panic!("Out of memory: program too large");
        }
        //println!("Byte {:02}: {:#04x}", i, c);
        ram[i + 512] = c;
    }

    ram_dump(&ram);

    let mut window = Window::new(
        &format!("chip8-rust: {}", filename),
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: Scale::X8,
            ..WindowOptions::default()
        }
    ).unwrap();

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for (i, pixel) in display.iter_mut().enumerate() {
            *pixel = if ram[i + 512] == 0 { GREEN } else { BLACK };
        }

        window.update_with_buffer(&display, WIDTH, HEIGHT).unwrap();
    }
}
