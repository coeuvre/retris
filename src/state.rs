pub enum Trans<C, G> {
    Switch(Box<State<Context=C, Game=G>>),
    Push(Box<State<Context=C, Game=G>>),
    Pop,
}

pub fn switch<C, G, S: State<Context=C, Game=G> + 'static>(state: S) -> Option<Trans<C, G>> {
    Some(Trans::Switch(Box::new(state)))
}

pub fn push<C, G, S: State<Context=C, Game=G> + 'static>(state: S) -> Option<Trans<C, G>> {
    Some(Trans::Push(Box::new(state)))
}

pub fn pop<C, G>() -> Option<Trans<C, G>> {
    Some(Trans::Pop)
}

pub trait State {
    type Context;
    type Game;

    fn update(&mut self, _ctx: &mut Self::Context, _game: &mut Self::Game) -> Option<Trans<Self::Context, Self::Game>> {
        None
    }
}

pub struct StateMachine<C, G> {
    stack: Vec<Box<State<Context=C, Game=G>>>,
}

impl<C, G> StateMachine<C, G> {
    pub fn new<S: State<Context=C, Game=G> + 'static>(state: S) -> StateMachine<C, G> {
        let stack: Vec<Box<State<Context=C, Game=G>>> = vec![Box::new(state)];
        StateMachine {
            stack: stack,
        }
    }

    pub fn update(&mut self, ctx: &mut C, game: &mut G) {
        while let Some(trans) = self.current_mut().update(ctx, game) {
            self.trans(trans);
        }
    }

    fn current_mut(&mut self) -> &mut State<Context=C, Game=G> {
        if let Some(state) = self.stack.last_mut() {
            &mut **state
        } else {
            // NOTE(coeuvre): There must be at least one state!
            unreachable!();
        }
    }

    fn trans(&mut self, trans: Trans<C, G>) {
        match trans {
            Trans::Switch(state) => self.switch(state),
            Trans::Push(state) => self.push(state),
            Trans::Pop => self.pop(),
        }
    }

    fn switch(&mut self, state: Box<State<Context=C, Game=G>>) {
        self.stack.pop();
        self.stack.push(state);
    }

    fn push(&mut self, state: Box<State<Context=C, Game=G>>) {
        self.stack.push(state);
    }

    fn pop(&mut self) {
        assert!(self.stack.len() > 0);
        self.stack.pop();
    }
}
