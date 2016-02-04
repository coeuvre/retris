extern crate rand;
extern crate hammer;

use hammer::prelude::*;

use block::*;

#[macro_use]
mod block;

pub enum GameState {
    Running,
    Paused,
}

pub struct Game {
    state_machine: StateMachine<GameState>,

    blocks: Bitmap,
    playfield: Playfield,
}

impl Game {
    pub fn new() -> Game {
        let blocks = Bitmap::open("./assets/blocks.bmp").unwrap();
        Game {
            state_machine: StateMachine::new(GameState::Running),

            playfield: Playfield::new(10, 20, blocks.height() as i32),
            blocks: blocks,
        }
    }
}

impl Scene for Game {
    fn handle_event(&mut self, event: &Event) {
        match *self.state_machine.current_state() {
            GameState::Running => {
                match *event {
                    Event::KeyDown {keycode: Some(Keycode::P), ..} => {
                        self.state_machine.trans(push(GameState::Paused));
                    }
                    _ => {
                        self.playfield.handle_event(event);
                    }
                }
            }

            GameState::Paused => {
                match *event {
                    Event::KeyDown {keycode: Some(Keycode::P), ..} => {
                        self.state_machine.trans(pop());
                    }
                    _ => {}
                }
            }
        }
    }

    fn update(&mut self, dt: f32) {
        match *self.state_machine.current_state() {
            GameState::Running => {
                self.playfield.update(dt);
            }

            GameState::Paused => {}
        }
    }

    fn render(&self, renderer: &mut Renderer) {
        self.playfield.render(renderer, 32, 32, &self.blocks);
    }
}

#[derive(Debug)]
pub enum PlayfieldState {
    Prepare {
        countdown: Timer,
    },
    Spawn {
        spawn_delay: Timer,
    },
    Falling {
        gravity_delay: Timer,
    },
    Locking {
        lock_delay: Timer,
        is_immediately: bool,
    },
    Breaking {
        breaking_line_delay: Timer,
        blink_delay: Timer,
    },
    Lost,
}

impl PlayfieldState {
    pub fn prepare() -> PlayfieldState {
        PlayfieldState::Prepare {
            countdown: Timer::new(0.0),
        }
    }

    pub fn spawn() -> PlayfieldState {
        PlayfieldState::Spawn {
            spawn_delay: Timer::new(0.0),
        }
    }

    pub fn falling() -> PlayfieldState {
        PlayfieldState::Falling {
            gravity_delay: Timer::new(gravity_to_delay(0.015625)),
        }
    }

    pub fn locking() -> PlayfieldState {
        PlayfieldState::Locking {
            lock_delay: Timer::new(frames_to_seconds(30.0)),
            is_immediately: false,
        }
    }

    pub fn locking_immediately() -> PlayfieldState {
        PlayfieldState::Locking {
            lock_delay: Timer::new(frames_to_seconds(0.0)),
            is_immediately: true,
        }
    }

    pub fn breaking() -> PlayfieldState {
        PlayfieldState::Breaking {
            breaking_line_delay: Timer::new(0.25),
            blink_delay: Timer::new(0.08),
        }
    }

    pub fn lost() -> PlayfieldState {
        PlayfieldState::Lost
    }
}

fn frames_to_seconds(frames: f32) -> f32 {
    // NOTE: Assuming the game run under 60 FPS.
    let fps = 60.0;
    frames / fps
}

fn gravity_to_delay(gravity: f32) -> f32 {
    // NOTE: gravity is how many cells per frame, so the inverse
    // is how many frames per cell.
    frames_to_seconds(1.0 / gravity)
}

pub struct Playfield {
    state_machine: StateMachine<PlayfieldState>,
    raw: PlayfieldRaw,
}

