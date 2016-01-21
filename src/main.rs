#![feature(custom_attribute, stmt_expr_attributes)]

extern crate rand;
extern crate sdl2;
extern crate time;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use time::PreciseTime;

use renderer::*;

mod renderer;

#[derive(Copy, Clone)]
pub struct Cell {
    color: RGBA,
}

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
        if x < self.width && y < self.height {
            self.data[y * self.width + x] = cell;
        }
    }

    pub fn set_with_cell(&mut self, x: usize, y: usize, cell: Cell) {
        self.set(x, y, Some(cell));
    }

    pub fn set_with_block(&mut self, x: i32, y: i32, block: &Block) {
        for (col, row, cell) in block_iter!(block) {
            self.set_with_cell((x + col as i32) as usize, (y + row as i32) as usize, cell);
        }
    }

    pub fn is_valid_position(&self, x: i32, y: i32, block: &Block) -> bool {
        for (col, row, _) in block_iter!(block) {
            let x = x + col as i32;
            let y = y + row as i32;

            if x < 0 || x >= self.width as i32 || y < 0 ||
               self.get(x as usize, y as usize).is_some() {
                return false;
            }
        }

        true
    }

    pub fn lock_block(&mut self, x: i32, y: i32, block: &Block) {
        self.set_with_block(x, y, block);

        let mut lines = vec![];
        for row in 0..self.height {
            let mut is_empty_line = true;
            let mut is_need_break = true;

            for col in 0..self.width {
                is_empty_line = false;
                let block = self.get(col, row);
                if block.is_none() {
                    is_need_break = false;
                    break;
                }
            }

            if !is_empty_line && is_need_break {
                lines.push(row);
            }
        }

        lines.reverse();
        for row in lines {
            for row in row + 1..self.height {
                for col in 0..self.width {
                    let cell = self.get(col, row).map(|b| b.clone());
                    self.set(col, row - 1, cell);
                }
            }
        }
    }

    pub fn get_ghost_block_pos(&self, x: i32, mut y: i32, block: &Block) -> (i32, i32) {
        loop {
            if !self.is_valid_position(x, y, block) {
                return (x, y + 1)
            }
            y -= 1;
        }
    }
}

pub struct BlockTemplate {
    templates: [[Block; 4]; 7],
    wall_kick_table: [[[(i32, i32); 5]; 8]; 2],
}

