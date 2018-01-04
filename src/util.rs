use image;
use image::{ImageResult, DynamicImage};
use image::imageops::FilterType;
use image::GenericImage;
use rand::Rand;
use rand::StdRng;
use rand::distributions::normal::StandardNormal;
use std::f64::consts::PI;

use core::Color;
use scanline::Scanline;

pub fn load_image(filepath: &str) -> ImageResult<DynamicImage> {
    image::open(filepath)
}

pub fn scaled_to_area(img: DynamicImage, area: usize) -> DynamicImage {
    let w = img.width();
    let h = img.height();
    if w * h < area as u32 {
        img
    } else {
        let x = ((area as f64) / (w * h) as f64).sqrt();
        img.resize((x * w as f64) as u32, (x * h as f64) as u32, FilterType::Nearest)
    }
}

pub fn rng_normal(rng: &mut StdRng) -> f64 {
    StandardNormal::rand(rng).0
}

pub fn degrees(radians: f64) -> f64 {
    radians * 180.0 / PI
}

pub fn draw_lines(buf: &mut [u8], w: usize, h: usize, a: &Color, lines: &[Scanline]) {
    let aa = a.a() as u32;
    let ar = a.r() as u32 * aa;
    let ag = a.g() as u32 * aa;
    let ab = a.b() as u32 * aa;
    for line in lines {
        for x in line.x1..(line.x2 + 1) {
            let i = 4 * (line.y * w + x);
            let ba = buf[i + 3] as u32;
            let br = buf[i] as u32 * ba;
            let bg = buf[i + 1] as u32 * ba;
            let bb = buf[i + 2] as u32 * ba;
            let diff = 255 - aa;
            buf[i] = ((ar + br * diff / 255) >> 8) as u8;
            buf[i + 1] = ((ag + bg * diff / 255) >> 8) as u8;
            buf[i + 2] = ((ab + bb * diff / 255) >> 8) as u8;
            buf[i + 3] = (aa + ba * diff / 255) as u8;
        }
    }
}

pub fn erase(buf: &mut [u8], color: &Color) {
    let mut i = 0;
    let len = buf.len();
    while i < len {
        buf[i] = color.r();
        buf[i + 1] = color.g();
        buf[i + 2] = color.b();
        buf[i + 3] = color.a();
        i += 4;
    }
}
