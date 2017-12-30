use rand::ThreadRng;
use scanline::Scanline;

#[derive(Debug)]
pub enum ShapeType {
    Triangle,
}

pub trait Shape: Clone {
    fn mutate(self, w: usize, h: usize, rng: &mut ThreadRng) -> Self;

    fn rasterize<'a>(&self, w: usize, h: usize, buf: &'a mut Vec<Scanline>) -> &'a [Scanline];
}
