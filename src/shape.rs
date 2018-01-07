use image::math::utils::clamp;
//use imageproc::drawing;
//use imageproc::drawing::Point;
use rand::{Rng, StdRng};
use std::mem::swap;

use scanline::Scanline;
use util::{degrees, rng_normal, scale_dimen};

#[derive(Debug, Copy, Clone)]
pub enum ShapeType {
    Triangle,
}

#[derive(Debug, Clone)]
pub enum Shape {
    Triangle { x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32 },
}

impl Shape {
    pub fn random(t: ShapeType, w: usize, h: usize, rng: &mut StdRng) -> Shape {
        match t {
            ShapeType::Triangle => random_triangle(w, h, rng)
        }
    }

    pub fn mutate(&mut self, w: usize, h: usize, rng: &mut StdRng) {
        match *self {
            Shape::Triangle {
                ref mut x1, ref mut y1,
                ref mut x2, ref mut y2,
                ref mut x3, ref mut y3,
            } => mutate_triangle(w, h, rng, x1, y1, x2, y2, x3, y3),
        }
    }

    pub fn rasterize<'a>(&self, w: usize, h: usize, buf: &'a mut Vec<Scanline>) -> &'a [Scanline] {
        match *self {
            Shape::Triangle { x1, y1, x2, y2, x3, y3 } => {
                rasterize_triangle(w, h, x1, y1, x2, y2, x3, y3, buf)
            },
        }
    }

    pub fn svg(&self, attrs: &str) -> String {
        match *self {
            Shape::Triangle { x1, y1, x2, y2, x3, y3 } => {
                format!("<polygon {} points=\"{},{} {},{} {},{}\" />",
                        attrs, x1, y1, x2, y2, x3, y3)
            },
        }
    }

    pub fn scaled(&self, scale: f32) -> Shape {
        match *self {
            Shape::Triangle { x1, y1, x2, y2, x3, y3 } => {
                Shape::Triangle {
                    x1: scale_dimen(x1, scale),
                    y1: scale_dimen(y1, scale),
                    x2: scale_dimen(x2, scale),
                    y2: scale_dimen(y2, scale),
                    x3: scale_dimen(x3, scale),
                    y3: scale_dimen(y3, scale),
                }
            },
        }
    }

    pub fn serialize(&self) -> String {
        match *self {
            Shape::Triangle { x1, y1, x2, y2, x3, y3 } => {
                format!("0:{},{},{},{},{},{}", x1, y1, x2, y2, x3, y3)
            }
        }
    }

//    pub fn draw(&self, img: &mut RgbaImage, color: &Color, scale: f32, buf: &mut Vec<Scanline>) {
//        match *self {
//            Shape::Triangle { x1, y1, x2, y2, x3, y3 } => {
//                let poly = &[
//                    Point::new((x1 as f32 * scale) as i32, (y1 as f32 * scale) as i32),
//                    Point::new((x2 as f32 * scale) as i32, (y2 as f32 * scale) as i32),
//                    Point::new((x3 as f32 * scale) as i32, (y3 as f32 * scale) as i32),
//                ];
//                let color = color.to_rgba();
//                println!("drawing {:?}, {:?}", self, color);
//                drawing::draw_convex_polygon_mut(img, poly, color);
//            },
//        }
//    }
}

fn random_triangle(w: usize, h: usize, rng: &mut StdRng) -> Shape {
    let mut x1 = rng.gen_range(0, w as i32);
    let mut y1 = rng.gen_range(0, h as i32);
    let mut x2 = x1 + rng.gen_range(0, 31) - 15;
    let mut y2 = y1 + rng.gen_range(0, 31) - 15;
    let mut x3 = x1 + rng.gen_range(0, 31) - 15;
    let mut y3 = y1 + rng.gen_range(0, 31) - 15;
    mutate_triangle(w, h, rng, &mut x1, &mut y1, &mut x2, &mut y2, &mut x3, &mut y3);
    Shape::Triangle { x1, y1, x2, y2, x3, y3 }
}

