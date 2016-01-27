use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Renderer, Texture};

use bitmap::Bitmap;

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
    pixels: Vec<u32>,
    width: i32,
    height: i32,
}

impl<'a> SoftwareRenderer<'a> {
    pub fn new(renderer: Renderer<'a>, width: u32, height: u32) -> SoftwareRenderer<'a> {
        SoftwareRenderer {
            buffer: renderer.create_texture_streaming(PixelFormatEnum::RGBA8888, (width, height))
                            .unwrap(),
            sdl_renderer: renderer,
            pixels: vec![0; (width * height) as usize],
            width: width as i32,
            height: height as i32,
        }
    }

    pub fn present(&mut self, window_width: u32, window_height: u32) {
        self.buffer
            .update(None,
                    unsafe {
                        ::std::slice::from_raw_parts(self.pixels.as_ptr() as *const u8,
                                                     self.pixels.len())
                    },
                    self.width as usize * 4)
            .unwrap();
        self.sdl_renderer.copy(&self.buffer,
                               None,
                               Rect::new(0, 0, window_width, window_height).unwrap());
        self.sdl_renderer.present();
        for pixel in &mut self.pixels {
            *pixel = 0;
        }
    }

    pub fn hline(&mut self, y: i32, mut x_min: i32, mut x_max: i32, color: RGBA) {
        if y < 0 || y >= self.height {
            return;
        }

        x_min = clamp(x_min, 0, self.width);
        x_max = clamp(x_max, 0, self.width);

        for x in x_min..x_max {
            let y = self.height - y - 1;
            self.pixels[(y * self.width + x) as usize] = color.into_u32();
        }
    }

    pub fn vline(&mut self, x: i32, mut y_min: i32, mut y_max: i32, color: RGBA) {
        if x < 0 || x >= self.width {
            return;
        }

        y_min = clamp(y_min, 0, self.height);
        y_max = clamp(y_max, 0, self.height);

        for y in y_min..y_max {
            let y = self.height - y - 1;
            self.pixels[(y * self.width + x) as usize] = color.into_u32();
        }
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

        for y in y_min..y_max {
            for x in x_min..x_max {
                let y = self.height - y - 1;
                self.pixels[(y * self.width + x) as usize] = color.into_u32();
            }
        }
    }

    pub fn blit_bitmap(&mut self, x: i32, y: i32, bitmap: &Bitmap) {
        self.blit_sub_bitmap(x, y, 0, 0, bitmap.width() as i32, bitmap.height() as i32, bitmap);
    }

    pub fn blit_sub_bitmap_alpha(&mut self, dst_x: i32, dst_y: i32,
                           src_x: i32, src_y: i32, width: i32, height: i32,
                           bitmap: &Bitmap, alpha: f32) {
        // TODO(coeuvre): Clip

        assert!(dst_x >= 0 && dst_x < self.width);
        assert!(dst_y >= 0 && dst_y < self.height);

        let x_min = dst_x;
        let x_max = dst_x + width as i32;
        let y_min = dst_y;
        let y_max = dst_y + height as i32;

        for y in y_min..y_max {
            for x in x_min..x_max {
                let dst = self.get(x, y);
                let src = bitmap.get((src_x + (x - x_min)) as u32, (src_y + (y - y_min)) as u32);
                let r = alpha * src.r + (1.0 - alpha) * dst.r;
                let g = alpha * src.g + (1.0 - alpha) * dst.g;
                let b = alpha * src.b + (1.0 - alpha) * dst.b;
                let a = src.a;
                self.set(x, y, rgba(r, g, b, a));
            }
        }
    }

    pub fn blit_sub_bitmap(&mut self, dst_x: i32, dst_y: i32,
                           src_x: i32, src_y: i32, width: i32, height: i32,
                           bitmap: &Bitmap) {
        self.blit_sub_bitmap_alpha(dst_x, dst_y, src_x, src_y, width, height, bitmap, 1.0);
    }

    pub fn get(&mut self, x: i32, y: i32) -> RGBA {
        let y = self.height - y - 1;
        RGBA::from_u32(self.pixels[(y * self.width + x) as usize])
    }

    pub fn set(&mut self, x: i32, y: i32, color: RGBA) {
        let y = self.height - y - 1;
        self.pixels[(y * self.width + x) as usize] = color.into_u32();
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

    pub fn from_u32(color: u32) -> RGBA {
        let r = ((color & 0xFF000000) >> 24) as u8;
        let g = ((color & 0x00FF0000) >> 16) as u8;
        let b = ((color & 0x0000FF00) >> 8) as u8;
        let a = ((color & 0x000000FF) >> 0) as u8;
        rgba(r, g, b, a)
    }
}

pub fn rgba<C: IntoColorComponent>(r: C, g: C, b: C, a: C) -> RGBA {
    RGBA {
        r: r.into_color_component(),
        g: g.into_color_component(),
        b: b.into_color_component(),
        a: a.into_color_component(),
    }
}

pub trait IntoColorComponent {
    fn into_color_component(self) -> f32;
}

impl IntoColorComponent for f32 {
    fn into_color_component(self) -> f32 {
        self
    }
}

impl IntoColorComponent for u8 {
    fn into_color_component(self) -> f32 {
        self as f32 / 255.0
    }
}
