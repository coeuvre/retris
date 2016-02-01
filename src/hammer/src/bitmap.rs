// This implementation use [https://upload.wikimedia.org/wikipedia/commons/c/c4/BMPfileFormat.png]
// as reference.

use std::fs::File;
use std::io::Read;
use std::mem;
use std::path::Path;

use renderer::{rgba, RGBA};

pub struct Bitmap {
    width: u32,
    height: u32,
    pixels: *const u32,

    _raw: Vec<u8>,
}

impl Bitmap {
    pub fn open<P: AsRef<Path>>(path: P) -> Option<Bitmap> {
        let mut file = File::open(path).unwrap();
        let mut buf = vec![];
        file.read_to_end(&mut buf).unwrap();

        unsafe {
            let raw = buf.as_ptr();
            let header: &BitmapHeader = mem::transmute(raw);
            assert!((header.sig & 0xFF) as u8 as char == 'B');
            assert!(((header.sig & 0xFF00) >> 8) as u8 as char == 'M');
            assert!(header.header_size >= 40);
            // TODO(coeuvre): Only supoort 32-bit color for now!
            assert!(header.bits_per_pixel == 32);

            Some(Bitmap {
                width: header.width,
                height: header.height,
                pixels: raw.offset(header.offset as isize) as *const u32,
                _raw: buf,
            })
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn get(&self, x: u32, y: u32) -> RGBA {
        assert!(x < self.width);
        assert!(y < self.height);
        unsafe {
            // TODO(coeuvre): Assuming the pixels format is 0xRGBA
            // and origin is at left-bottom corner.
            let pixels = *self.pixels.offset((y * self.width + x) as isize);
            let r = ((pixels & 0xFF000000) >> 24) as u8;
            let g = ((pixels & 0x00FF0000) >> 16) as u8;
            let b = ((pixels & 0x0000FF00) >> 8) as u8;
            let a = ((pixels & 0x000000FF) >> 0) as u8;
            rgba(r, g, b, a)
        }
    }
}

#[repr(C, packed)]
struct BitmapHeader {
    /// The characters "BM"
    sig: u16,
    /// The size of the file in bytes
    file_size: u32,
    /// Unused - must be zero
    reserved1: u16,
    /// Unused - must be zero
    reserved2: u16,
    /// Offset to start of Pixel Data
    offset: u32,

    /// Header Size - Must be at least 40
    header_size: u32,
    /// Image width in pixels
    width: u32,
    /// Image height in pixels
    height: u32,
    /// Must be 1
    planes: u16,
    /// Bits per pixel - 1, 4, 8, 16, 24, or 32
    bits_per_pixel: u16,
    /// Compression type (0 = uncompressed)
    compression: u32,
    /// Image Size - may be zero for uncompressed images
    image_size: u32,
    /// Preferred resolution in pixels per meter
    x_pixels_per_meter: u32,
    /// Preferred resolution in pixels per meter
    y_pixels_per_meter: u32,
    /// Number Color Map entries that are actually used
    colors_used: u32,
    /// Number of significant colors
    colors_important: u32,
    /*
    red_mask: u32,
    glue_mask: u32,
    blue_mask: u32,
    alpha_mask: u32,
    */
}
