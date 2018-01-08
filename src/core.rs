use super::SIZE;
use image::{Pixel, Rgba, RgbaImage};
use image::math::utils::clamp;
use std::fmt;
use std::marker::Sync;

use scanline::Scanline;
use util;

#[derive(Clone)]
pub struct Pixels {
    pub buf: [u8; Pixels::BUF_SIZE],
    pub w: usize,
    pub h: usize,
}

unsafe impl Sync for Pixels {}

impl Pixels {
    pub const BUF_SIZE: usize = SIZE * SIZE * 4;

    pub fn new(w: usize, h: usize) -> Pixels {
        Pixels { buf: [0; Pixels::BUF_SIZE], w, h }
    }

    pub fn from(img: RgbaImage) -> Pixels {
        let w = img.width() as usize;
        let h = img.height() as usize;
        assert!(w * h < SIZE * SIZE);
        let mut img: Vec<u8> = img.into_raw();
        img.resize(Pixels::BUF_SIZE, 0);
        let mut buf = [0; Pixels::BUF_SIZE];
        buf.copy_from_slice(img.as_ref());
        Pixels { buf, w, h }
    }

    pub fn get(&self, x: usize, y: usize) -> Color {
        let i = self.index(x, y);
        Color::from(&self.buf[i..i + 4])
    }

    pub fn get_arr(&self, x: usize, y: usize) -> &[u8] {
        let i = self.index(x, y);
        &self.buf[i..i + 4]
    }

    pub fn put(&mut self, x: usize, y: usize, color: &Color) {
        let i = self.index(x, y);
        self.buf[i] = color.r();
        self.buf[i + 1] = color.g();
        self.buf[i + 2] = color.b();
        self.buf[i + 3] = color.a();
    }

    pub fn erase(&mut self, color: &Color) {
        util::erase(&mut self.buf, color);
    }

    pub fn average_color(&self) -> Color {
        let mut r = 0u64;
        let mut g = 0u64;
        let mut b = 0u64;
        for y in 0..self.h {
            for x in 0..self.w {
                let c = self.get(x, y);
                r += c.r() as u64;
                g += c.g() as u64;
                b += c.b() as u64;
            }
        }
        let area = self.w as u64 * self.h as u64;
        r /= area;
        g /= area;
        b /= area;
        Color::new(r as u8, g as u8, b as u8, 255)
    }

    pub fn compute_color(&self, target: &Pixels, lines: &[Scanline], alpha: u8) -> Color {
        let mut rsum = 0;
        let mut gsum = 0;
        let mut bsum = 0;
        let mut count = 0;
        let a = 0xffff / (alpha as i32);
        unsafe {
            for line in lines {
                let mut i = target.index(line.x1, line.y);
                for _ in line.x1..line.x2 + 1 {
                    let tr = *target.buf.get_unchecked(i + 0) as i32;
                    let tg = *target.buf.get_unchecked(i + 1) as i32;
                    let tb = *target.buf.get_unchecked(i + 2) as i32;
                    let cr = *self.buf.get_unchecked(i + 0) as i32;
                    let cg = *self.buf.get_unchecked(i + 1) as i32;
                    let cb = *self.buf.get_unchecked(i + 2) as i32;
//                let t = target.get_arr(x, line.y);
//                let tr = t[0] as i32;
//                let tg = t[1] as i32;
//                let tb = t[2] as i32;
//                let c = self.get_arr(x, line.y);
//                let cr = c[0] as i32;
//                let cg = c[1] as i32;
//                let cb = c[2] as i32;

//                let t = target.get(x, line.y);
//                let tr = t.r() as i32;
//                let tg = t.g() as i32;
//                let tb = t.b() as i32;
//                let c = self.get(x, line.y);
//                let cr = c.r() as i32;
//                let cg = c.g() as i32;
//                let cb = c.b() as i32;
                    rsum += (tr - cr) * a + cr * 0x101;
                    gsum += (tg - cg) * a + cg * 0x101;
                    bsum += (tb - cb) * a + cb * 0x101;
                    count += 1;
                    i += 4;
                }
            }
        }
        if count == 0 {
            return Color::new(0, 0, 0, 0);
        }
        let r = clamp((rsum / count) >> 8, 0, 255);
        let g = clamp((gsum / count) >> 8, 0, 255);
        let b = clamp((bsum / count) >> 8, 0, 255);
        return Color::new(r as u8, g as u8, b as u8, alpha);
    }

