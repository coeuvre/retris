#![feature(custom_attribute, stmt_expr_attributes)]

extern crate rand;
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use renderer::*;

mod renderer;

pub struct BlockGenerator {
    template: Vec<Vec<Vec<bool>>>,
}

impl BlockGenerator {
    pub fn new() -> BlockGenerator {
        // NOTE(coeuvre): Bitmap data for blocks. The origin is left-bottom corner.
        //
        //   x x x x
        //   x x x x
        //   x x x x
        //   o x x x
        //
        #[rustfmt_skip]
        BlockGenerator {
            template: vec![
                // Block I
                vec![
                    vec![
                        false, true, false, false,
                        false, true, false, false,
                        false, true, false, false,
                        false, true, false, false,
                    ],
                    vec![
                        false, false, false, false,
                        true,  true,  true,  true,
                        false, false, false, false,
                        false, false, false, false,
                    ],
                ],

                // Block O
                vec![
                    vec![
                        false, false, false, false,
                        false, true, true, false,
                        false, true, true, false,
                        false, false, false, false,
                    ],
                ],
            ],
        }
    }

    pub fn generate(&self) -> Block {
        let shape = rand::random::<usize>() % self.template.len();
        let order = rand::random::<usize>() % self.template[shape].len();

        Block {
            x: 0,
            y: 0,
            shape: shape,
            order: order,
        }
    }

    pub fn data(&self, shape: usize, order: usize) -> &Vec<bool> {
        &self.template[shape][order]
    }
}

pub struct Block {
    x: i32,
    y: i32,
    shape: usize,
    order: usize,
}

impl Block {
    pub fn move_to(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn down(&mut self) {
        self.y -= 1;
    }

    pub fn left(&mut self) {
        self.x -= 1;
    }

    pub fn right(&mut self) {
        self.x += 1;
    }

    pub fn width(&self) -> usize {
        4
    }

    pub fn height(&self) -> usize {
        4
    }
}

pub struct Playfield {
    width: i32,
    height: i32,
    static_blocks: Vec<bool>,

    generator: BlockGenerator,
    current_block: Option<Block>,
}

impl Playfield {
    pub fn new(width: i32, height: i32) -> Playfield {
        Playfield {
            width: width,
            height: height,
            static_blocks: vec![false; (width * height) as usize],

            generator: BlockGenerator::new(),
            current_block: None,
        }
    }

    pub fn put_block(&mut self) {
        if self.current_block.is_some() {
            return;
        }

        let block = self.generator.generate();
        self.current_block = Some(block);
    }

    pub fn render(&mut self, renderer: &mut SoftwareRenderer, x: i32, y: i32) {
        let block_size_in_pixels = 32;

        let width = self.width * block_size_in_pixels;
        let height = self.height * block_size_in_pixels;

        let color = RGBA {
            r: 0.6,
            g: 0.6,
            b: 0.6,
            a: 1.0,
        };

        // Current block
        if let Some(ref current_block) = self.current_block {
            for (i, block) in self.generator
                                  .data(current_block.shape, current_block.order)
                                  .iter()
                                  .enumerate() {
                if *block {
                    let col = (i % current_block.width()) as i32;
                    let row = (i / current_block.height()) as i32;
                    let x_offset = (current_block.x + col) * block_size_in_pixels;
                    let y_offset = (current_block.y + row) * block_size_in_pixels;
                    let x = x + x_offset;
                    let y = y + y_offset;
                    renderer.fill_rect(x + 1,
                                       y + 1,
                                       x + block_size_in_pixels,
                                       y + block_size_in_pixels,
                                       color);
                }
            }
        }

        // Static blocks
        for (i, block) in self.static_blocks.iter().enumerate() {
            if *block {
                let col = i as i32 % self.width;
                let row = i as i32 / self.height;
                let x_offset = col * block_size_in_pixels;
                let y_offset = row * block_size_in_pixels;
                let x = x + x_offset;
                let y = y + y_offset;
                renderer.fill_rect(x + 1,
                                   y + 1,
                                   x + block_size_in_pixels,
                                   y + block_size_in_pixels,
                                   color);
            }
        }

        // Grids
        for row in 1..self.height {
            let y_offset = row * block_size_in_pixels;
            renderer.hline(y + y_offset, x, x + width, color);
        }

        for col in 1..self.width {
            let x_offset = col * block_size_in_pixels;
            renderer.vline(x + x_offset, y, y + height, color);
        }

        // Border
        let color = RGBA {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };

        renderer.rect(x, y, x + width, y + height, color);

    }
}

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

    let mut renderer = SoftwareRenderer::new(renderer, width, height);

    let mut event_pump = sdl2.event_pump().unwrap();

    let mut playfield = Playfield::new(10, 22);

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} => break 'running,
                Event::KeyDown {keycode: Some(Keycode::Space), ..} => {
                    playfield.put_block();
                }
                _ => {}
            }
        }

        playfield.render(&mut renderer, 64, 32);
        renderer.present(width, height);
    }
}
