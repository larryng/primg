use image;
use image::{ImageResult, DynamicImage};
use image::imageops::FilterType;
use image::GenericImage;
use rand::Rand;
use rand::ThreadRng;
use rand::distributions::normal::StandardNormal;
use std::f64::consts::PI;

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

pub fn rng_normal(rng: &mut ThreadRng) -> f64 {
    StandardNormal::rand(rng).0
}

pub fn degrees(radians: f64) -> f64 {
    radians * 180.0 / PI
}

