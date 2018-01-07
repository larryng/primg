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
        let mut rsum = 0i64;
        let mut gsum = 0i64;
        let mut bsum = 0i64;
        let mut count = 0i64;
        let a = 0xffff / alpha as i64;
        for line in lines {
            for x in line.x1..line.x2 + 1 {
                let t = target.get(x, line.y);
                let tr = t.r() as i64;
                let tg = t.g() as i64;
                let tb = t.b() as i64;
                let c = self.get(x, line.y);
                let cr = c.r() as i64;
                let cg = c.g() as i64;
                let cb = c.b() as i64;
                rsum += (tr - cr) * a + cr * 0x101;
                gsum += (tg - cg) * a + cg * 0x101;
                bsum += (tb - cb) * a + cb * 0x101;
                count += 1;
            }
        }
        if count == 0 {
            return Color::new(0, 0, 0, 0);
        }
        let r = clamp(rsum / count >> 8, 0, 255);
        let g = clamp(gsum / count >> 8, 0, 255);
        let b = clamp(bsum / count >> 8, 0, 255);
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
        let mut total = 0i64;
        for y in 0..h {
            for x in 0..w {
                let pa = a.get(x, y);
                let pb = b.get(x, y);

                let dr = pa.r() as i64 - pb.r() as i64;
                let dg = pa.g() as i64 - pb.g() as i64;
                let db = pa.b() as i64 - pb.b() as i64;
                let da = pa.a() as i64 - pb.a() as i64;
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
        let w = target.w;
        let h = target.h;
        let mut total = ((score * 255.0).powi(2) * (w * h * 4) as f32) as i64;
        for line in lines {
            for x in line.x1..line.x2 + 1 {
                let pt = target.get(x, line.y);
                let pb = before.get(x, line.y);
                let pa = after.get(x, line.y);

                let dr1 = pt.r() as i64 - pb.r() as i64;
                let dg1 = pt.g() as i64 - pb.g() as i64;
                let db1 = pt.b() as i64 - pb.b() as i64;
                let da1 = pt.a() as i64 - pb.a() as i64;

                let dr2 = pt.r() as i64 - pa.r() as i64;
                let dg2 = pt.g() as i64 - pa.g() as i64;
                let db2 = pt.b() as i64 - pa.b() as i64;
                let da2 = pt.a() as i64 - pa.a() as i64;

                total -= ((dr1 * dr1) + (dg1 * dg1) + (db1 * db1) + (da1 * da1)) as i64;
                total += ((dr2 * dr2) + (dg2 * dg2) + (db2 * db2) + (da2 * da2)) as i64;
            }
        }
        (total as f32 / (w * h * 4) as f32).sqrt() / 255.0
    }

    fn index(&self, x: usize, y: usize) -> usize {
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
        Pixel::from_channels(self.r(), self.g(), self.b(), self.a() )
    }

    #[cfg(target_os="android")]
    pub fn to_argb_i32(&self) -> i32 {
        let a = self.0 & 0xff;
        ((a << 24) | (self.0 >> 8)) as i32
    }

    #[inline(always)]
    pub fn r(&self) -> u8 {
        (self.0 >> 24) as u8
    }

    #[inline(always)]
    pub fn g(&self) -> u8 {
        ((self.0 >> 16) & 0xff) as u8
    }

    #[inline(always)]
    pub fn b(&self) -> u8 {
        ((self.0 >> 8) & 0xff) as u8
    }

    #[inline(always)]
    pub fn a(&self) -> u8 {
        (self.0 & 0xff) as u8
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
