extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Renderer, Texture};

pub struct OffscreenBuffer<'a> {
    renderer: Renderer<'a>,
    buffer: Texture,
    width: i32,
    height: i32,
}

impl<'a> OffscreenBuffer<'a> {
    pub fn new(renderer: Renderer<'a>, width: u32, height: u32) -> OffscreenBuffer<'a> {
        OffscreenBuffer {
            buffer: renderer.create_texture_streaming(PixelFormatEnum::RGBA8888, (width, height))
                            .unwrap(),
            renderer: renderer,
            width: width as i32,
            height: height as i32,
        }
    }

    pub fn present(&mut self, window_width: u32, window_height: u32) {
        // self.renderer.clear();
        self.renderer.copy(&self.buffer,
                           None,
                           Rect::new(0, 0, window_width, window_height).unwrap());
        self.renderer.present();
    }

    pub fn hline(&mut self, y: i32, mut x_min: i32, mut x_max: i32, color: RGBA) {
        if y < 0 || y >= self.height {
            return;
        }

        x_min = self.clamp_x(x_min);
        x_max = self.clamp_x(x_max);

        self.buffer
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                unsafe {
                    let mut row = buffer.as_mut_ptr();
                    row = row.offset((y as usize * pitch) as isize);
                    let mut pixel = row as *mut u32;
                    pixel = pixel.offset(x_min as isize);
                    for _ in 0..(x_max - x_min) {
                        *pixel = color.into_u32();
                        pixel = pixel.offset(1);
                    }
                }
            })
            .unwrap();
    }

    pub fn vline(&mut self, x: i32, mut y_min: i32, mut y_max: i32, color: RGBA) {
        if x < 0 || x >= self.width {
            return;
        }

        y_min = self.clamp_y(y_min);
        y_max = self.clamp_y(y_max);

        self.buffer
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                unsafe {
                    let mut col = buffer.as_mut_ptr();
                    col = (col as *mut u32).offset(x as isize) as *mut u8;
                    col = col.offset((y_min as usize * pitch) as isize);
                    for _ in y_min..y_max {
                        let mut pixel = col as *mut u32;
                        *pixel = color.into_u32();
                        col = col.offset(pitch as isize);
                    }
                }
            })
            .unwrap();
    }

    pub fn rect(&mut self, x_min: i32, y_min: i32, x_max: i32, y_max: i32, color: RGBA) {
        self.hline(y_min, x_min, x_max, color);
        self.hline(y_max, x_min, x_max, color);
        self.vline(x_min, y_min, y_max, color);
        self.vline(x_max, y_min, y_max, color);
    }

    fn clamp_x(&self, x: i32) -> i32 {
        if x < 0 {
            0
        } else if x >= self.width {
            self.width - 1
        } else {
            x
        }
    }

    fn clamp_y(&self, y: i32) -> i32 {
        if y < 0 {
            0
        } else if y >= self.height {
            self.height - 1
        } else {
            y
        }
    }
}

#[derive(Clone, Copy)]
pub struct RGBA {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl RGBA {
    pub fn into_u32(self) -> u32 {
        // NOTE(coeuvre): The pixel format is as following:
        //   - In memory: AA BB GG RR
        //   - In register: 0xRRGGBBAA
        let r = (self.r * 255.0) as u8 as u32;
        let g = (self.g * 255.0) as u8 as u32;
        let b = (self.b * 255.0) as u8 as u32;
        let a = (self.a * 255.0) as u8 as u32;
        (r << 24) | (g << 16) | (b << 8) | a
    }
}

pub struct Playfield {
    width: i32,
    height: i32,
}

impl Playfield {
    pub fn new(width: i32, height: i32) -> Playfield {
        Playfield {
            width: width,
            height: height,
        }
    }

    pub fn draw(&self, renderer: &mut OffscreenBuffer, x: i32, y: i32) {
        let block_size_in_pixels = 32;
        let color = RGBA {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };

        let width = self.width * block_size_in_pixels;
        let height = self.height * block_size_in_pixels;

        renderer.rect(x, y, x + width, y + height, color);

        let color = RGBA {
            r: 0.6,
            g: 0.6,
            b: 0.6,
            a: 1.0,
        };

        for row in 1..self.height {
            let y_offset = row * block_size_in_pixels;
            renderer.hline(y + y_offset, x, x + width, color);
        }

        for col in 1..self.width {
            let x_offset = col * block_size_in_pixels;
            renderer.vline(x + x_offset, y, y + height, color);
        }
    }
}

// pub struct PlayfieldBuilder {
// width: i32,
// height: i32,
// margin_x: i32,
// margin_y: i32,
// }
//
// impl PlayfieldBuilder {
// pub fn new() -> PlayfieldBuilder {
// PlayfieldBuilder {
// width: 0,
// width: 0,
// height: 0,
// margin_x: 0,
// margin_y: 0,
// }
// }
//
// pub fn size(self, width: i32, height: i32) -> PlayfieldBuilder {
// self.width = width;
// self.height = height;
// self
// }
//
// pub fn margin(self, x: i32, y: i32) -> PlayfieldBuilder {
// self.margin_x = x;
// self.margin_y = y;
// self
// }
// }
//

fn main() {
    let sdl2 = sdl2::init().unwrap();
    let video = sdl2.video().unwrap();

    let width = 600;
    let height = 800;
    let window = video.window("Retris", width, height)
                      .position_centered()
                      .opengl()
                      .build()
                      .unwrap();

    let renderer = window.renderer().build().unwrap();

    let mut buffer = OffscreenBuffer::new(renderer, width, height);

    let mut event_pump = sdl2.event_pump().unwrap();

    let playfield = Playfield::new(10, 20);

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} => break 'running,
                _ => {}
            }
        }

        playfield.draw(&mut buffer, 64, 32);
        buffer.present(width, height);
    }
}