    pub fn copy_lines(&mut self, src: &Pixels, lines: &[Scanline]) {
        for line in lines {
            let a = self.index(line.x1, line.y);
            let b = a + (line.x2 - line.x1 + 1) * 4;
            let dst = &mut self.buf[a..b];
            dst.copy_from_slice(&src.buf[a..b])
        }
    }

    pub fn draw_lines(&mut self, a: &Color, lines: &[Scanline]) {
        util::draw_lines(&mut self.buf, self.w, self.h, a, lines);
    }

    pub fn difference_full(a: &Pixels, b: &Pixels) -> f32 {
        let w = a.w;
        let h = a.h;
        let mut total = 0i32;
        for y in 0..h {
            for x in 0..w {
                let pa = a.get_arr(x, y);
                let pb = b.get_arr(x, y);

                let dr = pa[0] as i32 - pb[0] as i32;
                let dg = pa[1] as i32 - pb[1] as i32;
                let db = pa[2] as i32 - pb[2] as i32;
                let da = pa[3] as i32 - pb[3] as i32;
                total += (dr * dr) + (dg * dg) + (db * db) + (da * da);
            }
        }
        (total as f32 / (w * h * 4) as f32).sqrt() / 255.0
    }

    pub fn difference_partial(target: &Pixels,
                              before: &Pixels,
                              after: &Pixels,
                              score: f32,
                              lines: &[Scanline]) -> f32 {
        let ni = target.w * target.h * 4;
        let mut total = ((score * 255.0).powi(2) * ni as f32) as i32;

        unsafe {
            for line in lines {
                let mut i = target.index(line.x1, line.y);
                for _ in line.x1..line.x2 + 1 {
                    let dr1 = *target.buf.get_unchecked(i + 0) as i32 - *before.buf.get_unchecked(i + 0) as i32;
                    let dg1 = *target.buf.get_unchecked(i + 1) as i32 - *before.buf.get_unchecked(i + 1) as i32;
                    let db1 = *target.buf.get_unchecked(i + 2) as i32 - *before.buf.get_unchecked(i + 2) as i32;
                    let da1 = *target.buf.get_unchecked(i + 3) as i32 - *before.buf.get_unchecked(i + 3) as i32;

                    let dr2 = *target.buf.get_unchecked(i + 0) as i32 - *after.buf.get_unchecked(i + 0) as i32;
                    let dg2 = *target.buf.get_unchecked(i + 1) as i32 - *after.buf.get_unchecked(i + 1) as i32;
                    let db2 = *target.buf.get_unchecked(i + 2) as i32 - *after.buf.get_unchecked(i + 2) as i32;
                    let da2 = *target.buf.get_unchecked(i + 3) as i32 - *after.buf.get_unchecked(i + 3) as i32;

                    total -= (dr1 * dr1) + (dg1 * dg1) + (db1 * db1) + (da1 * da1);
                    total += (dr2 * dr2) + (dg2 * dg2) + (db2 * db2) + (da2 * da2);

                    i += 4;
                }
            }
        }
        (total as f32 / ni as f32).sqrt() / 255.0
    }

    pub fn index(&self, x: usize, y: usize) -> usize {
        4 * (y * self.w + x)
    }
}

#[derive(Copy, Clone)]
pub struct Color(u32);

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color(((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | a as u32)
    }

    pub fn from(c: &[u8]) -> Color {
        Color::new(c[0], c[1], c[2], c[3])
    }

    pub fn to_rgba(&self) -> Rgba<u8> {
        Pixel::from_channels(self.r(), self.g(), self.b(), self.a())
    }

    #[cfg(target_os = "android")]
    pub fn to_argb_i32(&self) -> i32 {
        let a = self.0 & 0xff;
        ((a << 24) | (self.0 >> 8)) as i32
    }

    pub fn r(&self) -> u8 {
        (self.0 >> 24) as u8
    }

    pub fn g(&self) -> u8 {
        ((self.0 >> 16) & 0xff) as u8
    }

    pub fn b(&self) -> u8 {
        ((self.0 >> 8) & 0xff) as u8
    }

    pub fn a(&self) -> u8 {
        (self.0 & 0xff) as u8
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
