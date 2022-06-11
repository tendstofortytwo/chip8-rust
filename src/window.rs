use minifb::{
    Key,
    WindowOptions,
    Scale,
    Error
};

use crate::util::is_bit_set;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const PX_OFF: u32 = 0x81c784;
const PX_ON: u32 = 0x29302a;

pub struct Window {
    win: minifb::Window,
    framebuffer: [u32; WIDTH * HEIGHT]
}

impl Window {
    pub fn new(title: &str) -> Result<Window, Error> {
        let mut win = match minifb::Window::new(
            title,
            WIDTH,
            HEIGHT,
            WindowOptions {
                scale: Scale::X8,
                ..WindowOptions::default()
            }
        ) {
            Ok(win) => win,
            Err(err) => {
                return Err(err);
            }
        };
        // 480 Hz
        win.limit_update_rate(Some(std::time::Duration::from_micros(2083)));
        Ok(Window { win, framebuffer: [PX_OFF; WIDTH * HEIGHT] })
    }

    pub fn handle_key_events(&self) -> [bool; 16] {
        let mut keys = [false; 16];
        self.win.get_keys().iter().for_each(|k| {
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
        });
        keys
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        self.win.is_key_down(key)
    }

    pub fn is_open(&self) -> bool {
        self.win.is_open()
    }

    pub fn clear_screen(&mut self) {
        for j in 0..self.framebuffer.len() {
            self.framebuffer[j] = PX_OFF;
        }
    }

    pub fn draw(&mut self, bytes: &Vec<u8>, init_x: u8, init_y: u8) -> u8 {
        let mut collision: u8 = 0;
        for (k, b) in bytes.iter().enumerate() {
            for j in 0..8 {
                let x = (init_x as usize + j) % WIDTH;
                let y = (init_y as usize + k) % HEIGHT;
                let coord = (y * WIDTH) + x;
                let is_old_set = self.framebuffer[coord] == PX_ON;
                // xor pixels bits only if they are set
                // if existing bit erased then set collision bit to true
                self.framebuffer[coord] = if is_bit_set(b, (8-j-1) as u8) {
                    if is_old_set { collision = 1; PX_OFF }
                    else { PX_ON }
                } else { self.framebuffer[coord] };
            }
        }
        collision
    }

    pub fn refresh(&mut self) {
        self.win.update_with_buffer(&self.framebuffer, WIDTH, HEIGHT).unwrap();
    }
}
