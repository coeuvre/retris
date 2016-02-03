extern crate sdl2;
extern crate time;

use std::collections::VecDeque;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use time::PreciseTime;

use renderer::*;

pub mod bitmap;
pub mod util;
pub mod state;
pub mod renderer;

pub mod prelude {
    pub use sdl2::event::Event;
    pub use sdl2::keyboard::Keycode;

    pub use bitmap::*;
    pub use renderer::*;
    pub use state::*;
    pub use util::*;

    pub use Hammer;
    pub use Scene;
}

pub trait Scene {
    fn handle_event(&mut self, _event: &Event) {}
    fn update(&mut self, _dt: f32) {}
    fn render(&self, _renderer: &mut Renderer) {}
}

pub struct Hammer {
    pub dt: f32,
    pub renderer: Renderer,
    pub events: EventQueue,
}

impl Hammer {
    pub fn run<S>(mut scene: S) where S: Scene {
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

        let mut hammer = Hammer {
            dt: 0.0,
            renderer: SoftwareRenderer::new(renderer, width, height),
            events: EventQueue::new(),
        };

        let mut event_pump = sdl2.event_pump().unwrap();

        let mut frame_last = PreciseTime::now();

        'running: loop {
            let frame_start = PreciseTime::now();

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown {keycode: Some(Keycode::Escape), ..} => break 'running,
                    _ => {}
                }
                scene.handle_event(&event);
                hammer.events.push(event);
            }

            let now = PreciseTime::now();
            hammer.dt = frame_last.to(now).num_milliseconds() as f32 / 1000.0;
            frame_last = now;
            scene.update(hammer.dt);
            scene.render(&mut hammer.renderer);
            hammer.renderer.present(width, height);

            hammer.events.clear();

            let frame_end = PreciseTime::now();
            let _ = frame_start.to(frame_end);
            // println!("FPS: {}", (1000.0 / span.num_milliseconds() as f64) as u32);
        }
    }
}

pub struct EventQueue {
    events: VecDeque<Event>,
}

impl EventQueue {
    pub fn new() -> EventQueue {
        EventQueue {
            events: VecDeque::new(),
        }
    }

    pub fn poll(&mut self) -> EventQueuePollIter {
        EventQueuePollIter {
            queue: self,
        }
    }

    pub fn push(&mut self, event: Event) {
        self.events.push_back(event);
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}

pub struct EventQueuePollIter<'a> {
    queue: &'a mut EventQueue,
}

impl<'a> Iterator for EventQueuePollIter<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        self.queue.events.pop_front()
    }
}

