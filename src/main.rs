extern crate rand;
extern crate hammer;
extern crate sdl2;

use std::collections::VecDeque;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use hammer::prelude::*;

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

pub struct Prepare {
    countdown: Timer,
}

impl Prepare {
    pub fn new() -> Prepare {
        Prepare {
            countdown: Timer::new(0.0),
        }
    }

    pub fn update(&mut self, hammer: &mut Hammer, _context: &mut Retris) -> Trans<PlayfieldState> {
        self.countdown.tick(hammer.dt);

        println!("coutndown: {:.2}", self.countdown.elapsed());

        if self.countdown.is_expired() {
            return switch(PlayfieldState::Spawn(Spawn::new()));
        }

        next()
    }
}

pub struct Spawn {
    spawn_delay: Timer,
}

impl Spawn {
    pub fn new() -> Spawn {
        Spawn {
            spawn_delay: Timer::new(0.0),
        }
    }

    pub fn update(&mut self, hammer: &mut Hammer, context: &mut Retris) -> Trans<PlayfieldState> {
        let dt = hammer.dt;
        let ref mut renderer = hammer.renderer;
        let ref mut playfield = context.playfield;

        assert!(playfield.falling_block.is_none());

        self.spawn_delay.tick(dt);
        if self.spawn_delay.is_expired() {
            playfield.spawn_falling_block();

            if playfield.can_move_falling_block_by(0, -1) {
                return switch(PlayfieldState::Falling(Falling::new()));
            } else {
                // NOTE(coeuvre): Block out
                return switch(PlayfieldState::Lost);
            }
        }

        playfield.render(renderer, 32, 32, &context.blocks);

        next()
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

    pub fn update(&mut self, hammer: &mut Hammer, context: &mut Retris) -> Trans<PlayfieldState> {
        let dt = hammer.dt;
        let ref mut renderer = hammer.renderer;
        let ref mut playfield = context.playfield;

        assert!(playfield.can_move_falling_block_by(0, -1));

        for event in hammer.events.poll() {
            match event {
                Event::KeyDown {keycode: Some(Keycode::Down), ..} => {
                    playfield.move_falling_block_by(0, -1);
                    self.gravity_delay.reset();
                }
                Event::KeyDown {keycode: Some(Keycode::Space), ..} => {
                    playfield.drop_falling_block();
                    return switch(PlayfieldState::Locking(Locking::immediately()));
                }
                _ => {
                    if let Some(trans) = PlayfieldState::handle_common_event(event, playfield) {
                        return trans;
                    }
                }
            }
        }

        self.gravity_delay.tick(dt);
        if self.gravity_delay.is_expired() {
            playfield.move_falling_block_by(0, -1);
            self.gravity_delay.reset();
        }

        if !playfield.can_move_falling_block_by(0, -1) {
            return switch(PlayfieldState::Locking(Locking::new()));
        }

        playfield.render(renderer, 32, 32, &context.blocks);

        next()
    }
}

pub struct Locking {
    lock_delay: Timer,
    is_immediately: bool,
}

impl Locking {
    pub fn new() -> Locking {
        Locking {
            lock_delay: Timer::new(frames_to_seconds(30.0)),
            is_immediately: false,
        }
    }

    pub fn immediately() -> Locking {
        Locking {
            lock_delay: Timer::new(frames_to_seconds(0.0)),
            is_immediately: true,
        }
    }

    fn lock(&mut self, playfield: &mut Playfield) -> Trans<PlayfieldState> {
        if playfield.is_falling_block_out_of_bounds() {
            // NOTE(coeuvre): Partial lock out
            switch(PlayfieldState::Lost)
        } else {
            playfield.lock_falling_block();

            if playfield.has_lines_to_break() {
                switch(PlayfieldState::Breaking(Breaking::new()))
            } else {
                switch(PlayfieldState::Spawn(Spawn::new()))
            }
        }
    }

