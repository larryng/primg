use image::math::utils::clamp;
use rand::{Rng, ThreadRng};
use scanline::Scanline;
use shape::*;
use std::mem::swap;
use util::{degrees, rng_normal};

#[derive(Debug, Clone)]
pub struct Triangle {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
    pub x3: i32,
    pub y3: i32,
}

impl Triangle {
    pub fn create_random(w: u32, h: u32, rng: &mut ThreadRng) -> Triangle {
        let x1 = rng.gen_range(0, w as i32);
        let y1 = rng.gen_range(0, h as i32);
        let x2 = x1 + rng.gen_range(0, 31) - 15;
        let y2 = y1 + rng.gen_range(0, 31) - 15;
        let x3 = x1 + rng.gen_range(0, 31) - 15;
        let y3 = y1 + rng.gen_range(0, 31) - 15;
        Triangle { x1, y1, x2, y2, x3, y3 }.mutate(w, h, rng)
    }

    fn is_valid(&self) -> bool {
        const MIN_DEGREES: f64 = 15.0;
        let a1: f64;
        let a2: f64;
        let a3: f64;
        {
            let mut x1 = (self.x2 - self.x1) as f64;
            let mut y1 = (self.y2 - self.y1) as f64;
            let mut x2 = (self.x3 - self.x1) as f64;
            let mut y2 = (self.y3 - self.y1) as f64;
            let d1 = (x1 * x1 + y1 * y1).sqrt();
            let d2 = (x2 * x2 + y2 * y2).sqrt();
            x1 /= d1;
            y1 /= d1;
            x2 /= d2;
            y2 /= d2;
            a1 = degrees((x1 * x2 + y1 * y2).acos());
        }
        {
            let mut x1 = (self.x1 - self.x2) as f64;
            let mut y1 = (self.y1 - self.y2) as f64;
            let mut x2 = (self.x3 - self.x2) as f64;
            let mut y2 = (self.y3 - self.y2) as f64;
            let d1 = (x1 * x1 + y1 * y1).sqrt();
            let d2 = (x2 * x2 + y2 * y2).sqrt();
            x1 /= d1;
            y1 /= d1;
            x2 /= d2;
            y2 /= d2;
            a2 = degrees((x1 * x2 + y1 * y2).acos());
        }
        a3 = 180.0 - a1 - a2;
        a1 > MIN_DEGREES && a2 > MIN_DEGREES && a3 > MIN_DEGREES
    }
}

impl Shape for Triangle {
    fn mutate(mut self, w: u32, h: u32, rng: &mut ThreadRng) -> Triangle {
        let w = w as i32;
        let h = h as i32;
        let m: i32 = 16;
        let min: i32 = -m;
        let max_x: i32 = w - 1 + m;
        let max_y: i32 = h - 1 + m;
        loop {
            let dx = (rng_normal(rng) * 31 as f64) as i32;
            let dy = (rng_normal(rng) * 31 as f64) as i32;
            match rng.gen_range(0, 3) {
                0 => {
                    self.x1 = clamp(self.x1 + dx, min, max_x);
                    self.y1 = clamp(self.y1 + dy, min, max_y);
                }
                1 => {
                    self.x2 = clamp(self.x2 + dx, min, max_x);
                    self.y2 = clamp(self.y2 + dy, min, max_y);
                }
                2 => {
                    self.x3 = clamp(self.x3 + dx, min, max_x);
                    self.y3 = clamp(self.y3 + dy, min, max_y);
                }
                _ => panic!("impossible")
            }
            if self.is_valid() {
                break;
            }
        }
        self
    }

    fn rasterize<'a>(&self, w: u32, h: u32, buf: &'a mut Vec<Scanline>) -> &'a [Scanline] {
        rasterize_triangle(w, h, self.x1, self.y1, self.x2, self.y2, self.x3, self.y3, buf)
    }
}

fn rasterize_triangle<'a>(w: u32, h: u32,
                          mut x1: i32, mut y1: i32,
                          mut x2: i32, mut y2: i32,
                          mut x3: i32, mut y3: i32,
                          mut buf: &'a mut Vec<Scanline>) -> &'a [Scanline] {
    if y1 > y3 {
        swap(&mut x1, &mut x3);
        swap(&mut y1, &mut y3);
    }
    if y1 > y2 {
        swap(&mut x1, &mut x2);
        swap(&mut y1, &mut y2);
    }
    if y2 > y3 {
        swap(&mut x2, &mut x3);
        swap(&mut y2, &mut y3);
    }
    if y2 == y3 {
        let count = rasterize_triangle_bottom(w, h, x1, y1, x2, y2, x3, y3, &mut buf, 0);
        &buf[0..count]
    } else if y1 == y2 {
        let count = rasterize_triangle_top(w, h, x1, y1, x2, y2, x3, y3, &mut buf, 0);
        &buf[0..count]
    } else {
        let x4 = x1 + (((y2 - y1) as f64 / (y3 - y1) as f64) * (x3 - x1) as f64) as i32;
        let y4 = y2;
        let first = rasterize_triangle_bottom(w, h, x1, y1, x2, y2, x4, y4, &mut buf, 0);
        let last = rasterize_triangle_top(w, h, x2, y2, x4, y4, x3, y3, &mut buf, first);
        &buf[0..first + last]
    }
}

fn rasterize_triangle_bottom(w: u32, h: u32,
                             x1: i32, y1: i32,
                             x2: i32, y2: i32,
                             x3: i32, y3: i32,
                             buf: &mut Vec<Scanline>,
                             offset: usize) -> usize {
    let s1 = (x2 - x1) as f64 / (y2 - y1) as f64;
    let s2 = (x3 - x1) as f64 / (y3 - y1) as f64;
    let mut ax = x1 as f64;
    let mut bx = x1 as f64;
    let mut count = 0usize;
    let mut y = y1;
    while y < y2 + 1 {
        let mut a = ax as i32;
        let mut b = bx as i32;
        ax += s1;
        bx += s2;
        if a > b {
            swap(&mut a, &mut b)
        }
        let line = &mut buf[offset + count];
        line.y = y;
        line.x1 = a;
        line.x2 = b;
        line.alpha = 0xff;
        if line.crop(w, h) {
            count += 1;
        }
        y += 1;
    }
    return count;
}

fn rasterize_triangle_top<'a>(w: u32, h: u32,
                              x1: i32, y1: i32,
                              x2: i32, y2: i32,
                              x3: i32, y3: i32,
                              buf: &'a mut Vec<Scanline>,
                              offset: usize) -> usize {
    let s1 = (x3 - x1) as f64 / (y3 - y1) as f64;
    let s2 = (x3 - x2) as f64 / (y3 - y2) as f64;
    let mut ax = x3 as f64;
    let mut bx = x3 as f64;
    let mut count = 0usize;
    let mut y = y3;
    while y > y1 {
        ax -= s1;
        bx -= s2;
        let mut a = ax as i32;
        let mut b = bx as i32;
        if a > b {
            swap(&mut a, &mut b);
        }
        let line = &mut buf[offset + count];
        line.y = y;
        line.x1 = a;
        line.x2 = b;
        line.alpha = 0xff;
        if line.crop(w, h) {
            count += 1;
        }
        y -= 1;
    }
    return count;
}
