extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Renderer, Texture};

struct OffscreenBuffer<'a> {
    renderer: Renderer<'a>,
    buffer: Texture,
    width: u32,
    height: u32,
}

impl<'a> OffscreenBuffer<'a> {
    pub fn new(renderer: Renderer<'a>, width: u32, height: u32) -> OffscreenBuffer<'a> {
        OffscreenBuffer {
            buffer: renderer.create_texture_streaming(PixelFormatEnum::RGBA8888, (width, height))
                            .unwrap(),
            renderer: renderer,
            width: width,
            height: height,
        }
    }

    pub fn present(&mut self, window_width: u32, window_height: u32) {
        self.renderer.clear();
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
        buffer.present(width, height);

        x_offset += 1;
        y_offset += 2;
    }
}
