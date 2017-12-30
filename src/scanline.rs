use image::math::utils::clamp;

#[derive(Debug)]
pub struct Scanline {
    pub y: i32,
    pub x1: i32,
    pub x2: i32,
}

impl Scanline {
    pub fn empty() -> Scanline {
        Scanline { y: 0, x1: 0, x2: 0 }
    }

    pub fn crop(&mut self, w: usize, h: usize) -> bool {
        let w = w as i32;
        let h = h as i32;
        if (self.y < 0 || self.y >= h) || self.x1 >= w || self.x2 < 0 {
            return false;
        } else {
            self.x1 = clamp(self.x1, 0, w - 1);
            self.x2 = clamp(self.x2, 0, w - 1);
            if self.x1 > self.x2 {
                false;
            }
        }
        true
    }
}

