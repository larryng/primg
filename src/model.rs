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
    pub w: usize,
    pub h: usize,
    pub sw: usize,
    pub sh: usize,
    scale: f32,
    pub bg: Color,
    target: Arc<Pixels>,
    current: Arc<RwLock<Pixels>>,
    score: f32,
    shapes: Vec<Shape>,
    colors: Vec<Color>,
    workers: Vec<Arc<RwLock<Worker>>>,
    pool: ThreadPool,
    scanlines: Vec<Scanline>,
}

impl Model {
    pub fn new(img: DynamicImage, n_workers: usize, out_size: usize) -> Model {
        let img = util::scaled_to_area(img, SIZE * SIZE).to_rgba();
        let target = Pixels::from(img);
        let w = target.w;
        let h = target.h;
        let bigger = cmp::max(w, h);
        let scale = out_size as f32 / bigger as f32;
        let sw = util::scale_dimen(w as i32, scale) as usize;
        let sh = util::scale_dimen(h as i32, scale) as usize;
        let bg = target.average_color();
        let mut current = Pixels::new(w, h);
        current.erase(&bg);
        let score = Pixels::difference_full(&target, &current);
        let target = Arc::new(target);
        let current = Arc::new(RwLock::new(current));
        let shapes = Vec::new();
        let colors = Vec::new();
        let workers = (0..n_workers).map(|_| Arc::new(RwLock::new(Worker::new(target.clone(), current.clone())))).collect();
        let pool = ThreadPool::new(n_workers);
        let scanlines = Scanline::buffer(h);
        Model { n_workers, w, h, sw, sh, scale, bg, target, current, score, shapes, colors, workers, pool, scanlines }
    }

    pub fn step(&mut self, t: ShapeType, a: u8, n: u32, m: u8) -> (Shape, Color) {
        let (tx, rx) = mpsc::channel();

        let score = self.score;
        let m = cmp::max(1, m as usize / self.n_workers) as u8;
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
//        println!("adding {:?}", best_state.shape);
        self.add(best_state.shape, best_state.alpha)
    }

    pub fn add(&mut self, shape: Shape, alpha: u8) -> (Shape, Color) {
        let mut current = self.current.write().unwrap();
        let before = current.clone();
        let lines = &shape.rasterize(self.w, self.h, &mut self.scanlines);
        let color = current.compute_color(&self.target, lines, alpha);
        current.draw_lines(&color, &lines);
        let score = Pixels::difference_partial(&self.target, &before, &current, self.score, lines);
        self.shapes.push(shape.clone());
        self.colors.push(color);
        self.score = score;
        (shape, color)
    }

    pub fn svg(&self) -> String {
        let mut lines = vec![];
        lines.push(format!("<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" width=\"{}\" height=\"{}\">",
                           self.sw, self.sh));
        lines.push(format!("<rect x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"#{:02x}{:02x}{:02x}\" />",
                           self.sw, self.sh, self.bg.r(), self.bg.g(), self.bg.b()));
        lines.push(format!("<g transform=\"scale({}) translate(0.5 0.5)\">", self.scale));

        for (i, shape) in self.shapes.iter().enumerate() {
            let c = &self.colors[i];
            let attrs = format!("fill=\"#{:02x}{:02x}{:02x}\" fill-opacity=\"{}\"",
                                c.r(), c.g(), c.b(), c.a() as f32 / 255.0);
            lines.push(shape.svg(&attrs));
        }

        lines.push(String::from("</g>"));
        lines.push(String::from("</svg>"));
        lines.join("\n")
    }

    pub fn save_rasterized(&self, path: &str) -> io::Result<()> {
        let w = self.sw;
        let h = self.sh;
        let scale = self.scale;
//        println!("w={}, h={}, scale={}", w, h, scale);
        let mut img = vec![0; w * h * 4];
        util::erase(&mut img, &self.bg);
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
                           self.w as u32,
                           self.h as u32,
                           image::ColorType::RGBA(8))
    }
}
