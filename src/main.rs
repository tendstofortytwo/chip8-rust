use std::error::Error;
use std::fs;

fn main() {
    println!("chip8-rust: CHIP-8 emulator written in Rust");

    let rom = match fs::read("../roms/maze.ch8") {
        Err(why) => panic!("Could not open file: {}", why.description()),
        Ok(file) => file
    };

    const RAM_SIZE: usize = 4096;

    let mut ram: [u8; 4096] = [0; RAM_SIZE];

    for (i, c) in rom.into_iter().enumerate() {
        if i >= RAM_SIZE {
            panic!("Out of memory: program too large");
        }
        println!("Byte {:02}: {:#04x}", i, c);
        ram[i + 512] = c;
    }
}
