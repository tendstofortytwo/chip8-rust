extern crate minifb;
extern crate rand;
extern crate rodio;

use std::{
    error::Error,
    fs,
    env
};
use rodio::Sink;

use minifb::{
    Key, 
    KeyRepeat, 
    Window, 
    WindowOptions, 
    Scale
};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const RAM_SIZE: usize = 4096;
const REGISTER_COUNT: usize = 16;
const PX_OFF: u32 = 0x81c784;
const PX_ON: u32 = 0x29302a;

fn ram_dump(ram: &Vec<u8>) {
    for c in ram {
        print!("{:02x} ", c);
    }

    println!("");
}

fn handle_key_events(window: &Window) -> Vec<bool> {
    let mut keys: Vec<bool> = vec![false; 16];
    window.get_keys().map(|keys_received| {
        for k in keys_received {
            match k {
                Key::Key1 => keys[0x1] = true,
                Key::Key2 => keys[0x2] = true,
                Key::Key3 => keys[0x3] = true,
                Key::Key4 => keys[0xc] = true,
                Key::Q => keys[0x4] = true,
                Key::W => keys[0x5] = true,
                Key::E => keys[0x6] = true,
                Key::R => keys[0xd] = true,
                Key::A => keys[0x7] = true,
                Key::S => keys[0x8] = true,
                Key::D => keys[0x9] = true,
                Key::F => keys[0xe] = true,
                Key::Z => keys[0xa] = true,
                Key::X => keys[0x0] = true,
                Key::C => keys[0xb] = true,
                Key::V => keys[0xf] = true,
                _ => ()
            };
        }
    });
    keys
}

// given a hex number, return number formed by taking
// d digits from it after removing the first o digits
// looking from least significant digit first
// eg. get_hex_digits(0x1e90ff, 3, 2) will return 0xe90:
// it removes first 2 digits (ff) and then returns the
// number formed by taking the next 3 (0xe90)
fn get_hex_digits(n: &u16, d: u32, o: u32) -> usize {
    let base: u16 = 0x10;
    ((n / base.pow(o)) % base.pow(d)) as usize
}

// check if nth bit of a byte is set,
// zero-indexed, least significant first
fn is_bit_set(byte: &u8, n: u8) -> bool {
    if byte & (1 << n) == 0 { false } else { true }
}

// return nth bit of a byte, zero-indexed, 
// least significant first
fn get_bit(byte: &u8, n: u8) -> u8 {
    if is_bit_set(byte, n) { 1 } else { 0 }
}

fn preload_ram(ram: &mut Vec<u8>) {
    // the ith element of this vector is a vector of bytes
    // representing the numbers in CHIP-8 format
    let digits: Vec<Vec<u8>> = vec![vec![0xf0, 0x90, 0x90, 0x90, 0xf0],
                                    vec![0x20, 0x60, 0x20, 0x20, 0x70],
                                    vec![0xf0, 0x10, 0xf0, 0x80, 0xf0],
                                    vec![0xf0, 0x10, 0xf0, 0x10, 0xf0],
                                    vec![0x90, 0x90, 0xf0, 0x10, 0x10],
                                    vec![0xf0, 0x80, 0xf0, 0x10, 0xf0],
                                    vec![0xf0, 0x80, 0xf0, 0x90, 0xf0],
                                    vec![0xf0, 0x10, 0x20, 0x40, 0x40],
                                    vec![0xf0, 0x90, 0xf0, 0x90, 0xf0],
                                    vec![0xf0, 0x90, 0xf0, 0x10, 0xf0],
                                    vec![0xf0, 0x90, 0xf0, 0x90, 0x90],
                                    vec![0xe0, 0x90, 0xe0, 0x90, 0xe0],
                                    vec![0xf0, 0x80, 0x80, 0x80, 0xf0],
                                    vec![0xe0, 0x90, 0x90, 0x90, 0xe0],
                                    vec![0xf0, 0x80, 0xf0, 0x80, 0xf0],
                                    vec![0xf0, 0x80, 0xf0, 0x80, 0x80]];
    
    // store each number n at 0xn0 - 0xn4
    for (j, d) in digits.iter().enumerate() {
        for (k, b) in d.iter().enumerate() {
            ram[(0x10 * j) + k] = *b;
        }
    }
}

