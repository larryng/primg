use shape::Shape;
use worker::Worker;

pub struct State {
    pub shape: Shape,
    pub alpha: u8,
    pub score: f64,
}

impl State {
    pub fn new(shape: Shape, alpha: u8) -> State {
        State { shape, alpha, score: -1.0 }
    }

    pub fn energy(&mut self, worker: &mut Worker) -> f64 {
        if self.score < 0.0 {
            self.score = worker.energy(&self.shape, self.alpha);
        }
        self.score
    }
}