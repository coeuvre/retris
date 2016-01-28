extern crate rand;
extern crate sdl2;
extern crate time;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use time::PreciseTime;

use bitmap::*;
use renderer::*;
use state::*;

pub mod bitmap;
pub mod renderer;
pub mod state;

const CYAN: i32 = 6;
const YELLOW: i32 = 0;
const PURPLE: i32 = 3;
const GREEN: i32 = 4;
const RED: i32 = 2;
const BLUE: i32 = 5;
const ORANGE: i32 = 1;

pub struct Retris {
    blocks: Bitmap,
    playfield: Playfield,
}

impl Retris {
    pub fn new() -> Retris {
        let blocks = Bitmap::open("./assets/blocks.bmp").unwrap();
        Retris {
            playfield: Playfield::new(10, 20, blocks.height() as i32),
            blocks: blocks,
        }
    }
}

pub struct Context {
    pub dt: f32,
    pub renderer: SoftwareRenderer,
    pub events: Vec<Event>,
}

pub struct Prepare {
    countdown: Timer,
}

impl Prepare {
    pub fn new() -> Prepare {
        Prepare {
            countdown: Timer::new(0.0),
        }
    }
}

impl State for Prepare {
    type Context = Context;
    type Game = Retris;

    fn update(&mut self, ctx: &mut Context, game: &mut Retris) -> Trans<Context, Retris> {
        self.countdown.tick(ctx.dt);

        println!("coutndown: {:.2}", self.countdown.elapsed());

        if self.countdown.is_expired() {
            let template = game.playfield.block_template.generate();
            let bottom = game.playfield.block_template.block(&template).bottom();
            game.playfield.falling_block = FallingBlock::new(3, 20 - bottom as i32, template);

            return Trans::switch(Falling::new());
        }

        Trans::none()
    }
}

fn handle_move_event(event: &Event, playfield: &mut Playfield) {
    match event {
        &Event::KeyDown {keycode: Some(Keycode::Up), ..} |
        &Event::KeyDown {keycode: Some(Keycode::Z), ..} => {
            let mut new_template = playfield.falling_block.template;
            if let &Event::KeyDown {keycode: Some(Keycode::Up), ..} = event {
                new_template.rrotate();
            } else {
                new_template.lrotate();
            }
            let block = playfield.block_template.block(&new_template);
            let table = playfield.block_template.wall_kick_table(&playfield.falling_block.template,
                                                                 &new_template);
            for &(dx, dy) in table {
                let x = playfield.falling_block.x + dx;
                let y = playfield.falling_block.y + dy;
                if playfield.block.is_valid_position(x, y, block) {
                    playfield.falling_block.x = x;
                    playfield.falling_block.y = y;
                    playfield.falling_block.template = new_template;
                    break;
                }
            }
        }
        &Event::KeyDown {keycode: Some(Keycode::Left), ..} => {
            let block = playfield.block_template.block(&playfield.falling_block.template);
            if playfield.block.is_valid_position(playfield.falling_block.x - 1,
                                                 playfield.falling_block.y,
                                                 block) {
                playfield.falling_block.left();
            }
        }
        &Event::KeyDown {keycode: Some(Keycode::Right), ..} => {
            let block = playfield.block_template.block(&playfield.falling_block.template);
            if playfield.block.is_valid_position(playfield.falling_block.x + 1,
                                                 playfield.falling_block.y,
                                                 block) {
                playfield.falling_block.right();
            }
        }
        _ => {}
    }
}

pub struct Falling {
    gravity_delay: Timer,
}

impl Falling {
    pub fn new() -> Falling {
        Falling {
            gravity_delay: Timer::new(gravity_to_delay(0.015625)),
        }
    }
}

impl State for Falling {
    type Context = Context;
    type Game = Retris;

