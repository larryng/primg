extern crate image;
extern crate num_cpus;
extern crate rand;
extern crate threadpool;

mod core;
mod model;
mod scanline;
mod shape;
mod state;
mod util;
mod worker;

pub use shape::ShapeType;

use core::{Pixels, Color};
use image::ImageFormat;
use image::{Pixel, Rgba, RgbaImage};
use model::Model;
use std::fs::File;
use scanline::Scanline;
use shape::Shape;
use worker::Worker;

const SIZE: usize = 256;

pub fn run(config: Config) {
    println!("{:?}", config);

//    (0)
//    let mut rng = rand::thread_rng();
//    println!("{:?}", Shape::random(config.t, 100, 100, &mut rng));

//    (1)
//    let mut rng = rand::thread_rng();
//    let mut t = Triangle::create_random(100, 100, &mut rng);
//    println!("{:?}", t);
//    for _ in 0..20 {
//        t = t.mutate(100, 100, &mut rng);
//        println!("{:?}", t);
//    }

//    (2)
//    let buf: &[u8] = &[255, 0, 0, 128, 0, 0, 255, 128];
//    image::save_buffer("test.png", buf, 2, 1, image::ColorType::RGBA(8));

//    (3)
//    let img = util::load_image(config.filepath.as_ref()).expect("couldn't load image");
//    let img = util::scaled_to_area(img, SIZE * SIZE).to_rgba();
//    img.save("out.bmp").expect("couldn't save image");

//    (4)
//    let mut rng = rand::thread_rng();
//    let mut t = Triangle {
//        x1: 25,
//        y1: 25,
//        x2: 0,
//        y2: 50,
//        x3: 80,
//        y3: 25,
//    };
//    let mut pixels = Pixels::new(100, 100);
//    let mut v = (0..101).map(|_| Scanline::empty()).collect();
//    let color = Color::new(255, 0, 0, 64);
//    for _ in 0..30 {
//        let lines = t.rasterize(100, 100, &mut v);
//        pixels.draw_lines(&color, &lines);
//    }
//    image::save_buffer("out.png", &pixels.buf, pixels.w as u32, pixels.h as u32, image::ColorType::RGBA(8));

//    (5)
//    let img = util::load_image(config.filepath.as_ref()).expect("couldn't load image");
//    let img = util::scaled_to_area(img, SIZE * SIZE).to_rgba();
//    let target = Pixels::from(img);
//    let mut current = Pixels::new(target.w, target.h);
//    current.erase(&target.average_color());
//    let count = 50;
//    let tw = target.w as i32 / count;
//    let th = target.h as i32 / count;
//    let mut v = (0..target.h + 1).map(|_| Scanline::empty()).collect();
//    for _ in 0..20 {
//        for i in 0..count {
//            for j in 0..count {
//                let x1 = i * tw;
//                let y1 = j * th;
//                let x2 = x1;
//                let y2 = y1 + th;
//                let x3 = x1 + tw;
//                let y3 = y1;
//                let t = Triangle { x1, y1, x2, y2, x3, y3 };
//                let lines = t.rasterize(target.w, target.h, &mut v);
//                let color = current.compute_color(&target, &lines, 128);
//                current.draw_lines(&color, lines);
//            }
//        }
//    }
//    image::save_buffer("out.png",
//                       &current.buf,
//                       current.w as u32,
//                       current.h as u32,
//                       image::ColorType::RGBA(8));

//    (6)
//    let img = util::load_image(config.filepath.as_ref()).expect("couldn't load image");
//    let img = util::scaled_to_area(img, SIZE * SIZE).to_rgba();
//    let target = Pixels::from(img);
//    let worker = Worker::new(&target);

//    (7)
    let img = util::load_image(config.in_path.as_ref()).expect("couldn't load image");
    let img = util::scaled_to_area(img, SIZE * SIZE);
    let mut model = Model::new(img, num_cpus::get());
    for _ in 0..config.num_shapes {
        model.step(config.shape_type, 128, 1000, 1);
    }
    model.save_current(&config.out_path).expect("wtf");

}

#[derive(Debug)]
pub struct Config {
    pub in_path: String,
    pub out_path: String,
    pub num_shapes: u32,
    pub shape_type: ShapeType,
    pub out_size: u32,
}

//
//struct A {}
//
//struct B<'a> {
//    borrowed_a: &'a A,
//}
//
//struct C<'a> {
//    a: A,
//    bs: Vec<B<'a>>,
//}
//
//impl<'a> C<'a> {
//    fn new<'b>() -> C<'b> {
//        let mut c: C<'b> = C { a: A {}, bs: Vec::new() };
//        let borrowed_a: &'b A = &c.a;
//        let b = B { borrowed_a };
//        c.bs.push(b);
//        c
//    }
//}

