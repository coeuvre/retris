#![feature(custom_attribute, stmt_expr_attributes)]

extern crate rand;
extern crate sdl2;
extern crate time;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use time::PreciseTime;

use renderer::*;

mod renderer;

// TODO(coeuvre): Placeholder type for Cell.
pub type Cell = ();

macro_rules! block_iter {
    ($block:expr) => {
        $block.data
            .iter()
            .enumerate()
            .filter(|&(_, cell)| cell.is_some())
            .map(|(i, cell)| {
                (i % $block.width, i / $block.width, cell.unwrap())
            })
    }
}

pub struct Block {
    width: usize,
    height: usize,
    data: Vec<Option<Cell>>,
}

impl Block {
    pub fn new(width: usize, height: usize) -> Block {
        Block {
            width: width,
            height: height,
            data: vec![None; width * height],
        }
    }

    pub fn from_data(width: usize, height: usize, data: Vec<Option<Cell>>) -> Block {
        assert!(data.len() == width * height);
        Block {
            width: width,
            height: height,
            data: data,
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&Cell> {
        if x >= self.width || y >= self.height {
            return None;
        }

        self.data[y * self.width + x].as_ref()
    }

    pub fn set(&mut self, x: usize, y: usize, cell: Option<Cell>) {
        self.data[y * self.width + x] = cell;
    }

    pub fn set_with_cell(&mut self, x: usize, y: usize, cell: Cell) {
        self.set(x, y, Some(cell));
    }

    pub fn set_with_block(&mut self, x: i32, y: i32, block: &Block) {
        for (col, row, cell) in block_iter!(block) {
            self.set_with_cell((x + col as i32) as usize, (y + row as i32) as usize, cell);
        }
    }
}

pub struct BlockTemplate {
    templates: Vec<Vec<Block>>,
}

impl BlockTemplate {
    pub fn new() -> BlockTemplate {
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
        BlockTemplate {
            templates: vec![
                // Block I
                vec![
                    Block::from_data(width, height, vec![
                        None, Some(()), None, None,
                        None, Some(()), None, None,
                        None, Some(()), None, None,
                        None, Some(()), None, None,
                    ]),
                    Block::from_data(width, height, vec![
                        Some(()),  Some(()),  Some(()),  Some(()),
                        None, None, None, None,
                        None, None, None, None,
                        None, None, None, None,
                    ]),
                ],

                // Block O
                vec![
                    Block::from_data(width, height, vec![
                        None, Some(()), Some(()), None,
                        None, Some(()), Some(()), None,
                        None, None, None, None,
                        None, None, None, None,
                    ]),
                ],
            ],
        }
    }

    pub fn generate(&self) -> BlockTemplateRef {
        let shape = rand::random::<usize>() % self.templates.len();
        let order_max = self.templates[shape].len();
        let order = rand::random::<usize>() % order_max;

        BlockTemplateRef {
            shape: shape,
            order: order,
            order_max: order_max,
        }
    }

    pub fn block(&self, r: &BlockTemplateRef) -> &Block {
        &self.templates[r.shape][r.order]
    }
}

pub struct BlockTemplateRef {
    shape: usize,
    order: usize,
    order_max: usize,
}

pub struct ActiveBlock {
    template: BlockTemplateRef,
    x: i32,
    y: i32,
}

impl ActiveBlock {
    pub fn new(template: BlockTemplateRef) -> ActiveBlock {
        ActiveBlock {
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
    block: Block,

    block_template: BlockTemplate,

    active_block: Option<ActiveBlock>,

    interval: f32,
    time_remain: f32,
}

impl Playfield {
    pub fn new(width: usize, height: usize) -> Playfield {
        Playfield {
            block: Block::new(width, height),

            block_template: BlockTemplate::new(),
            active_block: None,

            interval: 1.0,
            time_remain: 0.0,
        }
    }

    pub fn set_active_block(&mut self) {
        if self.active_block.is_some() {
            return;
        }

        let r = self.block_template.generate();
        let mut active_block = ActiveBlock::new(r);
        active_block.move_to(3, 19);
        self.active_block = Some(active_block);
        self.time_remain = self.interval;
    }

    pub fn move_active_block_left(&mut self) {
        if let Some(ref mut active_block) = self.active_block {
            active_block.left();

            let block = self.block_template.block(&active_block.template);
            let mut is_collide = false;
            let mut min_x = self.block.width as i32;
            for (col, row, _) in block_iter!(block) {
                let x = active_block.x + col as i32;
                let y = active_block.y + row as i32;

                min_x = std::cmp::min(x, min_x);

                if self.block.get(x as usize, y as usize).is_some() {
                    is_collide = true;
                    break;
                }
            }

            if min_x < 0 || is_collide {
                active_block.right();
            }
        }
    }

    pub fn move_active_block_right(&mut self) {
        if let Some(ref mut active_block) = self.active_block {
            active_block.right();

            let block = self.block_template.block(&active_block.template);
            let mut is_collide = false;
            let mut max_x = -1;
            for (col, row, _) in block_iter!(block) {
                let x = active_block.x + col as i32;
                let y = active_block.y + row as i32;

                max_x = std::cmp::max(x, max_x);

                if self.block.get(x as usize, y as usize).is_some() {
                    is_collide = true;
                    break;
                }
            }

            if max_x >= self.block.width as i32 || is_collide {
                active_block.left();
            }
        }
    }

    fn put_active_block(&mut self) {
        if let Some(ref mut active_block) = self.active_block {
            let block = self.block_template.block(&active_block.template);
            self.block.set_with_block(active_block.x, active_block.y, block);
        }

        self.active_block = None;

        let mut rows_need_remove = vec![];
        for row in 0..self.block.height {
            let mut row_is_empty = true;
            let mut row_need_remove = true;

            for col in 0..self.block.width {
                row_is_empty = false;
                let block = self.block.get(col, row);
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
            for row in row + 1..self.block.height {
                for col in 0..self.block.width {
                    let block = self.block.get(col, row).map(|b| b.clone());
                    self.block.set(col, row - 1, block);
                }
            }
        }
    }

    pub fn move_active_block_down(&mut self) {
        self.time_remain = self.interval;

        let mut is_collide = false;

        if let Some(ref mut active_block) = self.active_block {
            active_block.down();

            let block = self.block_template.block(&active_block.template);
            for (col, row, _) in block_iter!(block) {
                let x = active_block.x + col as i32;
                let y = active_block.y + row as i32;
                if y == -1 || self.block.get(x as usize, y as usize).is_some() {
                    is_collide = true;
                    break;
                }
            }

            if is_collide {
                active_block.up();
            }
        }

        if is_collide {
            self.put_active_block();
            self.set_active_block();
        }
    }

    pub fn transform_active_block(&mut self) {
        if let Some(ref mut active_block) = self.active_block {
            active_block.transform();
        }
    }

    pub fn update(&mut self, renderer: &mut SoftwareRenderer, dt: f32, x: i32, y: i32) {
        self.time_remain -= dt;
        if self.time_remain < 0.0 {
            self.move_active_block_down();
        }

        let block_size_in_pixels = 32i32;

        let color = RGBA {
            r: 0.6,
            g: 0.6,
            b: 0.6,
            a: 1.0,
        };

        // Current block
        if let Some(ref mut active_block) = self.active_block {
            let block = self.block_template.block(&active_block.template);

            for (col, row, _) in block_iter!(block) {
                // Simply clip the block
                if active_block.y + (row as i32) < self.block.height as i32 - 2 {
                    let x_offset = (active_block.x + col as i32) * block_size_in_pixels;
                    let y_offset = (active_block.y + row as i32) * block_size_in_pixels;
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

        // Fixed cells
        for (col, row, _) in block_iter!(self.block) {
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
        let width = self.block.width as i32 * block_size_in_pixels;
        let height = (self.block.height - 2) as i32 * block_size_in_pixels;

        for row in 1..self.block.height - 2 {
            let y_offset = row as i32 * block_size_in_pixels;
            renderer.hline(y + y_offset, x, x + width, color);
        }

        for col in 1..self.block.width {
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
                    retris.playfield.set_active_block();
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
                Event::KeyDown {keycode: Some(Keycode::Down), ..} => {
                    retris.playfield.move_active_block_down();
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
