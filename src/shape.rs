use image::math::utils::clamp;
//use imageproc::drawing;
//use imageproc::drawing::Point;
use rand::{Rng, StdRng};
use std::mem::swap;
use std::cmp::{min, max};

use scanline::Scanline;
use util::{degrees, rng_normal, rotate, scale_dimen};

#[derive(Debug, Copy, Clone)]
pub enum ShapeType {
    Triangle,
    Ellipse,
    Rectangle,
    RotatedRectangle,
}

#[derive(Debug, Clone)]
pub enum Shape {
    Triangle { x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32 },
    Ellipse { x: i32, y: i32, rx: i32, ry: i32 },
    Rectangle { x1: i32, y1: i32, x2: i32, y2: i32 },
    RotatedRectangle { x: i32, y: i32, sx: i32, sy: i32, angle: i32 },
}

impl Shape {
    pub fn random(t: ShapeType, w: usize, h: usize, rng: &mut StdRng) -> Shape {
        let w = w as i32;
        let h = h as i32;
        match t {
            ShapeType::Triangle => random_triangle(w, h, rng),
            ShapeType::Ellipse => random_ellipse(w, h, rng),
            ShapeType::Rectangle => random_rectangle(w, h, rng),
            ShapeType::RotatedRectangle => random_rotated_rectangle(w, h, rng),
        }
    }

    pub fn mutate(&mut self, w: usize, h: usize, rng: &mut StdRng) {
        let w = w as i32;
        let h = h as i32;
        match *self {
            Shape::Triangle {
                ref mut x1, ref mut y1,
                ref mut x2, ref mut y2,
                ref mut x3, ref mut y3,
            } => mutate_triangle(w, h, rng, x1, y1, x2, y2, x3, y3),
            Shape::Ellipse {
                ref mut x, ref mut y,
                ref mut rx, ref mut ry,
            } => mutate_ellipse(w, h, rng, x, y, rx, ry),
            Shape::Rectangle {
                ref mut x1, ref mut y1,
                ref mut x2, ref mut y2,
            } => mutate_rectangle(w, h, rng, x1, y1, x2, y2),
            Shape::RotatedRectangle {
                ref mut x, ref mut y,
                ref mut sx, ref mut sy,
                ref mut angle,
            } => mutate_rotated_rectangle(w, h, rng, x, y, sx, sy, angle),
        }
    }