fn main() {
    println!("chip8-rust: CHIP-8 emulator written in Rust");

    let args: Vec<String> = env::args().collect();

    let filename = String::from(&args[1]);

    let rom = match fs::read(&filename) {
        Err(why) => panic!("Could not open file: {}", why.description()),
        Ok(file) => file
    };

    // setup audio
    let audio_device = rodio::default_output_device().unwrap();
    let audio_sink = Sink::new(&audio_device);
    let audio_source = rodio::source::SineWave::new(440);
    audio_sink.append(audio_source);
    audio_sink.pause();

    // setup memory
    let mut ram: Vec<u8> = vec![0; RAM_SIZE]; // ram
    let mut display: Vec<u32> = vec![PX_OFF; WIDTH * HEIGHT]; // display
    let mut v: Vec<u8> = vec![0; REGISTER_COUNT]; // registers
    let mut i: usize = 0; // memory address register
    let mut dt: u8 = 0; // delay timer
    let mut st: u8 = 0; // sound timer
    let mut stack: Vec<usize> = Vec::new();
    let mut pc: usize = 0x200; // program counter

    preload_ram(&mut ram);

    for (j, c) in rom.into_iter().enumerate() {
        if j >= RAM_SIZE {
            panic!("Out of memory: program too large");
        }
        println!("Byte {:02}: {:#04x}", j, c);
        ram[j + 512] = c;
    }

    let mut window = Window::new(
        &format!("chip8-rust: {}", filename),
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: Scale::X8,
            ..WindowOptions::default()
        }
    ).unwrap();

    // 480 Hz
    window.limit_update_rate(Some(std::time::Duration::from_micros(2083)));

    let mut executing = true;
    let mut waiting_for_keypress = false;
    let mut store_keypress_in: usize = 0x0;
    // run once every 8 iterations, ie. 60Hz
    let mut time_to_runloop: usize = 8;

    while window.is_open() && 
            !window.is_key_down(Key::Escape) &&
            pc <= RAM_SIZE {
        //for (i, pixel) in display.iter_mut().enumerate() {
        //    *pixel = if ram[i + 512] == 0 { PX_OFF } else { PX_ON };
        //}

        let keys_pressed = handle_key_events(&window);

        for (j, k) in keys_pressed.iter().enumerate() {
            if *k {
                if waiting_for_keypress {
                    executing = true;
                    waiting_for_keypress = false;
                    v[store_keypress_in] = j as u8;
                    break;
                }
                println!("{:01x} pressed!", j);
            }
        }

        // get the instruction (2 bytes) out of RAM
        let b1 = ram[pc] as u16;
        let b2 = ram[pc + 1] as u16;
        let instruction = (b1 * 256) + b2;
        
        // flag to keep track of whether to move to next instruction
        // or not; in most cases we will, but sometimes not
        let mut next_instruction = true;

        println!("{:03x}, {:04x}, {:04x}, {:02x?}", pc, instruction, i, v);

        if executing {
            // all instruction comments below will follow the format wxyz for
            // referring to instruction
            match instruction {
                0x00e0 => {
                    // clear display
                    for j in 0..display.len() {
                        display[j] = PX_OFF;
                    }
                },
                0x00ee => {
                    // return from subroutine and panic if no subroutine to return from
                    pc = stack.pop().expect("Stack empty, cannot return from subroutine!");
                },
                0x1000..=0x1fff => {
                    // jump to memory location xyz
                    pc = get_hex_digits(&instruction, 3, 0);
                    next_instruction = false;
                },
                0x2000..=0x2fff => {
                    // call memory location xyz as subroutine (that will eventually return)
                    let loc = get_hex_digits(&instruction, 3, 0);
                    stack.push(pc);
                    pc = loc;
                    next_instruction = false;
                },
                0x3000..=0x3fff => {
                    // skip next instruction if Vx == yz
                    let val = get_hex_digits(&instruction, 2, 0);
                    let reg = get_hex_digits(&instruction, 1, 2);
                    if v[reg] == val as u8 {
                        pc += 2;
                    }
                },
                0x4000..=0x4fff => {
                    // skip next instruction if Vx != yz
                    let val = get_hex_digits(&instruction, 2, 0);
                    let reg = get_hex_digits(&instruction, 1, 2);
                    if v[reg] != val as u8 {
                        pc += 2;
                    }
                },
                0x5000..=0x5fff => {
                    // skip next instruction if Vx == Vy
                    let reg1 = get_hex_digits(&instruction, 1, 2);
                    let reg2 = get_hex_digits(&instruction, 1, 1);
                    if v[reg1] == v[reg2] {
                        pc += 2;
                    }
                },
                0x6000..=0x6fff => {
                    // load value yz into Vx
                    let val = get_hex_digits(&instruction, 2, 0);
                    let reg = get_hex_digits(&instruction, 1, 2);
                    v[reg] = val as u8;
                },
                0x7000..=0x7fff => {
                    // add value yz to Vx
                    let val = get_hex_digits(&instruction, 2, 0);
                    let reg = get_hex_digits(&instruction, 1, 2);
                    // we need to ignore overflows in adding in this case
                    v[reg] = v[reg].overflowing_add(val as u8).0;
                },
                0x8000..=0x8fff => {
                    // this seems to be a wrapper for all sorts
                    // of binary operations on Vx and Vy determined by z
                    let lsb = get_hex_digits(&instruction, 1, 0);
                    let reg1 = get_hex_digits(&instruction, 1, 2);
                    let reg2 = get_hex_digits(&instruction, 1, 1);

                    match lsb {
                        0x0 => {
                            // set Vx = Vy
                            v[reg1] = v[reg2];
                        },
                        0x1 => {
                            // set Vx = Vx OR Vy
                            v[reg1] |= v[reg2];
                        },
                        0x2 => {
                            // set Vx = Vx AND Vy
                            v[reg1] &= v[reg2];
                        },
                        0x3 => {
                            // set Vx = Vx XOR Vy
                            v[reg1] ^= v[reg2];
                        },
                        0x4 => {
                            // set Vx = Vx + Vy (and VF to 1 if overflow else 0)
                            let (res, over) = v[reg1].overflowing_add(v[reg2]);
                            v[reg1] = res;
                            v[0xf] = if over {1} else {0};
                        },
                        0x5 => {
                            // set Vx = Vx - Vy (and VF to 0 if borrow else 1)
                            let (res, over) = v[reg1].overflowing_sub(v[reg2]);
                            v[reg1] = res;
                            v[0xf] = if over {0} else {1};
                        },
                        0x6 => {
                            // right shift Vx 1 bit (and VF to value of bit lost)
                            let res = v[reg1].overflowing_shr(1).0;
                            v[0xf] = get_bit(&v[reg1], 0);
                            v[reg1] = res;
                        },
                        0x7 => {
                            // set Vx = Vy - Vx (and VF to 0 if borrow else 1)
                            let (res, over) = v[reg2].overflowing_sub(v[reg1]);
                            v[reg1] = res;
                            v[0xf] = if over {0} else {1};
                        },
                        0xe => {
                            // left shift Vx 1 bit (and VF to value of bit lost)
                            let res = v[reg1].overflowing_shl(1).0;
                            v[0xf] = get_bit(&v[reg1], 7);
                            v[reg1] = res;
                        },
                        _ => {
                            println!("Warning: unrecognized instruction: {:04x}", instruction);
                        }
                    };
                },
                0x9000..=0x9fff => {
                    // skip next instruction if Vx != Vy
                    let reg1 = get_hex_digits(&instruction, 1, 2);
                    let reg2 = get_hex_digits(&instruction, 1, 1);
                    if v[reg1] != v[reg2] {
                        pc += 2;
                    }
                },
                0xa000..=0xafff => {
                    // load value xyz into register I
                    i = get_hex_digits(&instruction, 3, 0);
                },
                0xb000..=0xbfff => {
                    // jump to memory location xyz + V0
                    pc = get_hex_digits(&instruction, 3, 0) + v[0] as usize;
                    next_instruction = false;
                },
                0xc000..=0xcfff => {
                    // set Vx = random byte AND yz
                    let rnd = rand::random::<u8>();
                    let val = get_hex_digits(&instruction, 2, 0);
                    let reg = get_hex_digits(&instruction, 1, 2);
                    v[reg] = rnd & val as u8;
                },
                0xd000..=0xdfff => {
                    // get z bytes and draw them starting at (Vx, Vy)
                    let reg1 = get_hex_digits(&instruction, 1, 2);
                    let reg2 = get_hex_digits(&instruction, 1, 1);
                    let init_x = v[reg1];
                    let init_y = v[reg2];
                    let mut byte_count = get_hex_digits(&instruction, 1, 0);
                    let mut bytes_to_print: Vec<u8> = Vec::new();
                    let mut j = 0;
                    let mut collision: u8 = 0;
                    while byte_count > 0 {
                        bytes_to_print.push(ram[i + j]);
                        byte_count -= 1;
                        j += 1;
                    }
                    for (k, b) in bytes_to_print.iter().enumerate() {
                        for j in 0..8 {
                            let x = (init_x as usize + k) % WIDTH;
                            let y = (init_y as usize + j) % HEIGHT;
                            let coord = (y * WIDTH) + x;
                            let is_old_set = display[coord] == PX_ON;
                            // xor pixels, if existing bit erased then set collision bit to true
                            display[coord] = if is_bit_set(b, j as u8) {
                                if is_old_set { collision = 1; PX_OFF }
                                else { PX_ON }
                            }
                            else {
                                if is_old_set { PX_ON }
                                else { PX_OFF }
                            };
                            v[0xf] = collision;
                        }
                    }
                },
                0xe000..=0xff65 => {
                    // these last few instructions are a bit arbitrarily named
                    // so let's check each nibble individually
                    let d1 = get_hex_digits(&instruction, 1, 3);
                    let d2 = get_hex_digits(&instruction, 1, 2);
                    let d3 = get_hex_digits(&instruction, 1, 1);
                    let d4 = get_hex_digits(&instruction, 1, 0);

                    if d1 == 0xe && d3 == 0x9 && d4 == 0xe {
                        // skip instruction if keycode Vx is pressed
                        if keys_pressed[v[d2] as usize] {
                            pc += 2;
                        }
                    }

                    else if d1 == 0xe && d3 == 0xa && d4 == 0x1 {
                        // skip instruction if keycode Vx is not pressed
                        if !keys_pressed[v[d2] as usize] {
                            pc += 2;
                        }
                    }

                    else if d1 == 0xf && d3 == 0x0 && d4 == 0x7 {
                        // set Vx to delay timer value
                        v[d2] = dt;
                    }

                    else if d1 == 0xf && d3 == 0x0 && d4 == 0xa {
                        // stop execution until keypress
                        executing = false;
                        waiting_for_keypress = true;
                        store_keypress_in = d2;
                    }

                    else if d1 == 0xf && d3 == 0x1 && d4 == 0x5 {
                        // set delay timer value to Vx
                        dt = v[d2];
                    }

                    else if d1 == 0xf && d3 == 0x1 && d4 == 0x8 {
                        // set sound timer value to Vx
                        st = v[d2];
                    }

                    else if d1 == 0xf && d3 == 0x1 && d4 == 0xe {
                        // i += Vx
                        i += v[d2] as usize;
                    }

                    else if d1 == 0xf && d3 == 0x2 && d4 == 0x9 {
                        // set i = location of sprite representing
                        // digit Vx in memory
                        i = (0x10 * v[d2]) as usize;
                    }

                    else if d1 == 0xf && d3 == 0x3 && d4 == 0x3 {
                        // store digits of Vx in memory locations
                        // i (hundreds), i+1 (tens), i+2 (ones)
                        ram[i] = v[d2] / 100;
                        ram[i+1] = (v[d2] % 100) / 10;
                        ram[i+2] = v[d2] % 10;
                    }

                    else if d1 == 0xf && d3 == 0x5 && d4 == 0x5 {
                        // store [V0, Vx] in memory locations [i, i+x]
                        for j in 0..=d2 {
                            ram[i+j] = v[j];
                        }
                    }

                    else if d1 == 0xf && d3 == 0x6 && d4 == 0x5 {
                        // load [V0, Vx] from memory locations [i, i+x]
                        for j in 0..=d2 {
                            v[j] = ram[i+j];
                        }
                    }
                    
                    else {
                        println!("Warning: unrecognized instruction: {:04x}", instruction);
                    }
                },
                _ => {
                    println!("Warning: unrecognized instruction: {:04x}", instruction);
                }
            };

            // update program counter if necessary
            if next_instruction {
                pc += 2;
            }
        }

        if time_to_runloop == 0 {
            if dt > 0 { dt -= 1; }
            
            if st > 0 {
                audio_sink.play();
                st -= 1;
            }
            else if st == 0 {
                audio_sink.pause();
            }
            
            window.update_with_buffer(&display, WIDTH, HEIGHT).unwrap();
            
            time_to_runloop = 4;
        }
        else {
            time_to_runloop -= 1;
        }
    }
}
