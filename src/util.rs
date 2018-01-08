use image;
use image::{ImageResult, DynamicImage};
use image::imageops::FilterType;
use image::GenericImage;
use rand::Rand;
use rand::StdRng;
use rand::distributions::normal::StandardNormal;
use std::f32::consts::PI;

use core::Color;
use scanline::Scanline;

pub fn load_image(filepath: &str) -> ImageResult<DynamicImage> {
    image::open(filepath)
}

pub fn scale_dimen(a: i32, scale: f32) -> i32 {
    (a as f32 * scale).round() as i32
}

pub fn scaled_to_area(img: DynamicImage, area: usize) -> DynamicImage {
    let w = img.width();
    let h = img.height();
    if w * h < area as u32 {
        img
    } else {
        let x = ((area as f32) / (w * h) as f32).sqrt();
        img.resize((x * w as f32) as u32, (x * h as f32) as u32, FilterType::Nearest)
    }
}

pub fn rng_normal(rng: &mut StdRng) -> f32 {
    StandardNormal::rand(rng).0 as f32
}

pub fn degrees(radians: f32) -> f32 {
    radians * 180.0 / PI
}

pub fn draw_lines(buf: &mut [u8], w: usize, h: usize, a: &Color, lines: &[Scanline]) {
    let aa = a.a() as u32;
    let ar = a.r() as u32 * aa;
    let ag = a.g() as u32 * aa;
    let ab = a.b() as u32 * aa;
    unsafe {
        for line in lines {
            let mut i = 4 * (line.y * w + line.x1) as isize;
            for _ in line.x1..(line.x2 + 1) {
                let p = buf.as_mut_ptr();
                let p0 = p.offset(i + 0);
                let p1 = p.offset(i + 1);
                let p2 = p.offset(i + 2);
                let p3 = p.offset(i + 3);

                let ba = *p3 as u32;
                let br = *p0 as u32 * ba;
                let bg = *p1 as u32 * ba;
                let bb = *p2 as u32 * ba;
                let diff = 255 - aa;
                *p0 = ((ar + br * diff / 255) >> 8) as u8;
                *p1 = ((ag + bg * diff / 255) >> 8) as u8;
                *p2 = ((ab + bb * diff / 255) >> 8) as u8;
                *p3 = (aa + ba * diff / 255) as u8;

                i += 4;
            }
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