    pub fn rasterize<'a>(&self, w: usize, h: usize, buf: &'a mut Vec<Scanline>) -> &'a [Scanline] {
        let w = w as i32;
        let h = h as i32;
        match *self {
            Shape::Triangle { x1, y1, x2, y2, x3, y3 } => {
                rasterize_triangle(w, h, x1, y1, x2, y2, x3, y3, buf)
            }
            Shape::Ellipse { x, y, rx, ry } => {
                rasterize_ellipse(w, h, x, y, rx, ry, buf)
            }
            Shape::Rectangle { x1, y1, x2, y2 } => {
                rasterize_rectangle(x1, y1, x2, y2, buf)
            }
            Shape::RotatedRectangle { x, y, sx, sy, angle } => {
                rasterize_rotated_rectangle(w, h, x, y, sx, sy, angle, buf)
            }
        }
    }

    pub fn svg(&self, attrs: &str) -> String {
        match *self {
            Shape::Triangle { x1, y1, x2, y2, x3, y3 } => {
                format!("<polygon {} points=\"{},{} {},{} {},{}\" />",
                        attrs, x1, y1, x2, y2, x3, y3)
            }
            Shape::Ellipse { x, y, rx, ry } => {
                format!("<ellipse {} cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" />",
                        attrs, x, y, rx, ry)
            }
            Shape::Rectangle { x1, y1, x2, y2 } => {
                let w = x2 - x1 + 1;
                let h = y2 - y1 + 1;
                format!("<rect {} x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" />",
                        attrs, x1, y1, w, h)
            }
            Shape::RotatedRectangle { x, y, sx, sy, angle } => {
                format!("<g transform=\"translate({} {}) rotate({}) scale({} {})\"><rect {} x=\"-0.5\" y=\"-0.5\" width=\"1\" height=\"1\" /></g>",
                        x, y, angle, sx, sy, attrs)
            }
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
            }
            Shape::Ellipse { x, y, rx, ry } => {
                Shape::Ellipse {
                    x: scale_dimen(x, scale),
                    y: scale_dimen(y, scale),
                    rx: scale_dimen(rx, scale),
                    ry: scale_dimen(ry, scale),
                }
            }
            Shape::Rectangle { x1, y1, x2, y2 } => {
                Shape::Rectangle {
                    x1: scale_dimen(x1, scale),
                    y1: scale_dimen(y1, scale),
                    x2: scale_dimen(x2, scale),
                    y2: scale_dimen(y2, scale),
                }
            }
            Shape::RotatedRectangle { x, y, sx, sy, angle } => {
                Shape::RotatedRectangle {
                    x: scale_dimen(x, scale),
                    y: scale_dimen(y, scale),
                    sx: scale_dimen(sx, scale),
                    sy: scale_dimen(sy, scale),
                    angle,
                }
            }
        }
    }

    #[cfg(target_os = "android")]
    pub fn serialize(&self) -> String {
        match *self {
            Shape::Triangle { x1, y1, x2, y2, x3, y3 } => {
                format!("0:{},{},{},{},{},{}", x1, y1, x2, y2, x3, y3)
            }
            Shape::Ellipse { x, y, rx, ry } => {
                format!("1:{},{},{},{}", x, y, rx, ry)
            }
            Shape::Rectangle { x1, y1, x2, y2 } => {
                format!("2:{},{},{},{}", x1, y1, x2, y2)
            }
            Shape::RotatedRectangle { x, y, sx, sy, angle } => {
                format!("3:{},{},{},{},{}", x, y, sx, sy, angle)
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

fn random_triangle(w: i32, h: i32, rng: &mut StdRng) -> Shape {
    let mut x1 = rng.gen_range(0, w);
    let mut y1 = rng.gen_range(0, h);
    let mut x2 = x1 + rng.gen_range(0, 31) - 15;
    let mut y2 = y1 + rng.gen_range(0, 31) - 15;
    let mut x3 = x1 + rng.gen_range(0, 31) - 15;
    let mut y3 = y1 + rng.gen_range(0, 31) - 15;
    mutate_triangle(w, h, rng, &mut x1, &mut y1, &mut x2, &mut y2, &mut x3, &mut y3);
    Shape::Triangle { x1, y1, x2, y2, x3, y3 }
}

fn random_ellipse(w: i32, h: i32, rng: &mut StdRng) -> Shape {
    let x = rng.gen_range(0, w);
    let y = rng.gen_range(0, h);
    let rx = rng.gen_range(0, 32) + 1;
    let ry = rng.gen_range(0, 32) + 1;
    Shape::Ellipse { x, y, rx, ry }
}

fn random_rectangle(w: i32, h: i32, rng: &mut StdRng) -> Shape {
    let x1 = rng.gen_range(0, w);
    let y1 = rng.gen_range(0, h);
    let x2 = clamp(x1 + rng.gen_range(0, 32) + 1, 0, w - 1);
    let y2 = clamp(y1 + rng.gen_range(0, 32) + 1, 0, h - 1);
    Shape::Rectangle { x1, y1, x2, y2 }
}

fn random_rotated_rectangle(w: i32, h: i32, rng: &mut StdRng) -> Shape {
    let x = rng.gen_range(0, w);
    let y = rng.gen_range(0, h);
    let sx = rng.gen_range(0, 32) + 1;
    let sy = rng.gen_range(0, 32) + 1;
    let angle = rng.gen_range(0, 360);
    Shape::RotatedRectangle { x, y, sx, sy, angle }
}

fn mutate_triangle(w: i32, h: i32, rng: &mut StdRng,
                   x1: &mut i32, y1: &mut i32,
                   x2: &mut i32, y2: &mut i32,
                   x3: &mut i32, y3: &mut i32) {
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

fn mutate_ellipse(w: i32, h: i32, rng: &mut StdRng,
                  x: &mut i32, y: &mut i32,
                  rx: &mut i32, ry: &mut i32) {
    match rng.gen_range(0, 3) {
        0 => {
            *x = clamp(*x + (rng_normal(rng) * 16.0) as i32, 0, w - 1);
            *y = clamp(*y + (rng_normal(rng) * 16.0) as i32, 0, h - 1);
        }
        1 => {
            *rx = clamp(*rx + (rng_normal(rng) * 16.0) as i32, 0, w - 1);
        }
        _ => {
            *ry = clamp(*ry + (rng_normal(rng) * 16.0) as i32, 0, h - 1);
        }
    }
}

fn mutate_rectangle(w: i32, h: i32, rng: &mut StdRng,
                    x1: &mut i32, y1: &mut i32,
                    x2: &mut i32, y2: &mut i32) {
    match rng.gen_range(0, 2) {
        0 => {
            *x1 = clamp(*x1 + (rng_normal(rng) * 16.0) as i32, 0, w - 1);
            *y1 = clamp(*y1 + (rng_normal(rng) * 16.0) as i32, 0, h - 1);
        }
        _ => {
            *x2 = clamp(*x1 + (rng_normal(rng) * 16.0) as i32, 0, w - 1);
            *y2 = clamp(*y1 + (rng_normal(rng) * 16.0) as i32, 0, h - 1);
        }
    }
    if *x1 > *x2 {
        swap(x1, x2);
    }
    if *y1 > *y2 {
        swap(y1, y2);
    }
}

fn mutate_rotated_rectangle(w: i32, h: i32, rng: &mut StdRng,
                            x: &mut i32, y: &mut i32,
                            sx: &mut i32, sy: &mut i32,
                            angle: &mut i32) {
    match rng.gen_range(0, 3) {
        0 => {
            *x = clamp(*x + (rng_normal(rng) * 16.0) as i32, 0, w - 1);
            *y = clamp(*y + (rng_normal(rng) * 16.0) as i32, 0, h - 1);
        }
        1 => {
            *sx = clamp(*sx + (rng_normal(rng) * 16.0) as i32, 1, w - 1);
            *sy = clamp(*sy + (rng_normal(rng) * 16.0) as i32, 1, h - 1);
        }
        _ => {
            *angle = *angle + (rng_normal(rng) * 32.0) as i32;
        }
    }
}

fn rasterize_triangle<'a>(w: i32, h: i32,
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

fn rasterize_triangle_bottom(w: i32, h: i32,
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

fn rasterize_triangle_top<'a>(w: i32, h: i32,
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

fn rasterize_ellipse<'a>(w: i32, h: i32,
                         x: i32, y: i32,
                         rx: i32, ry: i32,
                         buf: &'a mut Vec<Scanline>) -> &'a [Scanline] {
    let aspect = rx as f32 / ry as f32;
    let mut count = 0;
    for dy in 0..ry {
        let y1 = y - dy;
        let y2 = y + dy;
        if (y1 < 0 || y1 > h) && (y2 < 0 || y2 >= h) {
            continue;
        }
        let s = (((ry * ry - dy * dy) as f32).sqrt() * aspect) as i32;
        let mut x1 = x - s;
        let mut x2 = x + s;
        if x1 < 0 {
            x1 = 0;
        }
        if x2 >= w {
            x2 = w - 1;
        }
        if y1 >= 0 && y1 < h {
            let line = &mut buf[count];
            line.y = y1 as usize;
            line.x1 = x1 as usize;
            line.x2 = x2 as usize;
            count += 1;
        }
        if y2 >= 0 && y2 < h && dy > 0 {
            let line = &mut buf[count];
            line.y = y2 as usize;
            line.x1 = x1 as usize;
            line.x2 = x2 as usize;
            count += 1;
        }
    }
    &buf[0..count]
}

fn rasterize_rectangle<'a>(x1: i32, y1: i32,
                           x2: i32, y2: i32,
                           buf: &'a mut Vec<Scanline>) -> &'a [Scanline] {
    for y in y1..y2 + 1 {
        let line = &mut buf[(y - y1) as usize];
        line.y = y as usize;
        line.x1 = x1 as usize;
        line.x2 = x2 as usize;
    }
    &buf[0..(y2 - y1) as usize]
}

fn rasterize_rotated_rectangle<'a>(w: i32, h: i32,
                                   x: i32, y: i32,
                                   sx: i32, sy: i32,
                                   angle: i32,
                                   buf: &'a mut Vec<Scanline>) -> &'a [Scanline] {
    let sx = sx as f32;
    let sy = sy as f32;
    let angle = (angle as f32).to_radians();
    let (rx1, ry1) = rotate(-sx / 2.0, -sy / 2.0, angle);
    let (rx2, ry2) = rotate(sx / 2.0, -sy / 2.0, angle);
    let (rx3, ry3) = rotate(sx / 2.0, sy / 2.0, angle);
    let (rx4, ry4) = rotate(-sx / 2.0, sy / 2.0, angle);
    let (x1, y1) = ((rx1 as i32) + x, (ry1 as i32) + y);
    let (x2, y2) = ((rx2 as i32) + x, (ry2 as i32) + y);
    let (x3, y3) = ((rx3 as i32) + x, (ry3 as i32) + y);
    let (x4, y4) = ((rx4 as i32) + x, (ry4 as i32) + y);
    let miny = min(y1, min(y2, min(y3, y4)));
    let maxy = max(y1, max(y2, max(y3, y4)));
    let n = maxy - miny + 1;
    let mut mins = Vec::with_capacity(n as usize);
    let mut maxs = Vec::with_capacity(n as usize);
    for i in 0..n {
        mins.push(w);
        maxs.push(0);
    }
    let xs = &[x1, x2, x3, x4, x1];
    let ys = &[y1, y2, y3, y4, y1];
    for i in 0..4 {
        let (x, y) = (xs[i] as f32, ys[i] as f32);
        let (dx, dy) = ((xs[i + 1] - xs[i]) as f32, (ys[i + 1] - ys[i]) as f32);
        let count = ((dx * dx + dy * dy).sqrt() as i32) * 2;
        for j in 0..count {
            let t = j as f32 / (count - 1) as f32;
            let xi = (x + dx * t) as i32;
            let yi = (y + dy * t) as i32 - miny;
            mins[yi as usize] = min(mins[yi as usize], xi);
            maxs[yi as usize] = max(maxs[yi as usize], xi);
        }
    }
    let mut count = 0;
    for i in 0..n {
        let y = miny + i;
        if y < 0 || y >= h {
            continue;
        }
        let a = max(mins[i as usize], 0);
        let b = min(maxs[i as usize], w - 1);
        if b >= a {
            let line = &mut buf[count];
            line.y = y as usize;
            line.x1 = a as usize;
            line.x2 = b as usize;
            count += 1;
        }
    }
    &buf[0..count as usize]
}