fn mutate_triangle(w: usize, h: usize, rng: &mut StdRng,
                   x1: &mut i32, y1: &mut i32,
                   x2: &mut i32, y2: &mut i32,
                   x3: &mut i32, y3: &mut i32) {
    let w = w as i32;
    let h = h as i32;
    let m: i32 = 16;
    let min: i32 = -m;
    let max_x: i32 = w - 1 + m;
    let max_y: i32 = h - 1 + m;
    loop {
        let dx = (rng_normal(rng) * 31.0) as i32;
        let dy = (rng_normal(rng) * 31.0) as i32;
        match rng.gen_range(0, 3) {
            0 => {
                *x1 = clamp(*x1 + dx, min, max_x);
                *y1 = clamp(*y1 + dy, min, max_y);
            }
            1 => {
                *x2 = clamp(*x2 + dx, min, max_x);
                *y2 = clamp(*y2 + dy, min, max_y);
            }
            2 => {
                *x3 = clamp(*x3 + dx, min, max_x);
                *y3 = clamp(*y3 + dy, min, max_y);
            }
            _ => panic!("impossible")
        }
        if is_valid_triangle(x1, y1, x2, y2, x3, y3) {
            break;
        }
    }
}

fn is_valid_triangle(tx1: &i32, ty1: &i32, tx2: &i32, ty2: &i32, tx3: &i32, ty3: &i32) -> bool {
    const MIN_DEGREES: f32 = 15.0;
    let a1: f32;
    let a2: f32;
    let a3: f32;
    {
        let mut x1 = (*tx2 - *tx1) as f32;
        let mut y1 = (*ty2 - *ty1) as f32;
        let mut x2 = (*tx3 - *tx1) as f32;
        let mut y2 = (*ty3 - *ty1) as f32;
        let d1 = (x1 * x1 + y1 * y1).sqrt();
        let d2 = (x2 * x2 + y2 * y2).sqrt();
        x1 /= d1;
        y1 /= d1;
        x2 /= d2;
        y2 /= d2;
        a1 = degrees((x1 * x2 + y1 * y2).acos());
    }
    {
        let mut x1 = (*tx1 - *tx2) as f32;
        let mut y1 = (*ty1 - *ty2) as f32;
        let mut x2 = (*tx3 - *tx2) as f32;
        let mut y2 = (*ty3 - *ty2) as f32;
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

fn rasterize_triangle<'a>(w: usize, h: usize,
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
        let x4 = x1 + (((y2 - y1) as f32 / (y3 - y1) as f32) * (x3 - x1) as f32) as i32;
        let y4 = y2;
        let first = rasterize_triangle_bottom(w, h, x1, y1, x2, y2, x4, y4, &mut buf, 0);
        let last = rasterize_triangle_top(w, h, x2, y2, x4, y4, x3, y3, &mut buf, first);
        &buf[0..first + last]
    }
}

fn rasterize_triangle_bottom(w: usize, h: usize,
                             x1: i32, y1: i32,
                             x2: i32, y2: i32,
                             x3: i32, y3: i32,
                             buf: &mut Vec<Scanline>,
                             offset: usize) -> usize {
    let s1 = (x2 - x1) as f32 / (y2 - y1) as f32;
    let s2 = (x3 - x1) as f32 / (y3 - y1) as f32;
    let mut ax = x1 as f32;
    let mut bx = x1 as f32;
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
        if line.validating_set(w, h, y, a, b) {
            count += 1;
        }
        y += 1;
    }
    return count;
}

fn rasterize_triangle_top<'a>(w: usize, h: usize,
                              x1: i32, y1: i32,
                              x2: i32, y2: i32,
                              x3: i32, y3: i32,
                              buf: &'a mut Vec<Scanline>,
                              offset: usize) -> usize {
    let s1 = (x3 - x1) as f32 / (y3 - y1) as f32;
    let s2 = (x3 - x2) as f32 / (y3 - y2) as f32;
    let mut ax = x3 as f32;
    let mut bx = x3 as f32;
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
        if line.validating_set(w, h, y, a, b) {
            count += 1;
        }
        y -= 1;
    }
    return count;
}