    pub fn update(&mut self, hammer: &mut Hammer, context: &mut Retris) -> Trans<PlayfieldState> {
        let dt = hammer.dt;
        let ref mut renderer = hammer.renderer;
        let ref mut playfield = context.playfield;

        assert!(playfield.falling_block.is_some());
        assert!(!playfield.can_move_falling_block_by(0, -1));

        if self.is_immediately {
            return self.lock(playfield);
        }

        for event in hammer.events.poll() {
            if let Some(trans) = PlayfieldState::handle_common_event(event, playfield) {
                return trans;
            }
        }

        if playfield.can_move_falling_block_by(0, -1) {
            return switch(PlayfieldState::Falling(Falling::new()));
        } else {
            self.lock_delay.tick(dt);
            playfield.max_lock_delay.tick(dt);

            if self.lock_delay.is_expired() || playfield.max_lock_delay.is_expired() {
                return self.lock(playfield);
            }
        }

        playfield.render(renderer, 32, 32, &context.blocks);

        next()
    }
}

pub struct Breaking {
    breaking_line_delay: Timer,
    blink_delay: Timer,
}

impl Breaking {
    pub fn new() -> Breaking {
        Breaking {
            breaking_line_delay: Timer::new(0.25),
            blink_delay: Timer::new(0.08),
        }
    }

    pub fn update(&mut self, hammer: &mut Hammer, context: &mut Retris) -> Trans<PlayfieldState> {
        let dt = hammer.dt;
        let ref mut renderer = hammer.renderer;
        let ref mut playfield = context.playfield;

        assert!(playfield.falling_block.is_none());
        assert!(playfield.has_lines_to_break());

        self.breaking_line_delay.tick(dt);
        if self.breaking_line_delay.is_expired() {
            playfield.break_lines();
            return switch(PlayfieldState::Spawn(Spawn::new()));
        }

        self.blink_delay.tick(dt);
        if self.blink_delay.is_expired() {
            self.blink_delay.reset();
            playfield.blink_breaking_lines();
        }

        playfield.render(renderer, 32, 32, &context.blocks);

        next()
    }
}

pub enum PlayfieldState {
    Prepare(Prepare),
    Spawn(Spawn),
    Falling(Falling),
    Locking(Locking),
    Breaking(Breaking),
    Paused,
    Lost,
}

impl PlayfieldState {
    fn handle_common_event(event: Event, playfield: &mut Playfield) -> Option<Trans<PlayfieldState>> {
        match event {
            Event::KeyDown {keycode: Some(Keycode::P), ..} => {
                return Some(push(PlayfieldState::Paused));
            }
            Event::KeyDown {keycode: Some(Keycode::Up), ..} => {
                playfield.rotate_falling_block(1);
            }
            Event::KeyDown {keycode: Some(Keycode::Z), ..} => {
                playfield.rotate_falling_block(-1);
            }
            Event::KeyDown {keycode: Some(Keycode::Left), ..} => {
                playfield.move_falling_block_by(-1, 0);
            }
            Event::KeyDown {keycode: Some(Keycode::Right), ..} => {
                playfield.move_falling_block_by(1, 0);
            }
            Event::KeyDown {keycode: Some(Keycode::C), ..} => {
                playfield.hold_falling_block();
            }
            _ => {}
        }

        None
    }
}

impl State for PlayfieldState {
    type Context = Retris;

    fn update(&mut self, hammer: &mut Hammer, context: &mut Retris) -> Trans<PlayfieldState> {
        match *self {
            PlayfieldState::Prepare(ref mut prepare) => {
                prepare.update(hammer, context)
            }
            PlayfieldState::Spawn(ref mut spawn) => {
                spawn.update(hammer, context)
            }
            PlayfieldState::Falling(ref mut falling) => {
                falling.update(hammer, context)
            }
            PlayfieldState::Locking(ref mut locking) => {
                locking.update(hammer, context)
            }
            PlayfieldState::Breaking(ref mut breaking) => {
                breaking.update(hammer, context)
            }
            PlayfieldState::Paused => {
                let ref mut renderer = hammer.renderer;
                let ref mut playfield = context.playfield;

                for event in hammer.events.poll() {
                    match event {
                        Event::KeyDown {keycode: Some(Keycode::P), ..} => {
                            return pop();
                        }
                        _ => {}
                    }
                }

                playfield.render(renderer, 32, 32, &context.blocks);

                next()
            }
            PlayfieldState::Lost => {
                let ref mut renderer = hammer.renderer;
                let ref mut playfield = context.playfield;
                playfield.render(renderer, 32, 32, &context.blocks);
                next()
            }
        }
    }
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

pub struct BlockTemplateGenerator {
    next_templates: VecDeque<BlockTemplateRef>,
}

impl BlockTemplateGenerator {
    pub fn new(block_template: &BlockTemplate) -> BlockTemplateGenerator {
        BlockTemplateGenerator {
            next_templates: vec![
                BlockTemplateGenerator::generate_raw(block_template),
                BlockTemplateGenerator::generate_raw(block_template),
                BlockTemplateGenerator::generate_raw(block_template),
            ].into_iter().collect(),
        }
    }

