use Hammer;

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

    fn update(&mut self, _hammer: &mut Hammer, _context: &mut Self::Context) -> Trans<Self> { next() }
}

pub struct StateMachine<S, C> {
    stack: Vec<S>,
    context: C,
}

impl<S: State<Context=C>, C> StateMachine<S, C> {
    pub fn new(state: S, context: C) -> StateMachine<S, C> {
        StateMachine {
            stack: vec![state],
            context: context,
        }
    }

    pub fn update(&mut self, hammer: &mut Hammer) {
        assert!(self.stack.len() > 0);

        loop {
            match self.stack.last_mut().unwrap().update(hammer, &mut self.context) {
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
