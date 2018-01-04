use super::SIZE;

use image;
use image::DynamicImage;
use std::io;
use std::sync::{Arc, RwLock};
use std::sync::mpsc;
use std::cmp;
use threadpool::ThreadPool;

use core::{Color, Pixels};
use shape::{Shape, ShapeType};
use scanline::Scanline;
use util;
use worker::Worker;

pub struct Model {
    n_workers: usize,
    sw: usize,
    sh: usize,
    background: Color,
    target: Arc<Pixels>,
    current: Arc<RwLock<Pixels>>,
    score: f64,
    shapes: Vec<Shape>,
    colors: Vec<Color>,
    workers: Vec<Arc<RwLock<Worker>>>,
    pool: ThreadPool,
    scanlines: Vec<Scanline>,
}

impl Model {
    pub fn new(img: DynamicImage, n_workers: usize) -> Model {
        let img = util::scaled_to_area(img, SIZE * SIZE).to_rgba();
        let target = Pixels::from(img);
        let sw = target.w;
        let sh = target.h;
        let background = target.average_color();
        let mut current = Pixels::new(sw, sh);
        current.erase(&background);
        let score = Pixels::difference_full(&target, &current);
        let target = Arc::new(target);
        let current = Arc::new(RwLock::new(current));
        let shapes = Vec::new();
        let colors = Vec::new();
        let workers = (0..n_workers).map(|_| Arc::new(RwLock::new(Worker::new(target.clone(), current.clone())))).collect();
        let pool = ThreadPool::new(n_workers);
        let scanlines = Scanline::buffer(sh);
        Model { n_workers, sw, sh, background, target, current, score, shapes, colors, workers, pool, scanlines }
    }

    pub fn step(&mut self, t: ShapeType, a: u8, n: u32, m: u8) {
        let (tx, rx) = mpsc::channel();

        let score = self.score;
        for worker in &self.workers {
            let worker = worker.clone();
            let tx = tx.clone();
            self.pool.execute(move || {
                let mut worker = worker.write().unwrap();
                worker.init(score);
                let mut state = worker.best_hill_climb_state(t, a, n, m);
                let energy = state.energy(&mut worker);
                tx.send((state, energy)).unwrap();
            });
        }

        let (state, energy) = rx.recv().unwrap();
        let mut best_state = state;
        let mut best_energy = energy;
        let mut count = 1;
        for (state, energy) in rx {
            count += 1;
            if energy < best_energy {
                best_state = state;
                best_energy = energy;
            }
            if count >= self.n_workers {
                break;
            }
        }
        println!("adding {:?}", best_state.shape);
        self.add(best_state.shape, best_state.alpha);
    }

    pub fn add(&mut self, shape: Shape, alpha: u8) {
        let mut current = self.current.write().unwrap();
        let before = current.clone();
        let lines = &shape.rasterize(self.sw, self.sh, &mut self.scanlines);
        let color = current.compute_color(&self.target, lines, alpha);
        current.draw_lines(&color, &lines);
        let score = Pixels::difference_partial(&self.target, &before, &current, self.score, lines);
        self.shapes.push(shape);
        self.colors.push(color);
        self.score = score;
    }

    pub fn save_rasterized(&self, path: &str, out_size: u32) -> io::Result<()> {
        let bigger = cmp::max(self.sw, self.sh);
        let scale = out_size as f64 / bigger as f64;
        let w = (self.sw as f64 * scale).round() as usize;
        let h = (self.sh as f64 * scale).round() as usize;
        println!("w={}, h={}, scale={}", w, h, scale);
        let mut img = vec![0; w * h * 4];
        util::erase(&mut img, &self.background);
        let mut buf = Scanline::buffer(h);

        for i in 0..self.shapes.len() {
            let shape = &self.shapes[i];
            let color = &self.colors[i];
            let lines = shape.scaled(scale).rasterize(w, h, &mut buf);
            util::draw_lines(&mut img, w, h, &color, lines);
        }

        image::save_buffer(path,
                           &img,
                           w as u32,
                           h as u32,
                           image::ColorType::RGBA(8))
    }

    // for debugging
    pub fn _save_current(&self, path: &str) -> io::Result<()> {
        let current = self.current.read().unwrap();
        image::save_buffer(path,
                           &current.buf,
                           self.sw as u32,
                           self.sh as u32,
                           image::ColorType::RGBA(8))
    }
}
