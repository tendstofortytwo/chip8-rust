extern crate minifb;
extern crate rand;
extern crate rodio;

use std::{
    fs,
    env
};

mod cpu;
use cpu::CPU;

mod audio;
use audio::Audio;

mod window;
use window::Window;

mod util;

fn main() {
    println!("chip8-rust: CHIP-8 emulator written in Rust");

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        return eprintln!("Usage: {} <rom-file-name>", args[0]);
    }

    let filename = String::from(&args[1]);

    let rom = match fs::read(&filename) {
        Err(why) => {
            return eprintln!("Could not open file: {}", why.to_string());
        },
        Ok(file) => file
    };

    let audio = match Audio::new() {
        Ok(a) => a,
        Err(err) => {
            return eprintln!("Could not initialize audio device: {}", err);
        }
    };

    let win = match Window::new(&format!("chip8-rust: {}", filename)) {
        Ok(win) => win,
        Err(err) => {
            return eprintln!("Could not initialize window: {}", &err.to_string());
        }
    };

    let mut cpu = CPU::new(win, audio);
    match cpu.load_rom(&rom) {
        Ok(()) => (),
        Err(err) => {
            return eprintln!("Could not initialize CPU: {}", err);
        }
    };

    match cpu.run_loop() {
        Ok(()) => (),
        Err(err) => {
            return eprintln!("CPU crashed: {}", err);
        }
    }
}
