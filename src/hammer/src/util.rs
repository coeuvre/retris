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

