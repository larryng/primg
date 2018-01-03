use super::SIZE;

use image;
use image::{DynamicImage, ImageBuffer};
use std::io;
use std::rc::Rc;
use std::cell::RefCell;

use core::{Color, Pixels};
use shape::{Shape, ShapeType};
use scanline::Scanline;
use util;
use worker::Worker;

pub struct Model {
    sw: usize,
    sh: usize,
    background: Color,
    target: Rc<Pixels>,
    current: Rc<RefCell<Pixels>>,
    score: f64,
    shapes: Vec<Shape>,
    colors: Vec<Color>,
    workers: Vec<Worker>,
    scanlines: Vec<Scanline>,
}

impl Model {
    pub fn new(img: DynamicImage, nworkers: u32) -> Model {
        let img = util::scaled_to_area(img, SIZE * SIZE).to_rgba();
        let target = Pixels::from(img);
        let sw = target.w;
        let sh = target.h;
        let background = target.average_color();
        let mut current = Pixels::new(sw, sh);
        current.erase(&background);
        let score = Pixels::difference_full(&target, &current);
        let target = Rc::new(target);
        let current = Rc::new(RefCell::new(current));
        let shapes = Vec::new();
        let colors = Vec::new();
        let workers = (0..nworkers).map(|_| Worker::new(Rc::clone(&target), Rc::clone(&current))).collect();
        let scanlines = (0..sh + 1).map(|_| Scanline::empty()).collect();
        Model { sw, sh, background, target, current, score, shapes, colors, workers, scanlines }
    }

    pub fn step(&mut self, t: ShapeType, a: u8, n: u32, m: u8) {
        let state = {
            let worker = &mut self.workers[0];
            worker.best_hill_climb_state(t, a, n, m)
        };
        println!("adding shape {:?}", state.shape);
        self.add(state.shape, state.alpha);
    }

    pub fn add(&mut self, shape: Shape, alpha: u8) {
        let mut current = self.current.borrow_mut();
        let before = current.clone();
        let lines = &shape.rasterize(self.sw, self.sh, &mut self.scanlines);
        let color = current.compute_color(&self.target, lines, alpha);
        current.draw_lines(&color, &lines);
        let score = Pixels::difference_partial(&self.target, &before, &current, self.score, lines);
        self.shapes.push(shape);
        self.colors.push(color);
        self.score = score;
    }

    // for debugging
    pub fn save_current(&self, path: &str) -> io::Result<()> {
        let current = self.current.borrow();
        image::save_buffer(path,
                           &current.buf,
                           self.sw as u32,
                           self.sh as u32,
                           image::ColorType::RGBA(8))
    }
}
