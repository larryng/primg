use image::RgbaImage;
use rand;
use rand::ThreadRng;
use scanline::Scanline;

pub struct Worker<'a> {
    pub w: u32,
    pub h: u32,
    pub target: &'a RgbaImage,
    pub current: RgbaImage,
    pub buffer: RgbaImage,
    pub rng: ThreadRng,
    pub scanlines: Vec<Scanline>,
}

impl<'a> Worker<'a> {
    pub fn new(w: u32, h: u32, target: &RgbaImage) -> Worker {
        let rng = rand::thread_rng();
        let scanlines = (0..h).map(|_| Scanline::empty()).collect();
        let current = RgbaImage::new(w, h);
        let buffer = RgbaImage::new(w, h);
        Worker { w, h, target, current, buffer, rng, scanlines }
    }
}
