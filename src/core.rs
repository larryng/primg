use super::SIZE;
use image::RgbaImage;
use scanline::Scanline;

pub struct Pixels {
    pub buf: [u8; SIZE * SIZE],
    pub w: usize,
    pub h: usize,
}

impl Pixels {
    pub fn new(w: usize, h: usize) -> Pixels {
        Pixels { buf: [0; SIZE * SIZE], w, h }
    }

    pub fn from(img: RgbaImage) -> Pixels {
        let w = img.width() as usize;
        let h = img.height() as usize;
        assert!(w * h < SIZE * SIZE);
        let mut img: Vec<u8> = img.into_raw();
        img.resize(SIZE * SIZE, 0);
        let mut buf = [0; SIZE * SIZE];
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

    pub fn draw_lines(&mut self, s: &Color, lines: &[Scanline]) {
        const M: u32 = 0xffff;
        const MS: u32 = (M >> 8);
        let sr = s.r() as u32;
        let sg = s.g() as u32;
        let sb = s.b() as u32;
        let sa = s.a() as u32;
        for line in lines {
            let ma = line.alpha as u32;
            let a = ((M - s.a() as u32 * ma as u32 / M) * 0x101);
            for x in line.x1..(line.x2 + 1) {
                let x = x as usize;
                let y = line.y as usize;
                let d = self.get(x, y);
                let c = Color::new(
                    ((d.r() as u32 * a + sr * ma) / MS) as u8,
                    ((d.g() as u32 * a + sg * ma) / MS) as u8,
                    ((d.b() as u32 * a + sb * ma) / MS) as u8,
                    ((d.a() as u32 * a + sa * ma) / MS) as u8,
                );
                println!("a={}", ((d.a() as u32 * a + sa * ma) / MS));
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

