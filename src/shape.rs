use rand::ThreadRng;
use scanline::Scanline;

#[derive(Debug)]
pub enum ShapeType {
    Triangle,
}

pub trait Shape: Clone {
    fn mutate(self, w: u32, h: u32, rng: &mut ThreadRng) -> Self;

    fn rasterize<'a>(&self, w: u32, h: u32, buf: &'a mut Vec<Scanline>) -> &'a [Scanline];
}
