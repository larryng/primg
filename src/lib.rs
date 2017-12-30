extern crate image;
extern crate rand;

mod core;
mod scanline;
mod shape;
mod triangle;
mod util;
mod worker;

pub use shape::ShapeType;

use core::{Pixels, Color};
use image::ImageFormat;
use image::{Pixel, Rgba, RgbaImage};
use std::fs::File;
use triangle::Triangle;
use scanline::Scanline;
use shape::Shape;

const SIZE: usize = 256;

pub fn run(config: Config) {
    println!("{:?}", config);

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
    let img = util::load_image(config.filepath.as_ref()).expect("couldn't load image");
    let img = util::scaled_to_area(img, SIZE * SIZE).to_rgba();
    println!("wtf");
    let target = Pixels::from(img);
    println!("wtf");
    let mut current = Pixels::new(target.w, target.h);
    println!("wtf");
    current.erase(&target.average_color());
    println!("wtf");
    image::save_buffer("out.png", &current.buf, current.w as u32, current.h as u32, image::ColorType::RGBA(8));
}

#[derive(Debug)]
pub struct Config {
    pub filepath: String,
    pub n: u32,
    pub t: ShapeType,
}
