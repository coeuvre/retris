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

    pub fn get(&self, x: usize, y: usize) -> Option<&Block> {
        if x >= self.width || y >= self.height {
            return None;
        }

        self.data[y * self.width + x].as_ref()
    }

    pub fn set(&mut self, x: usize, y: usize, block: Option<Block>) {
        self.data[y * self.width + x] = block;
    }

    pub fn set_with_block(&mut self, x: usize, y: usize, block: Block) {
        self.set(x, y, Some(block));
    }

    pub fn set_with_blocks(&mut self, x: i32, y: i32, blocks: &Blocks) {
        for (col, row, block) in blocks_iter!(blocks) {
            self.set_with_block((x + col as i32) as usize, (y + row as i32) as usize, block);
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

    pub fn up(&mut self) {
        self.y += 1;
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

    pub fn set_active_blocks(&mut self) {
        if self.active_blocks.is_some() {
            return;
        }

        let r = self.blocks_template.generate();
        let mut active_blocks = ActiveBlocks::new(r);
        active_blocks.move_to(3, 19);
        self.active_blocks = Some(active_blocks);
        self.time_remain = self.interval;
    }

    pub fn move_active_blocks_left(&mut self) {
        if let Some(ref mut active_blocks) = self.active_blocks {
            active_blocks.left();

            let blocks = self.blocks_template.blocks(&active_blocks.template);
            let mut is_collide = false;
            let mut min_x = self.blocks.width as i32;
            for (col, row, _) in blocks_iter!(blocks) {
                let x = active_blocks.x + col as i32;
                let y = active_blocks.y + row as i32;

                min_x = std::cmp::min(x, min_x);

                if self.blocks.get(x as usize, y as usize).is_some() {
                    is_collide = true;
                    break;
                }
            }

            if min_x < 0 || is_collide {
                active_blocks.right();
            }
        }
    }

    pub fn move_active_blocks_right(&mut self) {
        if let Some(ref mut active_blocks) = self.active_blocks {
            active_blocks.right();

            let blocks = self.blocks_template.blocks(&active_blocks.template);
            let mut is_collide = false;
            let mut max_x = -1;
            for (col, row, _) in blocks_iter!(blocks) {
                let x = active_blocks.x + col as i32;
                let y = active_blocks.y + row as i32;

                max_x = std::cmp::max(x, max_x);

                if self.blocks.get(x as usize, y as usize).is_some() {
                    is_collide = true;
                    break;
                }
            }

            if max_x >= self.blocks.width as i32 || is_collide {
                active_blocks.left();
            }
        }
    }

    fn put_active_blocks(&mut self) {
        if let Some(ref mut active_blocks) = self.active_blocks {
            let blocks = self.blocks_template.blocks(&active_blocks.template);
            self.blocks.set_with_blocks(active_blocks.x, active_blocks.y, blocks);
        }

        self.active_blocks = None;

        let mut rows_need_remove = vec![];
        for row in 0..self.blocks.height {
            let mut row_is_empty = true;
            let mut row_need_remove = true;

            for col in 0..self.blocks.width {
                row_is_empty = false;
                let block = self.blocks.get(col, row);
                if block.is_none() {
                    row_need_remove = false;
                    break;
                }
            }

            if !row_is_empty && row_need_remove {
                rows_need_remove.push(row);
            }
        }

        rows_need_remove.reverse();
        for row in rows_need_remove {
            for row in row + 1..self.blocks.height {
                for col in 0..self.blocks.width {
                    let block = self.blocks.get(col, row).map(|b| b.clone());
                    self.blocks.set(col, row - 1, block);
                }
            }
        }
    }

    pub fn move_active_blocks_down(&mut self) {
        self.time_remain = self.interval;

        let mut is_collide = false;

        if let Some(ref mut active_blocks) = self.active_blocks {
            active_blocks.down();

            let blocks = self.blocks_template.blocks(&active_blocks.template);
            for (col, row, _) in blocks_iter!(blocks) {
                let x = active_blocks.x + col as i32;
                let y = active_blocks.y + row as i32;
                if y == -1 || self.blocks.get(x as usize, y as usize).is_some() {
                    is_collide = true;
                    break;
                }
            }

            if is_collide {
                active_blocks.up();
            }
        }

        if is_collide {
            self.put_active_blocks();
            self.set_active_blocks();
        }
    }

    pub fn transform_active_blocks(&mut self) {
        if let Some(ref mut active_blocks) = self.active_blocks {
            active_blocks.transform();
        }
    }

    pub fn update(&mut self, renderer: &mut SoftwareRenderer, dt: f32, x: i32, y: i32) {
        self.time_remain -= dt;
        if self.time_remain < 0.0 {
            self.move_active_blocks_down();
        }

        let block_size_in_pixels = 32i32;

        let color = RGBA {
            r: 0.6,
            g: 0.6,
            b: 0.6,
            a: 1.0,
        };

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
                    retris.playfield.set_active_blocks();
                }
                Event::KeyDown {keycode: Some(Keycode::Left), ..} => {
                    retris.playfield.move_active_blocks_left();
                }
                Event::KeyDown {keycode: Some(Keycode::Right), ..} => {
                    retris.playfield.move_active_blocks_right();
                }
                Event::KeyDown {keycode: Some(Keycode::Up), ..} => {
                    retris.playfield.transform_active_blocks();
                }
                Event::KeyDown {keycode: Some(Keycode::Down), ..} => {
                    retris.playfield.move_active_blocks_down();
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
