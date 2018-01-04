use shape::Shape;
use worker::Worker;

#[derive(Clone)]
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

    pub fn do_move(&mut self, worker: &mut Worker, undo: &mut State) {
        undo.copy_from(self);

        self.shape.mutate(worker.w, worker.h, &mut worker.rng);
        self.score = -1.0;
    }

    pub fn copy_from(&mut self, undo: &State) {
        self.shape = undo.shape.clone();
        self.alpha = undo.alpha;
        self.score = undo.score;
    }
}