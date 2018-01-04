use rand;
use std::sync::{Arc, RwLock};

use core::Pixels;
use scanline::Scanline;
use shape::{Shape, ShapeType};
use state::State;

pub struct Worker {
    pub w: usize,
    pub h: usize,
    pub target: Arc<Pixels>,
    pub current: Arc<RwLock<Pixels>>,
    pub buffer: Pixels,
    pub rng: rand::StdRng,
    pub scanlines: Vec<Scanline>,
    pub score: f64,
}

unsafe impl Sync for Worker {}

impl Worker {
    pub fn new(target: Arc<Pixels>, current: Arc<RwLock<Pixels>>) -> Worker {
        let w = target.w;
        let h = target.h;
        let buffer = Pixels::new(w, h);
        let rng = rand::StdRng::new().expect("wtf");
        let scanlines = Scanline::buffer(h);
        let score = -1.0;
        Worker { w, h, target, current, buffer, rng, scanlines, score }
    }

    pub fn init(&mut self, score: f64) {
        self.score = score;
    }

    pub fn energy(&mut self, shape: &Shape, alpha: u8) -> f64 {
        let lines = shape.rasterize(self.w, self.h, &mut self.scanlines);
        let current = self.current.read().unwrap();
        let color = current.compute_color(self.target.as_ref(), lines, alpha);
        self.buffer.copy_lines(&current, lines);
        self.buffer.draw_lines(&color, lines);
        Pixels::difference_partial(&self.target, &current, &self.buffer, self.score, lines)
    }

    pub fn best_hill_climb_state(&mut self, t: ShapeType, a: u8, n: u32, m: u8) -> State {
        let mut state = self.best_random_state(t, a, n);
        self.hill_climb(&mut state, 100);
        let mut best_state = state.clone();
        let mut best_energy = best_state.energy(self);
        for _ in 1..m {
            state = self.best_random_state(t, a, n);
            self.hill_climb(&mut state, 100);
            let energy = state.energy(self);
            if energy < best_energy {
                best_energy = energy;
                best_state.copy_from(&state);
            }
        }
        best_state
    }

    pub fn hill_climb(&mut self, state: &mut State, max_age: i32) {
        let mut undo = state.clone();
        let mut best_state = state.clone();
        let mut best_energy = best_state.energy(self);
        let mut age = 0;
        while age < max_age {
            state.do_move(self, &mut undo);
            let energy = state.energy(self);
            if energy > best_energy {
                state.copy_from(&undo);
            } else {
                best_energy = energy;
                best_state.copy_from(state);
                age -= 1;
            }
            age += 1;
        }
        state.copy_from(&best_state);
    }

    pub fn best_random_state(&mut self, t: ShapeType, a: u8, n: u32) -> State {
        let mut best_state = self.random_state(t, a);
        let mut best_energy = best_state.energy(self);
        for _ in 1..n {
            let mut state = self.random_state(t, a);
            let energy = state.energy(self);
            if energy < best_energy {
                best_energy = energy;
                best_state = state;
            }
        }
        best_state
    }

    pub fn random_state(&mut self, t: ShapeType, alpha: u8) -> State {
        let shape = Shape::random(t, self.w, self.h, &mut self.rng);
        State::new(shape, alpha)
    }
}
