extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Renderer, Texture};

struct OffscreenBuffer<'a> {
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

    pub fn render_weird_graient(&mut self, x_offset: i32, y_offset: i32) {
        let width = self.width;
        let height = self.height;
        self.buffer
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                unsafe {
                    let mut row = buffer.as_mut_ptr();
                    for y in 0..height {
                        let mut pixel = row as *mut u32;
                        for x in 0..width {
                            // NOTE(coeuvre): The pixel format is as following:
                            //   - In memory: AA BB GG RR
                            //   - In register: 0xRRGGBBAA
                            let r = (x as i32 + x_offset) as u8 as u32;
                            let g = (y as i32 + y_offset) as u8 as u32;
                            *pixel = (r << 24) | (g << 16);
                            pixel = pixel.offset(1);
                        }
                        row = row.offset(pitch as isize);
                    }
                }
            })
            .unwrap();

    }

    pub fn draw_hline(&mut self, mut y: i32, mut x_min: i32, mut x_max: i32, color: u32) {
        y = self.clamp_y(y);
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
                        *pixel = color;
                        pixel = pixel.offset(1);
                    }
                }
            })
            .unwrap();
    }

    pub fn draw_vline(&mut self, mut x: i32, mut y_min: i32, mut y_max: i32, color: u32) {
        x = self.clamp_x(x);
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
                        *pixel = color;
                        col = col.offset(pitch as isize);
                    }
                }
            })
            .unwrap();
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

fn main() {
    let sdl2 = sdl2::init().unwrap();
    let video = sdl2.video().unwrap();

    let width = 800;
    let height = 600;
    let window = video.window("Retris", width, height)
                      .position_centered()
                      .opengl()
                      .build()
                      .unwrap();

    let renderer = window.renderer().build().unwrap();

    let mut buffer = OffscreenBuffer::new(renderer, width, height);

    let mut event_pump = sdl2.event_pump().unwrap();

    let mut x_offset = 0;
    let mut y_offset = 0;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} => break 'running,
                _ => {}
            }
        }

        buffer.render_weird_graient(x_offset, y_offset);
        buffer.draw_hline(10, 100, width as i32, 0x00000000);
        buffer.draw_vline(100, 10, height as i32, 0x00000000);
        buffer.present(width, height);

        x_offset += 1;
        y_offset += 2;
    }
}