    fn update(&mut self, ctx: &mut Context, game: &mut Retris) -> Trans<Context, Retris> {
        let mut trans = Trans::none();
        let dt = ctx.dt;
        let ref mut renderer = ctx.renderer;
        let ref mut playfield = game.playfield;

        assert!(playfield.block.is_valid_position(playfield.falling_block.x,
                                                  playfield.falling_block.y - 1,
                                                  &playfield.block_template.block(&playfield.falling_block.template)));

        for event in &ctx.events {
            handle_move_event(event, playfield);
            match event {
                &Event::KeyDown {keycode: Some(Keycode::Down), ..} => {
                    // NOTE(coeuvre): No need to check whether position (x, y - 1)
                    // is valid here because we check it every frame.
                    playfield.falling_block.down();
                    self.gravity_delay.reset();
                }
                &Event::KeyDown {keycode: Some(Keycode::Space), ..} => {
                    let (x, y) = playfield.block
                                          .get_ghost_block_pos(playfield.falling_block.x,
                                                               playfield.falling_block.y,
                                                               playfield.block_template.block(&playfield.falling_block.template));
                    playfield.falling_block.move_to(x, y);
                    // Partial lock out
                    if playfield.block.is_out_of_bounds(x, y, playfield.block_template.block(&playfield.falling_block.template)) {
                        trans = Trans::switch(Lost);
                    } else {
                        // Lock the block
                        playfield.block.set_with_block(x, y, playfield.block_template.block(&playfield.falling_block.template));

                        let lines = playfield.block.get_break_lines();
                        if lines.len() > 0 {
                            trans = Trans::switch(Breaking::new(lines));
                        } else {
                            playfield.spawn_falling_block();
                        }
                    }
                }
                _ => {}
            }
        }

        if trans.is_none() {
            self.gravity_delay.tick(dt);
            if self.gravity_delay.is_expired() {
                playfield.falling_block.down();
                self.gravity_delay.reset();
            }

            if !playfield.block.is_valid_position(playfield.falling_block.x,
                                                  playfield.falling_block.y - 1,
                                                  &playfield.block_template.block(&playfield.falling_block.template)) {
                trans = Trans::switch(Locking::new());
            }

            playfield.render_falling_block(renderer, 32, 32, &game.blocks);
        }

        playfield.render(renderer, 32, 32, &game.blocks);

        trans
    }
}

pub struct Locking {
    lock_delay: Timer,
}

impl Locking {
    pub fn new() -> Locking {
        Locking {
            lock_delay: Timer::new(frames_to_seconds(30.0)),
        }
    }
}

impl State for Locking {
    type Context = Context;
    type Game = Retris;

    fn update(&mut self, ctx: &mut Context, game: &mut Retris) -> Trans<Context, Retris> {
        let mut trans = Trans::none();
        let dt = ctx.dt;
        let ref mut renderer = ctx.renderer;
        let ref mut playfield = game.playfield;

        for event in &ctx.events {
            handle_move_event(event, playfield);
        }

        if playfield.block.is_valid_position(playfield.falling_block.x,
                                             playfield.falling_block.y - 1,
                                             &playfield.block_template.block(&playfield.falling_block.template)) {
            trans = Trans::switch(Falling::new());
        }

        self.lock_delay.tick(dt);
        playfield.max_lock_delay.tick(dt);

        if self.lock_delay.is_expired() || playfield.max_lock_delay.is_expired() {
            let x = playfield.falling_block.x;
            let y = playfield.falling_block.y;

            // Partial lock out
            if playfield.block.is_out_of_bounds(x, y, playfield.block_template.block(&playfield.falling_block.template)) {
                trans = Trans::switch(Lost);
            } else {
                // Lock the block
                playfield.block.set_with_block(x, y, playfield.block_template.block(&playfield.falling_block.template));

                let lines = playfield.block.get_break_lines();
                if lines.len() > 0 {
                    trans = Trans::switch(Breaking::new(lines));
                } else {
                    playfield.spawn_falling_block();
                    trans = Trans::switch(Falling::new());
                }

            }
        }

        playfield.render_falling_block(renderer, 32, 32, &game.blocks);
        playfield.render(renderer, 32, 32, &game.blocks);
        trans
    }
}

