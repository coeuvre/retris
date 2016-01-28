pub trait State {
    type Context;
    type Game;

    fn update(&mut self, _ctx: &mut Self::Context, _game: &mut Self::Game) -> Trans<Self::Context, Self::Game> {
        Trans::none()
    }
}

pub enum Trans<C, G> {
    None,
    Switch(Box<State<Context=C, Game=G>>),
    Push(Box<State<Context=C, Game=G>>),
    Pop,
}

impl<C, G> Trans<C, G> {
    pub fn none() -> Trans<C, G> {
        Trans::None
    }

    pub fn switch<S: State<Context=C, Game=G> + 'static>(state: S) -> Trans<C, G> {
        Trans::Switch(Box::new(state))
    }

    pub fn push<S: State<Context=C, Game=G> + 'static>(state: S) -> Trans<C, G> {
        Trans::Push(Box::new(state))
    }

    pub fn pop() -> Trans<C, G> {
        Trans::Pop
    }

    pub fn is_none(&self) -> bool {
        match self {
            &Trans::None => true,
            _ => false,
        }
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
        let trans = self.current_mut().update(ctx, game);
        self.trans(trans);
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
            Trans::None => (),
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
