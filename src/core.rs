use super::SIZE;
use image::RgbaImage;
use image::math::utils::clamp;
use scanline::Scanline;

pub struct Pixels {
    pub buf: [u8; Pixels::BUF_SIZE],
    pub w: usize,
    pub h: usize,
}

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
        let mut i = 0;
        let len = Pixels::BUF_SIZE;
        while i < len {
            self.buf[i] = color.r();
            self.buf[i + 1] = color.g();
            self.buf[i + 2] = color.b();
            self.buf[i + 3] = color.a();
            i += 4;
        }
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
        let mut rsum = 0u64;
        let mut gsum = 0u64;
        let mut bsum = 0u64;
        let mut count = 0u64;
        let a = 0xffff / alpha as u64;
        for line in lines {
            for x in line.x1..line.x2 + 1 {
                let x = x as usize;
                let y = line.y as usize;
                let t = target.get(x, y);
                let c = self.get(x, y);
                rsum += (t.r() - c.r()) as u64 * a + c.r() as u64 * 0x101;
                gsum += (t.g() - c.g()) as u64 * a + c.g() as u64 * 0x101;
                bsum += (t.b() - c.b()) as u64 * a + c.b() as u64 * 0x101;
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

    pub fn draw_lines(&mut self, a: &Color, lines: &[Scanline]) {
        let aa = a.a() as u32;
        let ar = a.r() as u32 * aa;
        let ag = a.g() as u32 * aa;
        let ab = a.b() as u32 * aa;
        for line in lines {
            for x in line.x1..(line.x2 + 1) {
                let x = x as usize;
                let y = line.y as usize;
                let b = self.get(x, y);
                let ba = b.a() as u32;
                let br = b.r() as u32 * ba;
                let bg = b.g() as u32 * ba;
                let bb = b.b() as u32 * ba;
                let diff = 255 - aa;
                let c = Color::new(
                    ((ar + br * diff / 255) >> 8) as u8,
                    ((ag + bg * diff / 255) >> 8) as u8,
                    ((ab + bb * diff / 255) >> 8) as u8,
                    (aa + ba * diff / 255) as u8,
                );
                self.put(x, y, &c);
            }
        }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        4 * (y * self.w + x)
    }
}

pub struct Color(u32);

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color(((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | a as u32)
    }

    pub fn from(c: &[u8]) -> Color {
        Color::new(c[0], c[1], c[2], c[3])
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