pub struct Breaking {
    lines: Vec<usize>,
}

impl Breaking {
    pub fn new(lines: Vec<usize>) -> Breaking {
        Breaking {
            lines: lines,
        }
    }
}

impl State for Breaking {
    type Context = Context;
    type Game = Retris;

    fn update(&mut self, ctx: &mut Context, game: &mut Retris) -> Trans<Context, Retris> {
        let mut trans = Trans::none();
        let ref mut renderer = ctx.renderer;
        let ref mut playfield = game.playfield;

        playfield.render(renderer, 32, 32, &game.blocks);

        trans
    }
}

pub struct Lost;

impl State for Lost {
    type Context = Context;
    type Game = Retris;
}

#[derive(Copy, Clone)]
pub struct Cell {
    index: i32,
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

    pub fn bottom(&self) -> usize {
        for (_, row, _) in block_iter!(self) {
            return row;
        }

        unreachable!();
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

    pub fn is_out_of_bounds(&self, x: i32, y: i32, block: &Block) -> bool {
        for (col, row, _) in block_iter!(block) {
            let x = x + col as i32;
            let y = y + row as i32;

            if x < 0 || x >= self.width as i32 ||
               y < 0 || y >= self.height as i32 {
                return true;
            }
        }

        false
    }

    pub fn get_break_lines(&self) -> Vec<usize> {
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
        lines
    }

    pub fn break_lines(&mut self) {
        for row in self.get_break_lines() {
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
             color: RGBA {r: 0.0, g: 240.0 / 255.0, b: 241.0 / 255.0, a: 1.0},
             index: CYAN,
         };
         let yellow = Cell {
             color: RGBA {r: 240.0 / 255.0, g: 242.0 / 255.0, b: 0.0, a: 1.0},
             index: YELLOW,
         };
         let purple = Cell {
             color: RGBA {r: 161.0 / 255.0, g: 0.0, b: 244.0 / 255.0, a: 1.0},
             index: PURPLE,
         };
         let green = Cell {
             color: RGBA {r: 0.0, g: 242.0 / 255.0, b: 0.0, a: 1.0},
             index: GREEN,
         };
         let red = Cell {
             color: RGBA {r: 243.0 / 255.0, g: 0.0, b: 0.0, a: 1.0},
             index: RED,
         };
         let blue = Cell {
             color: RGBA {r: 0.0, g: 0.0, b: 244.0 / 255.0, a: 1.0},
             index: BLUE,
         };
         let orange = Cell {
             color: RGBA {r: 242.0 / 255.0, g: 161.0 / 255.0, b: 0.0, a: 1.0},
             index: ORANGE,
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
    pub fn none() -> BlockTemplateRef {
        BlockTemplateRef {
            shape: 0,
            order: 0,
            order_max: 0,
        }
    }

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
    pub fn none() -> FallingBlock {
        FallingBlock {
            template: BlockTemplateRef::none(),
            x: 0,
            y: 0,
        }
    }

    pub fn new(x: i32, y: i32, template: BlockTemplateRef) -> FallingBlock {
        FallingBlock {
            template: template,
            x: x,
            y: y,
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

pub struct StateStack<T> {
    stack: Vec<T>,
}

impl<T> StateStack<T> {
    pub fn new(init: T) -> StateStack<T> {
        StateStack {
            stack: vec![init],
        }
    }

    pub fn push(&mut self, state: T) {
        self.stack.push(state);
    }

    pub fn pop(&mut self) {
        assert!(self.stack.len() > 1);

        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    pub fn top(&self) -> &T {
        self.stack.last().unwrap()
    }

    pub fn set(&mut self, state: T) {
        *self.stack.last_mut().unwrap() = state;
    }
}

pub enum PlayfieldState {
    Prepare,
    Spwaning,
    Falling,
    Locking,
    Breaking,
    Paused,
    Lost,
}

pub struct Timer {
    interval: f32,
    elapsed: f32,
}

impl Timer {
    pub fn new(interval: f32) -> Timer {
        Timer {
            interval: interval,
            elapsed: 0.0,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.elapsed += dt;
    }

    pub fn is_expired(&self) -> bool {
        self.elapsed >= self.interval
    }

    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    pub fn percent(&self) -> f32 {
        self.elapsed / self.interval
    }

    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }
}

fn frames_to_seconds(frames: f32) -> f32 {
    // NOTE(coeuvre): Assuming the game run under 60 FPS.
    let fps = 60.0;
    frames / fps
}

fn gravity_to_delay(gravity: f32) -> f32 {
    // NOTE(coeuvre): gravity is how many cells per frame, so the inverse
    // is how many frames per cell.
    frames_to_seconds(1.0 / gravity)
}


pub struct Playfield {
    state: StateStack<PlayfieldState>,

    block: Block,

    block_template: BlockTemplate,

    falling_block: FallingBlock,
    next_templates: Vec<BlockTemplateRef>,

    held_template: Option<BlockTemplateRef>,
    can_hold_falling_block: bool,

    gravity_delay: Timer,
    lock_delay: Timer,
    max_lock_delay: Timer,

    breaking_line_delay: Timer,
    breaking_lines: Vec<usize>,
    blink_interval: Timer,
    is_blink: bool,

    block_size_in_pixels: i32,
}

impl Playfield {
    pub fn new(width: usize, height: usize, block_size_in_pixels: i32) -> Playfield {
        let block_template = BlockTemplate::new();
        let falling_block = Playfield::generate_falling_block(&block_template);
        let next_templates = vec![
            block_template.generate(),
            block_template.generate(),
            block_template.generate(),
        ];

        Playfield {
            state: StateStack::new(PlayfieldState::Falling),

            block: Block::new(width, height),

            block_template: block_template,

            falling_block: falling_block,
            next_templates: next_templates,

            held_template: None,
            can_hold_falling_block: true,

            gravity_delay: Timer::new(gravity_to_delay(0.015625)),
            lock_delay: Timer::new(frames_to_seconds(30.0)),
            max_lock_delay: Timer::new(frames_to_seconds(60.0)),

            breaking_line_delay: Timer::new(0.25),
            breaking_lines: vec![],
            blink_interval: Timer::new(0.05),
            is_blink: false,

            block_size_in_pixels: block_size_in_pixels,
        }
    }

    fn spawn_falling_block(&mut self) {
        let template = self.next_templates.remove(0);
        self.next_templates.push(self.block_template.generate());
        self.spawn_falling_block_with_template(template);
    }

    fn spawn_falling_block_with_template(&mut self, template: BlockTemplateRef) {
        self.falling_block = Playfield::generate_falling_block_with_template(&self.block_template, template);
        self.gravity_delay.reset();
        self.lock_delay.reset();
        self.max_lock_delay.reset();
        // Block out
        if !self.block.is_valid_position(self.falling_block.x,
                                         self.falling_block.y,
                                         self.block_template.block(&self.falling_block.template)) {
            self.state.set(PlayfieldState::Lost);
        }
    }

    fn generate_falling_block(block_template: &BlockTemplate) -> FallingBlock {
        let template = block_template.generate();
        Playfield::generate_falling_block_with_template(block_template, template)
    }

    fn generate_falling_block_with_template(block_template: &BlockTemplate,
                                            template: BlockTemplateRef) -> FallingBlock{
        let bottom = block_template.block(&template).bottom();
        FallingBlock::new(3, 20 - bottom as i32, template)
    }

    fn lock_falling_block(&mut self) {
        let x = self.falling_block.x;
        let y = self.falling_block.y;
        self.lock_falling_block_at(x, y);
    }

    fn lock_falling_block_at(&mut self, x: i32, y: i32) {
        {
            let block = self.block_template.block(&self.falling_block.template);
            // Partial lock out
            if self.block.is_out_of_bounds(x, y, block) {
                self.state.set(PlayfieldState::Lost);
                return;
            }
            // Lock the block
            self.block.set_with_block(x, y, block);
        }
        self.state.set(PlayfieldState::Breaking);
    }

    pub fn move_falling_block(&mut self, movement: Movement) {
        match self.state.top() {
            &PlayfieldState::Falling | &PlayfieldState::Locking => {
                let x = self.falling_block.x;
                let y = self.falling_block.y;
                let template = self.falling_block.template;

                match movement {
                    Movement::Left => {
                        let block = self.block_template.block(&template);
                        if self.block.is_valid_position(x - 1, y, block) {
                            self.falling_block.left();
                            self.lock_delay.reset();
                            if self.block.is_valid_position(x, y - 1, block) {
                                self.state.set(PlayfieldState::Falling);
                            }
                        }
                    }
                    Movement::Right => {
                        let block = self.block_template.block(&template);
                        if self.block.is_valid_position(x + 1, y, block) {
                            self.falling_block.right();
                            self.lock_delay.reset();
                            if self.block.is_valid_position(x, y - 1, block) {
                                self.state.set(PlayfieldState::Falling);
                            }
                        }
                    }
                    Movement::Down => {
                        if let &PlayfieldState::Falling = self.state.top() {
                            if self.block.is_valid_position(x, y - 1, self.block_template.block(&template)) {
                                self.falling_block.down();
                                self.gravity_delay.reset();
                                self.lock_delay.reset();
                            } else {
                                self.state.set(PlayfieldState::Locking);
                            }
                        }
                    }
                    Movement::Drop => {
                        if let &PlayfieldState::Falling = self.state.top() {
                            let (x, y) = self.block.get_ghost_block_pos(x, y, self.block_template.block(&template));
                            self.lock_falling_block_at(x, y);
                        }
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
                                self.lock_delay.reset();
                                if self.block.is_valid_position(x, y - 1, block) {
                                    self.state.set(PlayfieldState::Falling);
                                }
                                break;
                            }
                        }
                    }
                    Movement::Hold => {
                        if self.can_hold_falling_block {
                            let mut held_template = template;
                            held_template.order = 0;
                            if self.held_template.is_none() {
                                self.spawn_falling_block();
                            } else {
                                let new_template = self.held_template.as_ref().unwrap().clone();
                                self.spawn_falling_block_with_template(new_template);
                            }
                            self.held_template = Some(held_template);
                            self.can_hold_falling_block = false;
                        }
                    }
                }
            }
            _ => {}
        }

    }

    pub fn pause(&mut self) {
        match self.state.top() {
            &PlayfieldState::Paused => {}
            _ => {
                self.state.push(PlayfieldState::Paused);
            }
        }
    }

    pub fn resume(&mut self) {
        match self.state.top() {
            &PlayfieldState::Paused => {
                self.state.pop();
            }
            _ => {}
        }
    }

    fn change_to_falling_state(&mut self) {
        self.can_hold_falling_block = true;
        self.spawn_falling_block();
        self.state.set(PlayfieldState::Falling);
    }

    fn width_in_pixels(&self) -> i32 {
        self.block.width as i32 * self.block_size_in_pixels
    }

    fn height_in_pixels(&self) -> i32 {
        self.block.height as i32 * self.block_size_in_pixels
    }

    pub fn render_held_blocks(&self, renderer: &mut SoftwareRenderer, x: i32, y: i32, blocks_bitmap: &Bitmap) {
        if let Some(held_template) = self.held_template {
            let block = self.block_template.block(&held_template);

            for (col, row, cell) in block_iter!(block) {
                let x_offset = col as i32 * self.block_size_in_pixels;
                let y_offset = (block.height - row) as i32 * self.block_size_in_pixels;
                let x = x + x_offset;
                let y = y + (self.height_in_pixels() - y_offset);
                renderer.blit_sub_bitmap(x + 1, y + 1,
                                         self.block_size_in_pixels * cell.index,
                                         0,
                                         self.block_size_in_pixels,
                                         self.block_size_in_pixels, blocks_bitmap);
            }
        }
    }

    pub fn render_falling_block(&self, renderer: &mut SoftwareRenderer, x: i32, y: i32, blocks_bitmap: &Bitmap) {
        let x = self.x_offset_for_cells(x);
        let block = self.block_template.block(&self.falling_block.template);
        let (ghost_x, ghost_y) = self.block.get_ghost_block_pos(self.falling_block.x, self.falling_block.y, block);

        for (col, row, cell) in block_iter!(block) {
            // Ghost
            {
                let x_offset = (ghost_x + col as i32) * self.block_size_in_pixels;
                let y_offset = (ghost_y + row as i32) * self.block_size_in_pixels;
                let x = x + x_offset;
                let y = y + y_offset;
                renderer.rect(x + 1,
                              y + 1,
                              x + self.block_size_in_pixels - 1,
                              y + self.block_size_in_pixels - 1,
                              cell.color);
            }

            // Simply clip the block
            if self.falling_block.y + (row as i32) < self.block.height as i32 {
                let x_offset = (self.falling_block.x + col as i32) * self.block_size_in_pixels;
                let y_offset = (self.falling_block.y + row as i32) * self.block_size_in_pixels;
                let x = x + x_offset;
                let y = y + y_offset;
                renderer.blit_sub_bitmap(x + 1, y + 1,
                                         self.block_size_in_pixels * cell.index,
                                         0,
                                         self.block_size_in_pixels,
                                         self.block_size_in_pixels, blocks_bitmap);
            }
        }
    }

    pub fn render_cells(&self, renderer: &mut SoftwareRenderer, x: i32, y: i32, blocks_bitmap: &Bitmap) {
        let x = self.x_offset_for_cells(x);
        for (col, row, cell) in block_iter!(self.block) {
            let mut alpha = 1.0;
            if self.breaking_lines.contains(&row) {
                if self.is_blink {
                    continue;
                }

                alpha = 0.5;
            }

            let x_offset = (col as i32) * self.block_size_in_pixels;
            let y_offset = (row as i32) * self.block_size_in_pixels;
            let x = x + x_offset;
            let y = y + y_offset;
            renderer.blit_sub_bitmap_alpha(x + 1, y + 1,
                                           self.block_size_in_pixels * cell.index,
                                           0,
                                           self.block_size_in_pixels,
                                           self.block_size_in_pixels, blocks_bitmap,
                                           alpha);
        }
    }

    pub fn render_grids(&self, renderer: &mut SoftwareRenderer, x: i32, y: i32, color: RGBA) {
        let x = self.x_offset_for_cells(x);
        for row in 1..self.block.height {
            let y_offset = row as i32 * self.block_size_in_pixels;
            renderer.hline(y + y_offset, x, x + self.width_in_pixels(), color);
        }

        for col in 1..self.block.width {
            let x_offset = col as i32 * self.block_size_in_pixels;
            renderer.vline(x + x_offset, y, y + self.height_in_pixels(), color);
        }
    }

    pub fn render_borders(&self, renderer: &mut SoftwareRenderer, x: i32, y: i32, color: RGBA) {
        let x = self.x_offset_for_cells(x);
        renderer.rect(x, y,
                      x + self.width_in_pixels(),
                      y + self.height_in_pixels(),
                      color);
    }

    pub fn render_next_blocks(&self, renderer: &mut SoftwareRenderer, x: i32, y: i32, blocks_bitmap: &Bitmap) {
        let x = self.x_offset_for_next_blocks(x);
        for (i, template) in self.next_templates.iter().enumerate() {
            let block = self.block_template.block(template);

            for (col, row, cell) in block_iter!(block) {
                let x_offset = col as i32 * self.block_size_in_pixels;
                let y_offset = (block.height - row) as i32 * self.block_size_in_pixels;
                let x = x + x_offset;
                let y = y + (self.height_in_pixels() - y_offset) -
                        i as i32 * 4 * self.block_size_in_pixels;
                renderer.blit_sub_bitmap(x + 1, y + 1,
                                         self.block_size_in_pixels * cell.index,
                                         0,
                                         self.block_size_in_pixels,
                                         self.block_size_in_pixels, blocks_bitmap);
            }
        }
    }

    fn x_offset_for_cells(&self, x: i32) -> i32 {
        x + 5 * self.block_size_in_pixels
    }

    fn x_offset_for_next_blocks(&self, x: i32) -> i32 {
        self.x_offset_for_cells(x) + self.width_in_pixels() + self.block_size_in_pixels
    }

    pub fn render(&self, renderer: &mut SoftwareRenderer, x: i32, y: i32, blocks_bitmap: &Bitmap) {
        self.render_held_blocks(renderer, x, y, blocks_bitmap);
        self.render_cells(renderer, x, y, blocks_bitmap);
        self.render_grids(renderer, x, y, rgba(0.2, 0.2, 0.2, 1.0));
        self.render_borders(renderer, x, y, rgba(1.0, 1.0, 1.0, 1.0));
        self.render_next_blocks(renderer, x, y, blocks_bitmap);
    }

    pub fn update(&mut self, dt: f32) {
        self.breaking_lines = vec![];
        match self.state.top() {
            /*
            &PlayfieldState::Falling => {
                if self.block.is_valid_position(self.falling_block.x,
                                                self.falling_block.y - 1,
                                                &self.block_template.block(&self.falling_block.template)) {
                    self.gravity_delay.tick(dt);
                    if self.gravity_delay.is_expired() {
                        self.move_falling_block(Movement::Down);
                    }
                } else {
                    self.state.set(PlayfieldState::Locking);
                }
            }
            &PlayfieldState::Locking => {
                self.lock_delay.tick(dt);
                self.max_lock_delay.tick(dt);
                if self.lock_delay.is_expired() || self.max_lock_delay.is_expired() {
                    self.lock_falling_block();
                }
            }
            */
            &PlayfieldState::Breaking => {
                self.breaking_line_delay.tick(dt);
                self.blink_interval.tick(dt);
                if self.blink_interval.is_expired() {
                    self.blink_interval.reset();
                    self.is_blink = !self.is_blink;
                }

                self.breaking_lines = self.block.get_break_lines();
                if self.breaking_lines.len() == 0 {
                    self.breaking_line_delay.reset();
                    self.blink_interval.reset();
                    self.change_to_falling_state();
                } else if self.breaking_line_delay.is_expired() {
                    self.breaking_line_delay.reset();
                    self.blink_interval.reset();
                    self.block.break_lines();
                    self.change_to_falling_state();
                }
            }
            _ => {}
        }
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

    let mut context = Context {
        dt: 0.0,
        renderer: SoftwareRenderer::new(renderer, width, height),
        events: vec![],
    };

    let mut retris = Retris::new();

    let mut state_machine = StateMachine::new(Prepare::new());

    let mut event_pump = sdl2.event_pump().unwrap();

    let mut frame_last = PreciseTime::now();

    'running: loop {
        let frame_start = PreciseTime::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown {keycode: Some(Keycode::Escape), ..} => break 'running,
                /*
                Event::KeyDown {keycode: Some(Keycode::P), ..} => {
                    match retris.playfield.state.top() {
                        &PlayfieldState::Paused => retris.playfield.resume(),
                        _ => retris.playfield.pause(),
                    }
                }
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
                */
                _ => {}
            }
            context.events.push(event);
        }

        let now = PreciseTime::now();
        context.dt = frame_last.to(now).num_milliseconds() as f32 / 1000.0;
        frame_last = now;
        state_machine.update(&mut context, &mut retris);
        context.renderer.present(width, height);

        context.events.clear();

        let frame_end = PreciseTime::now();
        let _ = frame_start.to(frame_end);
        // println!("FPS: {}", (1000.0 / span.num_milliseconds() as f64) as u32);
    }
}