    pub fn generate(&mut self, block_template: &BlockTemplate) -> BlockTemplateRef {
        let new_template = BlockTemplateGenerator::generate_raw(block_template);
        self.next_templates.push_back(new_template);
        self.next_templates.pop_front().unwrap()
    }

    pub fn next_templates(&self) -> &[BlockTemplateRef] {
        self.next_templates.as_slices().0
    }

    fn generate_raw(block_template: &BlockTemplate) -> BlockTemplateRef {
        let shape = rand::random::<usize>() % block_template.templates.len();
        let order = 0;
        let order_max = block_template.templates[shape].len();
        BlockTemplateRef {
            shape: shape,
            order: order,
            order_max: order_max,
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

    pub fn move_by(&mut self, dx: i32, dy: i32) {
        self.x += dx;
        self.y += dy;
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
    block: Block,

    falling_block: Option<FallingBlock>,

    generator: BlockTemplateGenerator,
    block_template: BlockTemplate,

    held_template: Option<BlockTemplateRef>,
    can_hold_falling_block: bool,

    max_lock_delay: Timer,

    breaking_lines: Vec<usize>,
    is_breaking_lines_visible: bool,

    block_size_in_pixels: i32,
}

impl Playfield {
    pub fn new(width: usize, height: usize, block_size_in_pixels: i32) -> Playfield {
        let block_template = BlockTemplate::new();
        Playfield {
            block: Block::new(width, height),

            falling_block: None,

            generator: BlockTemplateGenerator::new(&block_template),
            block_template: block_template,

            held_template: None,
            can_hold_falling_block: true,

            max_lock_delay: Timer::new(frames_to_seconds(60.0)),

            breaking_lines: vec![],
            is_breaking_lines_visible: true,

            block_size_in_pixels: block_size_in_pixels,
        }
    }

    fn spawn_falling_block(&mut self) {
        let template = self.generator.generate(&self.block_template);
        self.spawn_falling_block_with(template);
    }

    fn spawn_falling_block_with(&mut self, template: BlockTemplateRef) {
        let bottom = self.block_template.block(&template).bottom();
        self.falling_block = Some(FallingBlock::new(3, self.block.height as i32 - bottom as i32, template));
    }

    pub fn drop_falling_block(&mut self) {
        if let Some(ref mut falling_block) = self.falling_block {
            let (x, y) = self.block.get_ghost_block_pos(falling_block.x,
                                                        falling_block.y,
                                                        self.block_template.block(&falling_block.template));
            falling_block.move_to(x, y);
        }
    }

    pub fn hold_falling_block(&mut self) {
        if self.can_hold_falling_block {
            let falling_block = self.falling_block.take();
            if let Some(falling_block) = falling_block {
                let mut template = falling_block.template;
                template.order = 0;
                if let Some(held_template) = self.held_template.take() {
                    self.spawn_falling_block_with(held_template);
                } else {
                    self.spawn_falling_block();
                }
                self.held_template = Some(template);
                self.can_hold_falling_block = false;
            }
        }
    }

    pub fn rotate_falling_block(&mut self, direction: i32) {
        if let Some(ref mut falling_block) = self.falling_block {
            let mut new_template = falling_block.template;
            if direction > 0 {
                new_template.rrotate();
            } else {
                new_template.lrotate();
            }

            let table = self.block_template.wall_kick_table(&falling_block.template,
                                                            &new_template);
            for &(dx, dy) in table {
                if self.block.is_valid_position(falling_block.x + dx,
                                                falling_block.y + dy,
                                                &self.block_template
                                                     .block(&new_template)) {
                    falling_block.move_by(dx, dy);
                    falling_block.template = new_template;
                    break;
                }
            }
        }
    }

    pub fn can_move_falling_block_by(&self, dx: i32, dy: i32) -> bool {
        if let Some(ref falling_block) = self.falling_block {
            self.block.is_valid_position(falling_block.x + dx,
                                         falling_block.y + dy,
                                         &self.block_template
                                              .block(&falling_block.template))
        } else {
            false
        }
    }

    pub fn move_falling_block_by(&mut self, dx: i32, dy: i32) {
        if self.can_move_falling_block_by(dx, dy) {
            self.falling_block.as_mut().unwrap().move_by(dx, dy);
        }
    }

    pub fn is_falling_block_out_of_bounds(&self) -> bool {
        if let Some(ref falling_block) = self.falling_block {
            self.block.is_out_of_bounds(falling_block.x,
                                        falling_block.y,
                                        self.block_template.block(&falling_block.template))
        } else {
            true
        }
    }

    pub fn lock_falling_block(&mut self) {
        assert!(!self.is_falling_block_out_of_bounds());
        let falling_block = self.falling_block.take();
        if let Some(falling_block) = falling_block {
            self.block.set_with_block(falling_block.x,
                                      falling_block.y,
                                      self.block_template.block(&falling_block.template));
            self.can_hold_falling_block = true;
            self.max_lock_delay.reset();
        }
    }

    pub fn has_lines_to_break(&mut self) -> bool {
        self.breaking_lines = self.block.get_break_lines();
        self.breaking_lines.len() > 0
    }

    pub fn blink_breaking_lines(&mut self) {
        self.is_breaking_lines_visible = !self.is_breaking_lines_visible;
    }

    pub fn break_lines(&mut self) {
        self.block.break_lines();
        self.breaking_lines.clear();
        self.is_breaking_lines_visible = true;
    }

    fn width_in_pixels(&self) -> i32 {
        self.block.width as i32 * self.block_size_in_pixels
    }

    fn height_in_pixels(&self) -> i32 {
        self.block.height as i32 * self.block_size_in_pixels
    }

    fn render_held_blocks(&self, renderer: &mut SoftwareRenderer, x: i32, y: i32, blocks_bitmap: &Bitmap) {
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

    fn render_falling_block(&self, renderer: &mut SoftwareRenderer, x: i32, y: i32, blocks_bitmap: &Bitmap) {
        let x = self.x_offset_for_cells(x);
        if let Some(ref falling_block) = self.falling_block {
            let block = self.block_template.block(&falling_block.template);

            for (col, row, cell) in block_iter!(block) {
                // Simply clip the block
                if falling_block.y + (row as i32) < self.block.height as i32 {
                    let x_offset = (falling_block.x + col as i32) * self.block_size_in_pixels;
                    let y_offset = (falling_block.y + row as i32) * self.block_size_in_pixels;
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
    }

    fn render_ghost_block(&self, renderer: &mut SoftwareRenderer, x: i32, y: i32) {
        let x = self.x_offset_for_cells(x);
        if let Some(ref falling_block) = self.falling_block {
            let block = self.block_template.block(&falling_block.template);
            let (ghost_x, ghost_y) = self.block
                                         .get_ghost_block_pos(falling_block.x,
                                                              falling_block.y,
                                                              self.block_template.block(&falling_block.template));

            for (col, row, cell) in block_iter!(block) {
                // Simply clip the block
                if ghost_y + (row as i32) < self.block.height as i32 {
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
            }
        }
    }

    fn render_cells(&self, renderer: &mut SoftwareRenderer, x: i32, y: i32, blocks_bitmap: &Bitmap) {
        let x = self.x_offset_for_cells(x);
        for (col, row, cell) in block_iter!(self.block) {
            let mut alpha = 1.0;
            if self.breaking_lines.contains(&row) {
                if !self.is_breaking_lines_visible {
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

    fn render_grids(&self, renderer: &mut SoftwareRenderer, x: i32, y: i32, color: RGBA) {
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

    fn render_borders(&self, renderer: &mut SoftwareRenderer, x: i32, y: i32, color: RGBA) {
        let x = self.x_offset_for_cells(x);
        renderer.rect(x, y,
                      x + self.width_in_pixels(),
                      y + self.height_in_pixels(),
                      color);
    }

    fn render_next_blocks(&self, renderer: &mut SoftwareRenderer, x: i32, y: i32, blocks_bitmap: &Bitmap) {
        let x = self.x_offset_for_next_blocks(x);
        for (i, template) in self.generator.next_templates().iter().enumerate() {
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
        if !self.falling_block.is_none() {
            self.render_ghost_block(renderer, x, y);
            self.render_falling_block(renderer, x, y, blocks_bitmap);
        }
        self.render_cells(renderer, x, y, blocks_bitmap);
        self.render_grids(renderer, x, y, rgba(0.2, 0.2, 0.2, 1.0));
        self.render_borders(renderer, x, y, rgba(1.0, 1.0, 1.0, 1.0));
        self.render_next_blocks(renderer, x, y, blocks_bitmap);
    }
}


fn main() {
    let state = PlayfieldState::Prepare(Prepare::new());
    let context = Retris::new();
    let state_machine = StateMachine::new(state, context);
    Hammer::run(state_machine);
}
