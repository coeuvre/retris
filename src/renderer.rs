use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Renderer, Texture};

fn clamp(value: i32, min: i32, max: i32) -> i32 {
    if value < min {
        0
    } else if value >= max {
        max - 1
    } else {
        value
    }
}

pub struct SoftwareRenderer<'a> {
    sdl_renderer: Renderer<'a>,
    buffer: Texture,
    width: i32,
    height: i32,
}

impl<'a> SoftwareRenderer<'a> {
    pub fn new(renderer: Renderer<'a>, width: u32, height: u32) -> SoftwareRenderer<'a> {
        SoftwareRenderer {
            buffer: renderer.create_texture_streaming(PixelFormatEnum::RGBA8888, (width, height))
                            .unwrap(),
            sdl_renderer: renderer,
            width: width as i32,
            height: height as i32,
        }
    }

    pub fn present(&mut self, window_width: u32, window_height: u32) {
        self.sdl_renderer.clear();
        self.sdl_renderer.copy(&self.buffer,
                               None,
                               Rect::new(0, 0, window_width, window_height).unwrap());
        self.sdl_renderer.present();
    }

    pub fn hline(&mut self, y: i32, mut x_min: i32, mut x_max: i32, color: RGBA) {
        if y < 0 || y >= self.height {
            return;
        }

        x_min = clamp(x_min, 0, self.width);
        x_max = clamp(x_max, 0, self.width);

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

        y_min = clamp(y_min, 0, self.height);
        y_max = clamp(y_max, 0, self.height);

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

    pub fn fill_rect(&mut self,
                     mut x_min: i32,
                     mut y_min: i32,
                     mut x_max: i32,
                     mut y_max: i32,
                     color: RGBA) {
        x_min = clamp(x_min, 0, self.width);
        x_max = clamp(x_max, 0, self.width);
        y_min = clamp(y_min, 0, self.height);
        y_max = clamp(y_max, 0, self.height);
        self.buffer
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                unsafe {
                    let mut row = buffer.as_mut_ptr();
                    row = row.offset((y_min as usize * pitch) as isize);
                    for _ in 0..(y_max - y_min) {
                        let mut pixel = row as *mut u32;
                        pixel = pixel.offset(x_min as isize);
                        for _ in 0..(x_max - x_min) {
                            *pixel = color.into_u32();
                            pixel = pixel.offset(1);
                        }
                        row = row.offset(pitch as isize);
                    }
                }
            })
            .unwrap();
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
