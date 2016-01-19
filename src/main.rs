#![feature(custom_attribute, stmt_expr_attributes)]

extern crate rand;
extern crate sdl2;
extern crate time;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use time::PreciseTime;

use renderer::*;

mod renderer;

// TODO(coeuvre): Placeholder type for Block.
pub type Block = ();

pub struct Blocks {
    width: usize,
    height: usize,
    data: Vec<Option<Block>>,
}

impl Blocks {
    pub fn new(width: usize, height: usize) -> Blocks {
        Blocks {
            width: width,
            height: height,
            data: vec![None; width * height],
        }
    }

    pub fn from_data(width: usize, height: usize, data: Vec<Option<Block>>) -> Blocks {
        assert!(data.len() == width * height);
        Blocks {
            width: width,
            height: height,
            data: data,
        }
    }
}

pub struct BlocksTemplate {
    templates: Vec<Vec<Blocks>>,
}

impl BlocksTemplate {
    pub fn new() -> BlocksTemplate {
        let width = 4;
        let height = 4;
        // NOTE(coeuvre): Bitmap data for blocks. The origin is left-bottom corner.
        //
        //   x x x x
        //   x x x x
        //   x x x x
        //   o x x x
        //
        #[rustfmt_skip]
        BlocksTemplate {
            templates: vec![
                // Block I
                vec![
                    Blocks::from_data(width, height, vec![
                        None, Some(()), None, None,
                        None, Some(()), None, None,
                        None, Some(()), None, None,
                        None, Some(()), None, None,
                    ]),
                    Blocks::from_data(width, height, vec![
                        Some(()),  Some(()),  Some(()),  Some(()),
                        None, None, None, None,
                        None, None, None, None,
                        None, None, None, None,
                    ]),
                ],

                // Block O
                vec![
                    Blocks::from_data(width, height, vec![
                        None, Some(()), Some(()), None,
                        None, Some(()), Some(()), None,
                        None, None, None, None,
                        None, None, None, None,
                    ]),
                ],
            ],
        }
    }

    pub fn generate(&self) -> BlocksTemplateRef {
        let shape = rand::random::<usize>() % self.templates.len();
        let order_max = self.templates[shape].len();
        let order = rand::random::<usize>() % order_max;

        BlocksTemplateRef {
            shape: shape,
            order: order,
            order_max: order_max,
        }
    }

    pub fn blocks(&self, r: &BlocksTemplateRef) -> &Blocks {
        &self.templates[r.shape][r.order]
    }
}

pub struct BlocksTemplateRef {
    shape: usize,
    order: usize,
    order_max: usize,
}

pub struct ActiveBlocks {
    template: BlocksTemplateRef,
    x: i32,
    y: i32,
}

impl ActiveBlocks {
    pub fn new(template: BlocksTemplateRef) -> ActiveBlocks {
        ActiveBlocks {
            template: template,
            x: 0,
            y: 0,
        }
    }

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

    pub fn transform(&mut self) {
        self.template.order = (self.template.order + 1) % self.template.order_max;
    }
}

pub struct Playfield {
    blocks: Blocks,

    blocks_template: BlocksTemplate,

    active_blocks: Option<ActiveBlocks>,

    interval: f32,
    time_remain: f32,
}

impl Playfield {
    pub fn new(width: usize, height: usize) -> Playfield {
        Playfield {
            blocks: Blocks::new(width, height),

            blocks_template: BlocksTemplate::new(),
            active_blocks: None,

            interval: 1.0,
            time_remain: 0.0,
        }
    }

    pub fn put_block(&mut self) {
        if self.active_blocks.is_some() {
            return;
        }

        let r = self.blocks_template.generate();
        let mut active_blocks = ActiveBlocks::new(r);
        active_blocks.move_to(3, 19);
        self.active_blocks = Some(active_blocks);
        self.time_remain = self.interval;
    }

    pub fn move_active_block_left(&mut self) {
        if let Some(ref mut active_blocks) = self.active_blocks {
            active_blocks.left();
        }
    }

    pub fn move_active_block_right(&mut self) {
        if let Some(ref mut active_blocks) = self.active_blocks {
            active_blocks.right();
        }
    }