impl BlockTemplate {
    pub fn new() -> BlockTemplate {
        let cyan = Cell {
            color: RGBA {r: 0.0, g: 240.0 / 255.0, b: 241.0 / 255.0, a: 1.0}
        };
        let yellow = Cell {
            color: RGBA {r: 240.0 / 255.0, g: 242.0 / 255.0, b: 0.0, a: 1.0}
        };
        let purple = Cell {
            color: RGBA {r: 161.0 / 255.0, g: 0.0, b: 244.0 / 255.0, a: 1.0}
        };
        let green = Cell {
            color: RGBA {r: 0.0, g: 242.0 / 255.0, b: 0.0, a: 1.0}
        };
        let red = Cell {
            color: RGBA {r: 243.0 / 255.0, g: 0.0, b: 0.0, a: 1.0}
        };
        let blue = Cell {
            color: RGBA {r: 0.0, g: 0.0, b: 244.0 / 255.0, a: 1.0}
        };
        let orange = Cell {
            color: RGBA {r: 242.0 / 255.0, g: 161.0 / 255.0, b: 0.0, a: 1.0}
        };

        // NOTE(coeuvre): Bitmap data for blocks. The origin is left-bottom corner.
        //
        //   x x x x
        //   x x x x
        //   x x x x
        //   o x x x
        //
        //   Using SRS described at https://tetris.wiki/SRS.
        //
        #[rustfmt_skip]
        BlockTemplate {
            templates: [
                // Cyan I
                [
                    Block::from_data(4, 4, vec![
                        None, None, None, None,
                        None, None, None, None,
                        Some(cyan),  Some(cyan),  Some(cyan),  Some(cyan),
                        None, None, None, None,
                    ]),
                    Block::from_data(4, 4, vec![
                        None, None, Some(cyan), None,
                        None, None, Some(cyan), None,
                        None, None, Some(cyan), None,
                        None, None, Some(cyan), None,
                    ]),
                    Block::from_data(4, 4, vec![
                        None, None, None, None,
                        Some(cyan),  Some(cyan),  Some(cyan),  Some(cyan),
                        None, None, None, None,
                        None, None, None, None,
                    ]),
                    Block::from_data(4, 4, vec![
                        None, Some(cyan), None, None,
                        None, Some(cyan), None, None,
                        None, Some(cyan), None, None,
                        None, Some(cyan), None, None,
                    ]),
                ],

                // Yellow O
                [
                    Block::from_data(4, 3, vec![
                        None, None, None, None,
                        None, Some(yellow), Some(yellow), None,
                        None, Some(yellow), Some(yellow), None,
                    ]),
                    Block::from_data(4, 3, vec![
                        None, None, None, None,
                        None, Some(yellow), Some(yellow), None,
                        None, Some(yellow), Some(yellow), None,
                    ]),
                    Block::from_data(4, 3, vec![
                        None, None, None, None,
                        None, Some(yellow), Some(yellow), None,
                        None, Some(yellow), Some(yellow), None,
                    ]),
                    Block::from_data(4, 3, vec![
                        None, None, None, None,
                        None, Some(yellow), Some(yellow), None,
                        None, Some(yellow), Some(yellow), None,
                    ]),
                ],

                // Purple T
                [
                    Block::from_data(3, 3, vec![
                        None, None, None,
                        Some(purple), Some(purple), Some(purple),
                        None, Some(purple), None,
                    ]),
                    Block::from_data(3, 3, vec![
                        None, Some(purple), None,
                        None, Some(purple), Some(purple),
                        None, Some(purple), None,
                    ]),
                    Block::from_data(3, 3, vec![
                        None, Some(purple), None,
                        Some(purple), Some(purple), Some(purple),
                        None, None, None,
                    ]),
                    Block::from_data(3, 3, vec![
                        None, Some(purple), None,
                        Some(purple), Some(purple), None,
                        None, Some(purple), None,
                    ]),
                ],

                // Green S
                [
                    Block::from_data(3, 3, vec![
                        None, None, None,
                        Some(green), Some(green), None,
                        None, Some(green), Some(green),
                    ]),
                    Block::from_data(3, 3, vec![
                        None, None, Some(green),
                        None, Some(green), Some(green),
                        None, Some(green), None,
                    ]),
                    Block::from_data(3, 3, vec![
                        Some(green), Some(green), None,
                        None, Some(green), Some(green),
                        None, None, None,
                    ]),
                    Block::from_data(3, 3, vec![
                        None, Some(green), None,
                        Some(green), Some(green), None,
                        Some(green), None, None,
                    ]),
                ],

                // Red Z
                [
                    Block::from_data(3, 3, vec![
                        None, None, None,
                        None, Some(red), Some(red),
                        Some(red), Some(red), None,
                    ]),
                    Block::from_data(3, 3, vec![
                        None, Some(red), None,
                        None, Some(red), Some(red),
                        None, None, Some(red),
                    ]),
                    Block::from_data(3, 3, vec![
                        None, Some(red), Some(red),
                        Some(red), Some(red), None,
                        None, None, None,
                    ]),
                    Block::from_data(3, 3, vec![
                        Some(red), None, None,
                        Some(red), Some(red), None,
                        None, Some(red), None,
                    ]),
                ],

                // Blue J
                [
                    Block::from_data(3, 3, vec![
                        None, None, None,
                        Some(blue), Some(blue), Some(blue),
                        Some(blue), None, None,
                    ]),
                    Block::from_data(3, 3, vec![
                        None, Some(blue), None,
                        None, Some(blue), None,
                        None, Some(blue), Some(blue),
                    ]),
                    Block::from_data(3, 3, vec![
                        None, None, Some(blue),
                        Some(blue), Some(blue), Some(blue),
                        None, None, None,
                    ]),
                    Block::from_data(3, 3, vec![
                        Some(blue), Some(blue), None,
                        None, Some(blue), None,
                        None, Some(blue), None,
                    ]),
                ],

                // Orange L
                [
                    Block::from_data(3, 3, vec![
                        None, None, None,
                        Some(orange), Some(orange), Some(orange),
                        None, None, Some(orange),
                    ]),
                    Block::from_data(3, 3, vec![
                        None, Some(orange), Some(orange),
                        None, Some(orange), None,
                        None, Some(orange), None,
                    ]),
                    Block::from_data(3, 3, vec![
                        Some(orange), None, None,
                        Some(orange), Some(orange), Some(orange),
                        None, None, None,
                    ]),
                    Block::from_data(3, 3, vec![
                        None, Some(orange), None,
                        None, Some(orange), None,
                        Some(orange), Some(orange), None,
                    ]),
                ],
            ],

            wall_kick_table: [
                // J, L, S, T, Z wall kick data
                [
                    // 0 -> 1
                    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
                    // 1 -> 0
                    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
                    // 1 -> 2
                    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
                    // 2 -> 1
                    [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
                    // 2 -> 3
                    [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
                    // 3 -> 2
                    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
                    // 3 -> 0
                    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
                    // 0 -> 3
                    [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
                ],

                // I wall kick data
                [
                    // 0 -> 1
                    [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
                    // 1 -> 0
                    [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
                    // 1 -> 2
                    [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
                    // 2 -> 1
                    [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
                    // 2 -> 3
                    [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
                    // 3 -> 2
                    [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
                    // 3 -> 0
                    [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
                    // 0 -> 3
                    [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
                ],
            ],
        }
    }

    pub fn generate(&self) -> BlockTemplateRef {
        let shape = rand::random::<usize>() % self.templates.len();
        let order = 0;
        let order_max = self.templates[shape].len();

        BlockTemplateRef {
            shape: shape,
            order: order,
            order_max: order_max,
        }
    }

    pub fn block(&self, template: &BlockTemplateRef) -> &Block {
        &self.templates[template.shape][template.order]
    }

    pub fn wall_kick_table(&self, template: &BlockTemplateRef, new_template: &BlockTemplateRef) -> &[(i32, i32); 5] {
        assert!(template.order != new_template.order);

        let index = match template.order {
            0 => match new_template.order {
                1 => 0,
                3 => 7,
                _ => unreachable!(),
            },
            1 => match new_template.order {
                2 => 2,
                0 => 1,
                _ => unreachable!(),
            },
            2 => match new_template.order {
                3 => 4,
                1 => 3,
                _ => unreachable!(),
            },
            3 => match new_template.order {
                0 => 6,
                2 => 5,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };

        if template.shape == 0 {
            &self.wall_kick_table[1][index]
        } else {
            &self.wall_kick_table[0][index]
        }
    }
}

#[derive(Copy, Clone)]
pub struct BlockTemplateRef {
    shape: usize,
    order: usize,
    order_max: usize,
}

impl BlockTemplateRef {
    pub fn rrotate(&mut self) {
        self.order = (self.order + 1) % self.order_max;
    }

    pub fn lrotate(&mut self) {
        if self.order == 0 {
            self.order = self.order_max - 1;
        } else {
            self.order -= 1;
        }
    }
}

#[derive(Clone)]
pub struct FallingBlock {
    template: BlockTemplateRef,
    x: i32,
    y: i32,
}

impl FallingBlock {
    pub fn new(template: BlockTemplateRef) -> FallingBlock {
        FallingBlock {
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
}

pub enum Movement {
    Left,
    Right,
    Down,
    RRotate,
    LRotate,
    Drop,
    Hold,
}

pub struct Playfield {
    block: Block,

    block_template: BlockTemplate,

    falling_block: FallingBlock,
    next_templates: Vec<BlockTemplateRef>,

    held_template: Option<BlockTemplateRef>,
    can_hold_falling_block: bool,

    interval: f32,
    time_remain: f32,
}

impl Playfield {
    pub fn new(width: usize, height: usize) -> Playfield {
        let block_template = BlockTemplate::new();
        let falling_block = Playfield::generate_falling_block(&block_template);
        let next_templates = vec![
            block_template.generate(),
            block_template.generate(),
            block_template.generate(),
        ];

        Playfield {
            block: Block::new(width, height),

            block_template: block_template,

            falling_block: falling_block,
            next_templates: next_templates,

            held_template: None,
            can_hold_falling_block: true,

            interval: 1.0,
            time_remain: 0.0,
        }
    }

    pub fn set_falling_block(&mut self) {
        let template = self.next_templates.remove(0);
        self.next_templates.push(self.block_template.generate());
        self.set_falling_block_with_template(template);
    }

    fn set_falling_block_with_template(&mut self, template: BlockTemplateRef) {
        self.falling_block = Playfield::generate_falling_block_with_template(template);
        self.time_remain = self.interval;
    }

    fn generate_falling_block(block_template: &BlockTemplate) -> FallingBlock {
        let template = block_template.generate();
        Playfield::generate_falling_block_with_template(template)
    }

    fn generate_falling_block_with_template(template: BlockTemplateRef) -> FallingBlock{
        let mut falling_block = FallingBlock::new(template);
        falling_block.move_to(3, 19);
        falling_block
    }

    fn lock_falling_block(&mut self) {
        let x = self.falling_block.x;
        let y =self.falling_block.y;
        self.lock_falling_block_at(x, y);
    }

    fn lock_falling_block_at(&mut self, x: i32, y: i32) {
        self.block.lock_block(
            x, y,
            self.block_template.block(&self.falling_block.template)
        );
        self.can_hold_falling_block = true;
        self.set_falling_block();
    }

    pub fn move_falling_block(&mut self, movement: Movement) {
        let x = self.falling_block.x;
        let y = self.falling_block.y;
        let template = self.falling_block.template;

        match movement {
            Movement::Left => {
                let block = self.block_template.block(&template);
                if self.block.is_valid_position(x - 1, y, block) {
                    self.falling_block.left();
                }
            }
            Movement::Right => {
                let block = self.block_template.block(&template);
                if self.block.is_valid_position(x + 1, y, block) {
                    self.falling_block.right();
                }
            }
            Movement::Down => {
                self.time_remain = self.interval;
                if self.block.is_valid_position(x, y - 1, self.block_template.block(&template)) {
                    self.falling_block.down();
                } else {
                    self.lock_falling_block();
                }
            }
            Movement::Drop => {
                let (x, y) = self.block.get_ghost_block_pos(x, y, self.block_template.block(&template));
                self.lock_falling_block_at(x, y);
            }
            Movement::RRotate | Movement::LRotate => {
                let mut new_template = template;
                match movement {
                    Movement::RRotate => new_template.rrotate(),
                    Movement::LRotate => new_template.lrotate(),
                    _ => unreachable!(),
                };
                let block = self.block_template.block(&new_template);
                let table = self.block_template.wall_kick_table(&template, &new_template);
                for &(dx, dy) in table {
                    let x = x + dx;
                    let y = y + dy;
                    if self.block.is_valid_position(x, y, block) {
                        self.falling_block.x = x;
                        self.falling_block.y = y;
                        self.falling_block.template = new_template;
                        break;
                    }
                }
            }
            Movement::Hold => {
                if self.can_hold_falling_block {
                    let mut held_template = template;
                    held_template.order = 0;
                    if self.held_template.is_none() {
                        self.set_falling_block();
                    } else {
                        let new_template = self.held_template.as_ref().unwrap().clone();
                        self.set_falling_block_with_template(new_template);
                    }
                    self.held_template = Some(held_template);
                    self.can_hold_falling_block = false;
                }
            }
        }
    }

    pub fn update(&mut self, renderer: &mut SoftwareRenderer, dt: f32, x: i32, y: i32) {
        self.time_remain -= dt;
        if self.time_remain < 0.0 {
            self.move_falling_block(Movement::Down);
        }

        let block_size_in_pixels = 32i32;
        let width = self.block.width as i32 * block_size_in_pixels;
        let height = (self.block.height - 2) as i32 * block_size_in_pixels;

        // Held template
        if let Some(ref mut held_template) = self.held_template {
            let block = self.block_template.block(&held_template);

            for (col, row, cell) in block_iter!(block) {
                let x_offset = col as i32 * block_size_in_pixels;
                let y_offset = (block.height - row) as i32 * block_size_in_pixels;
                let x = x + x_offset;
                let y = y + (height - y_offset);
                renderer.fill_rect(x + 1,
                                   y + 1,
                                   x + block_size_in_pixels,
                                   y + block_size_in_pixels,
                                   cell.color);
            }
        }


        let x = x + 5 * block_size_in_pixels;

        // Falling block
        let block = self.block_template.block(&self.falling_block.template);
        let (ghost_x, ghost_y) = self.block.get_ghost_block_pos(self.falling_block.x, self.falling_block.y, block);

        for (col, row, cell) in block_iter!(block) {
            // Ghost
            {
                let x_offset = (ghost_x + col as i32) * block_size_in_pixels;
                let y_offset = (ghost_y + row as i32) * block_size_in_pixels;
                let x = x + x_offset;
                let y = y + y_offset;
                renderer.rect(x + 1,
                              y + 1,
                              x + block_size_in_pixels - 1,
                              y + block_size_in_pixels - 1,
                              cell.color);
            }

            // Simply clip the block
            if self.falling_block.y + (row as i32) < self.block.height as i32 - 2 {
                let x_offset = (self.falling_block.x + col as i32) * block_size_in_pixels;
                let y_offset = (self.falling_block.y + row as i32) * block_size_in_pixels;
                let x = x + x_offset;
                let y = y + y_offset;
                renderer.fill_rect(x + 1,
                                   y + 1,
                                   x + block_size_in_pixels,
                                   y + block_size_in_pixels,
                                   cell.color);
            }
        }

        // Fixed cells
        for (col, row, cell) in block_iter!(self.block) {
            let x_offset = (col as i32) * block_size_in_pixels;
            let y_offset = (row as i32) * block_size_in_pixels;
            let x = x + x_offset;
            let y = y + y_offset;
            renderer.fill_rect(x + 1,
                               y + 1,
                               x + block_size_in_pixels,
                               y + block_size_in_pixels,
                               cell.color);
        }

        let color = RGBA {
            r: 0.6,
            g: 0.6,
            b: 0.6,
            a: 1.0,
        };

        // Grids

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

        // Next templates
        let x = x + width + block_size_in_pixels;
        for (i, template) in self.next_templates.iter().enumerate() {
            let block = self.block_template.block(template);

            for (col, row, cell) in block_iter!(block) {
                let x_offset = col as i32 * block_size_in_pixels;
                let y_offset = (block.height - row) as i32 * block_size_in_pixels;
                let x = x + x_offset;
                let y = y + (height - y_offset) - i as i32 * 4 * block_size_in_pixels;
                renderer.fill_rect(x + 1,
                                   y + 1,
                                   x + block_size_in_pixels,
                                   y + block_size_in_pixels,
                                   cell.color);
            }
        }
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
        self.playfield.update(renderer, dt, 32, 32);
    }
}

fn main() {
    let sdl2 = sdl2::init().unwrap();
    let video = sdl2.video().unwrap();

    let width = 800;
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
    retris.playfield.set_falling_block();

    let mut frame_last = PreciseTime::now();

    'running: loop {
        let frame_start = PreciseTime::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} => break 'running,
                Event::KeyDown {keycode: Some(Keycode::Z), ..} => {
                    retris.playfield.move_falling_block(Movement::LRotate);
                }
                Event::KeyDown {keycode: Some(Keycode::C), ..} => {
                    retris.playfield.move_falling_block(Movement::Hold);
                }
                Event::KeyDown {keycode: Some(Keycode::Space), ..} => {
                    retris.playfield.move_falling_block(Movement::Drop);
                }
                Event::KeyDown {keycode: Some(Keycode::Left), ..} => {
                    retris.playfield.move_falling_block(Movement::Left);
                }
                Event::KeyDown {keycode: Some(Keycode::Right), ..} => {
                    retris.playfield.move_falling_block(Movement::Right);
                }
                Event::KeyDown {keycode: Some(Keycode::Up), ..} => {
                    retris.playfield.move_falling_block(Movement::RRotate);
                }
                Event::KeyDown {keycode: Some(Keycode::Down), ..} => {
                    retris.playfield.move_falling_block(Movement::Down);
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