pub struct PlayfieldRaw {
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

impl PlayfieldRaw {
    pub fn new(width: usize, height: usize, block_size_in_pixels: i32) -> PlayfieldRaw {
        let block_template = BlockTemplate::new();
        PlayfieldRaw {
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

    fn handle_common_event(&mut self, event: &Event) {
        match *event {
            Event::KeyDown {keycode: Some(Keycode::Up), ..} => {
                self.rotate_falling_block(1);
            }
            Event::KeyDown {keycode: Some(Keycode::Z), ..} => {
                self.rotate_falling_block(-1);
            }
            Event::KeyDown {keycode: Some(Keycode::Left), ..} => {
                self.move_falling_block_by(-1, 0);
            }
            Event::KeyDown {keycode: Some(Keycode::Right), ..} => {
                self.move_falling_block_by(1, 0);
            }
            Event::KeyDown {keycode: Some(Keycode::C), ..} => {
                self.hold_falling_block();
            }
            _ => {}
        }
    }

    pub fn handle_event(&mut self, event: &Event, state: &mut PlayfieldState) -> Option<Trans<PlayfieldState>> {
        match *state {
            PlayfieldState::Falling { ref mut gravity_delay } => {
                match *event {
                    Event::KeyDown {keycode: Some(Keycode::Down), ..} => {
                        self.move_falling_block_by(0, -1);
                        gravity_delay.reset();
                    }
                    Event::KeyDown {keycode: Some(Keycode::Space), ..} => {
                        self.drop_falling_block();
                        return Some(switch(PlayfieldState::locking_immediately()));
                    }
                    _ => {
                        self.handle_common_event(event);
                    }
                }

                if !self.can_move_falling_block_by(0, -1) {
                    return Some(switch(PlayfieldState::locking()));
                }
            }

            PlayfieldState::Locking { .. } => {
                self.handle_common_event(event);

                if self.can_move_falling_block_by(0, -1) {
                    return Some(switch(PlayfieldState::falling()));
                }
            }

            _ => {}
        }

        None
    }

    fn lock(&mut self) -> Trans<PlayfieldState> {
        if self.is_falling_block_out_of_bounds() {
            // NOTE: Partial lock out
            switch(PlayfieldState::lost())
        } else {
            self.lock_falling_block();

            if self.has_lines_to_break() {
                switch(PlayfieldState::breaking())
            } else {
                switch(PlayfieldState::spawn())
            }
        }
    }

    pub fn update(&mut self, dt: f32, state: &mut PlayfieldState) -> Option<Trans<PlayfieldState>> {
        //println!("{:?}", state);

        match *state {
            PlayfieldState::Prepare { ref mut countdown } => {
                countdown.tick(dt);

                println!("coutndown: {:.2}", countdown.elapsed());

                if countdown.is_expired() {
                    return Some(switch(PlayfieldState::spawn()))
                }
            }

            PlayfieldState::Spawn { ref mut spawn_delay } => {
                assert!(self.falling_block.is_none());

                spawn_delay.tick(dt);
                if spawn_delay.is_expired() {
                    self.spawn_falling_block();

                    if self.can_move_falling_block_by(0, -1) {
                        return Some(switch(PlayfieldState::falling()));
                    } else {
                        // NOTE: Block out
                        return Some(switch(PlayfieldState::lost()));
                    }
                }
            }

            PlayfieldState::Falling { ref mut gravity_delay } => {
                assert!(self.can_move_falling_block_by(0, -1));

                gravity_delay.tick(dt);
                if gravity_delay.is_expired() {
                    self.move_falling_block_by(0, -1);
                    gravity_delay.reset();

                    if !self.can_move_falling_block_by(0, -1) {
                        return Some(switch(PlayfieldState::locking()));
                    }
                }
            }

            PlayfieldState::Locking {
                ref mut lock_delay,
                is_immediately,
            } => {
                assert!(self.falling_block.is_some());
                assert!(!self.can_move_falling_block_by(0, -1));

                if is_immediately {
                    return Some(self.lock());
                }

                lock_delay.tick(dt);
                self.max_lock_delay.tick(dt);

                if lock_delay.is_expired() || self.max_lock_delay.is_expired() {
                    return Some(self.lock());
                }
            }

            PlayfieldState::Breaking{
                ref mut breaking_line_delay,
                ref mut blink_delay,
            } => {
                assert!(self.falling_block.is_none());
                assert!(self.has_lines_to_break());

                breaking_line_delay.tick(dt);
                if breaking_line_delay.is_expired() {
                    self.break_lines();
                    return Some(switch(PlayfieldState::spawn()));
                }

                blink_delay.tick(dt);
                if blink_delay.is_expired() {
                    blink_delay.reset();
                    self.blink_breaking_lines();
                }
            }

            PlayfieldState::Lost => {}
        }

        None
    }
}

impl Playfield {
    pub fn new(width: usize, height: usize, block_size_in_pixels: i32) -> Playfield {
        Playfield {
            state_machine: StateMachine::new(PlayfieldState::Prepare {
                countdown: Timer::new(0.0),
            }),

            raw: PlayfieldRaw::new(width, height, block_size_in_pixels),
        }
    }

    pub fn handle_event(&mut self, event: &Event) {
        while let Some(trans) = self.raw.handle_event(event, self.state_machine.current_state_mut()) {
            self.state_machine.trans(trans);
        }
    }

    pub fn update(&mut self, dt: f32) {
        while let Some(trans) = self.raw.update(dt, self.state_machine.current_state_mut()) {
            self.state_machine.trans(trans);
        }
    }

    pub fn render(&self, renderer: &mut Renderer, x: i32, y: i32, blocks_bitmap: &Bitmap) {
        self.raw.render(renderer, x, y, blocks_bitmap);
    }
}


fn main() {
    let retris = Game::new();
    Hammer::new().title("Retris").resolution(800, 800).run(retris);
}
