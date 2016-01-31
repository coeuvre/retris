pub enum Trans<S: State> {
    /// Keep current state and go to next frame
    Next,
    Switch(S),
    Push(S),
    Pop,
}

pub fn next<S: State>() -> Trans<S> {
    Trans::Next
}

pub fn switch<S: State>(state: S) -> Trans<S> {
    Trans::Switch(state)
}

pub fn push<S: State>(state: S) -> Trans<S> {
    Trans::Push(state)
}

pub fn pop<S: State>() -> Trans<S> {
    Trans::Pop
}

pub trait State: Sized {
    type Context;
    type Game;

    fn update(&mut self, _ctx: &mut Self::Context, _game: &mut Self::Game) -> Trans<Self> { next() }
}

pub struct StateMachine<S> {
    stack: Vec<S>,
}

impl<S: State> StateMachine<S> {
    pub fn new(state: S) -> StateMachine<S> {
        StateMachine {
            stack: vec![state],
        }
    }

    pub fn update<C, G>(&mut self, ctx: &mut C, game: &mut G) where S: State<Context=C, Game=G> {
        assert!(self.stack.len() > 0);

        loop {
            match self.stack.last_mut().unwrap().update(ctx, game) {
                Trans::Next => break,
                Trans::Switch(state) => self.switch(state),
                Trans::Push(state) => self.push(state),
                Trans::Pop => self.pop(),
            }
        }
    }

    fn switch(&mut self, state: S) {
        self.stack.pop();
        self.stack.push(state);
    }

    fn push(&mut self, state: S) {
        self.stack.push(state);
    }

    fn pop(&mut self) {
        assert!(self.stack.len() > 0);
        self.stack.pop();
    }
}
