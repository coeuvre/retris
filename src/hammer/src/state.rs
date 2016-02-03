pub enum Trans<S> {
    Switch(S),
    Push(S),
    Pop,
}

pub fn switch<S>(state: S) -> Trans<S> {
    Trans::Switch(state)
}

pub fn push<S>(state: S) -> Trans<S> {
    Trans::Push(state)
}

pub fn pop<S>() -> Trans<S> {
    Trans::Pop
}

pub struct StateMachine<S> {
    stack: Vec<S>,
}

impl<S> StateMachine<S> {
    pub fn new(init: S) -> StateMachine<S> {
        StateMachine {
            stack: vec![init],
        }
    }

    pub fn current_state(&mut self) -> &S {
        self.stack.last().unwrap()
    }

    pub fn current_state_mut(&mut self) -> &mut S {
        self.stack.last_mut().unwrap()
    }

    pub fn trans(&mut self, trans: Trans<S>) {
        match trans {
            Trans::Switch(state) => {
                self.stack.pop();
                self.stack.push(state);
            }

            Trans::Push(state) => {
                self.stack.push(state);
            }

            Trans::Pop => {
                assert!(self.stack.len() > 0);
                self.stack.pop();
            }
        }
    }
}