    pub fn move_active_block_down(&mut self) {
        if let Some(ref mut active_blocks) = self.active_blocks {
            active_blocks.down();
        }
    }

    pub fn transform_active_block(&mut self) {
        if let Some(ref mut active_blocks) = self.active_blocks {
            active_blocks.transform();
        }
    }

    pub fn update(&mut self, renderer: &mut SoftwareRenderer, dt: f32, x: i32, y: i32) {
        self.time_remain -= dt;
        if self.time_remain < 0.0 {
            self.time_remain += self.interval;
            self.move_active_block_down();
        }

        let block_size_in_pixels = 32i32;

        let color = RGBA {
            r: 0.6,
            g: 0.6,
            b: 0.6,
            a: 1.0,
        };

        macro_rules! blocks_iter {
            ($blocks:expr) => {
                $blocks.data
                    .iter()
                    .enumerate()
                    .filter(|&(_, block)| block.is_some())
                    .map(|(i, block)| {
                        (i % $blocks.width, i / $blocks.width, block.unwrap())
                    })
            }
        }

        // Current block
        if let Some(ref mut active_blocks) = self.active_blocks {
            let blocks = self.blocks_template.blocks(&active_blocks.template);

            for (col, row, _) in blocks_iter!(blocks) {
                // Simply clip the blocks
                if active_blocks.y + (row as i32) < self.blocks.height as i32 - 2 {
                    let x_offset = (active_blocks.x + col as i32) * block_size_in_pixels;
                    let y_offset = (active_blocks.y + row as i32) * block_size_in_pixels;
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
        for (col, row, _) in blocks_iter!(self.blocks) {
            let x_offset = (col as i32) * block_size_in_pixels;
            let y_offset = (row as i32) * block_size_in_pixels;
            let x = x + x_offset;
            let y = y + y_offset;
            renderer.fill_rect(x + 1,
                               y + 1,
                               x + block_size_in_pixels,
                               y + block_size_in_pixels,
                               color);
        }

        // Grids
        let width = self.blocks.width as i32 * block_size_in_pixels;
        let height = (self.blocks.height - 2) as i32 * block_size_in_pixels;

        for row in 1..self.blocks.height - 2 {
            let y_offset = row as i32 * block_size_in_pixels;
            renderer.hline(y + y_offset, x, x + width, color);
        }

        for col in 1..self.blocks.width {
            let x_offset = col as i32 * block_size_in_pixels;
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


pub struct Retris {
    playfield: Playfield,
}

impl Retris {
    pub fn new() -> Retris {
        Retris { playfield: Playfield::new(10, 22) }
    }

    pub fn update(&mut self, renderer: &mut SoftwareRenderer, dt: f32) {
        self.playfield.update(renderer, dt, 64, 32);
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

    let renderer = window.renderer().present_vsync().build().unwrap();

    let mut renderer = SoftwareRenderer::new(renderer, width, height);

    let mut event_pump = sdl2.event_pump().unwrap();

    let mut retris = Retris::new();

    let mut frame_last = PreciseTime::now();

    'running: loop {
        let frame_start = PreciseTime::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} => break 'running,
                Event::KeyDown {keycode: Some(Keycode::Space), ..} => {
                    retris.playfield.put_block();
                }
                Event::KeyDown {keycode: Some(Keycode::Left), ..} => {
                    retris.playfield.move_active_block_left();
                }
                Event::KeyDown {keycode: Some(Keycode::Right), ..} => {
                    retris.playfield.move_active_block_right();
                }
                Event::KeyDown {keycode: Some(Keycode::Up), ..} => {
                    retris.playfield.transform_active_block();
                }
                _ => {}
            }
        }

        let now = PreciseTime::now();
        let dt = frame_last.to(now).num_milliseconds() as f32 / 1000.0;
        frame_last = now;
        retris.update(&mut renderer, dt);
        renderer.present(width, height);

        let frame_end = PreciseTime::now();
        let _ = frame_start.to(frame_end);
        // println!("FPS: {}", (1000.0 / span.num_milliseconds() as f64) as u32);
    }
}
