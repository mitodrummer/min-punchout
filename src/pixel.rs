//! A tiny pixel canvas rendered through half-block glyphs. Each terminal cell
//! holds two vertical pixels: `▀` with a foreground for the top pixel and a
//! background for the bottom one. True colour throughout.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    pub const fn hex(v: u32) -> Rgb {
        Rgb(((v >> 16) & 0xff) as u8, ((v >> 8) & 0xff) as u8, (v & 0xff) as u8)
    }
    /// Blend toward `to` by `t` in 0..=1.
    pub fn mix(self, to: Rgb, t: f32) -> Rgb {
        let f = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round().clamp(0.0, 255.0) as u8;
        Rgb(f(self.0, to.0), f(self.1, to.1), f(self.2, to.2))
    }
    pub fn darker(self, t: f32) -> Rgb {
        self.mix(Rgb(0, 0, 0), t)
    }
    fn color(self) -> Color {
        Color::Rgb(self.0, self.1, self.2)
    }
}

pub struct Canvas {
    pub w: i32,
    pub h: i32,
    px: Vec<Rgb>,
    /// Optional tint applied to every pixel drawn while set.
    tint: Option<(Rgb, f32)>,
}

impl Canvas {
    pub fn new(w: i32, h: i32, bg: Rgb) -> Canvas {
        Canvas { w, h, px: vec![bg; (w.max(0) * h.max(0)) as usize], tint: None }
    }

    pub fn set_tint(&mut self, tint: Option<(Rgb, f32)>) {
        self.tint = tint;
    }

    pub fn put(&mut self, x: i32, y: i32, c: Rgb) {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return;
        }
        let c = match self.tint {
            Some((to, t)) => c.mix(to, t),
            None => c,
        };
        self.px[(y * self.w + x) as usize] = c;
    }

    pub fn get(&self, x: i32, y: i32) -> Option<Rgb> {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            None
        } else {
            Some(self.px[(y * self.w + x) as usize])
        }
    }

    /// Copy a w x h region of `src` at (sx, sy) to (dx, dy) here.
    pub fn blit(&mut self, src: &Canvas, sx: i32, sy: i32, w: i32, h: i32, dx: i32, dy: i32) {
        for y in 0..h {
            for x in 0..w {
                if let Some(c) = src.get(sx + x, sy + y) {
                    self.put(dx + x, dy + y, c);
                }
            }
        }
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Rgb) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.put(xx, yy, c);
            }
        }
    }

    /// Filled rectangle with a one pixel outline.
    pub fn rect_o(&mut self, x: i32, y: i32, w: i32, h: i32, fill: Rgb, outline: Rgb) {
        self.rect(x - 1, y - 1, w + 2, h + 2, outline);
        self.rect(x, y, w, h, fill);
    }

    pub fn ellipse(&mut self, cx: i32, cy: i32, rx: f32, ry: f32, c: Rgb) {
        let top = (cy as f32 - ry).floor() as i32;
        let bot = (cy as f32 + ry).ceil() as i32;
        for y in top..=bot {
            let dy = (y as f32 - cy as f32) / ry.max(0.5);
            if dy.abs() > 1.0 {
                continue;
            }
            let half = rx * (1.0 - dy * dy).sqrt();
            let x0 = (cx as f32 - half).round() as i32;
            let x1 = (cx as f32 + half).round() as i32;
            for x in x0..=x1 {
                self.put(x, y, c);
            }
        }
    }

    /// Filled ellipse with a one pixel outline.
    pub fn ellipse_o(&mut self, cx: i32, cy: i32, rx: f32, ry: f32, fill: Rgb, outline: Rgb) {
        self.ellipse(cx, cy, rx + 1.0, ry + 1.0, outline);
        self.ellipse(cx, cy, rx, ry, fill);
    }

    /// Thick line: a run of filled circles.
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, width: f32, c: Rgb) {
        let dx = (x1 - x0) as f32;
        let dy = (y1 - y0) as f32;
        let steps = dx.abs().max(dy.abs()).max(1.0) as i32 * 2;
        let r = width / 2.0;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = x0 as f32 + dx * t;
            let y = y0 as f32 + dy * t;
            self.ellipse(x.round() as i32, y.round() as i32, r, r, c);
        }
    }

    /// Thick line with an outline underneath.
    pub fn line_o(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, width: f32, fill: Rgb, outline: Rgb) {
        self.line(x0, y0, x1, y1, width + 2.0, outline);
        self.line(x0, y0, x1, y1, width, fill);
    }

    /// 3x5 pixel font. `scale` multiplies glyph size. Returns the width drawn.
    pub fn text(&mut self, x: i32, y: i32, s: &str, scale: i32, c: Rgb) -> i32 {
        let mut cx = x;
        for ch in s.chars() {
            let g = glyph(ch);
            for (row, bits) in g.iter().enumerate() {
                for col in 0..3 {
                    if bits & (0b100 >> col) != 0 {
                        self.rect(cx + col * scale, y + row as i32 * scale, scale, scale, c);
                    }
                }
            }
            cx += 4 * scale;
        }
        cx - x - scale
    }

    pub fn text_width(s: &str, scale: i32) -> i32 {
        (s.chars().count() as i32 * 4 - 1) * scale
    }

    /// Blit onto a ratatui buffer at `area`, with each logical pixel covering
    /// `scale` x `scale` physical pixels and two physical pixel rows per cell.
    pub fn flush(&self, buf: &mut Buffer, area: Rect, scale: i32) {
        let cells_w = (self.w * scale).min(area.width as i32);
        let cells_h = ((self.h * scale + 1) / 2).min(area.height as i32);
        for cy in 0..cells_h {
            for cx in 0..cells_w {
                let lx = cx / scale;
                let top = self.get(lx, (cy * 2) / scale);
                let bot = self.get(lx, (cy * 2 + 1) / scale);
                let cell = &mut buf[((area.x as i32 + cx) as u16, (area.y as i32 + cy) as u16)];
                match (top, bot) {
                    (Some(t), Some(b)) if t == b => {
                        cell.set_char('█').set_style(Style::new().fg(t.color()));
                    }
                    (Some(t), Some(b)) => {
                        cell.set_char('▀').set_style(Style::new().fg(t.color()).bg(b.color()));
                    }
                    (Some(t), None) => {
                        cell.set_char('▀').set_style(Style::new().fg(t.color()));
                    }
                    (None, Some(b)) => {
                        cell.set_char('▄').set_style(Style::new().fg(b.color()));
                    }
                    (None, None) => {}
                }
            }
        }
    }
}

/// Rows top to bottom, 3 bits each.
fn glyph(ch: char) -> [u8; 5] {
    match ch.to_ascii_uppercase() {
        'A' => [0b010, 0b101, 0b111, 0b101, 0b101],
        'B' => [0b110, 0b101, 0b110, 0b101, 0b110],
        'C' => [0b011, 0b100, 0b100, 0b100, 0b011],
        'D' => [0b110, 0b101, 0b101, 0b101, 0b110],
        'E' => [0b111, 0b100, 0b111, 0b100, 0b111],
        'F' => [0b111, 0b100, 0b110, 0b100, 0b100],
        'G' => [0b011, 0b100, 0b101, 0b101, 0b011],
        'H' => [0b101, 0b101, 0b111, 0b101, 0b101],
        'I' => [0b111, 0b010, 0b010, 0b010, 0b111],
        'J' => [0b001, 0b001, 0b001, 0b101, 0b010],
        'K' => [0b101, 0b101, 0b110, 0b101, 0b101],
        'L' => [0b100, 0b100, 0b100, 0b100, 0b111],
        'M' => [0b101, 0b111, 0b111, 0b101, 0b101],
        'N' => [0b110, 0b101, 0b101, 0b101, 0b101],
        'O' => [0b010, 0b101, 0b101, 0b101, 0b010],
        'P' => [0b110, 0b101, 0b110, 0b100, 0b100],
        'Q' => [0b010, 0b101, 0b101, 0b111, 0b001],
        'R' => [0b110, 0b101, 0b110, 0b101, 0b101],
        'S' => [0b011, 0b100, 0b010, 0b001, 0b110],
        'T' => [0b111, 0b010, 0b010, 0b010, 0b010],
        'U' => [0b101, 0b101, 0b101, 0b101, 0b111],
        'V' => [0b101, 0b101, 0b101, 0b101, 0b010],
        'W' => [0b101, 0b101, 0b111, 0b111, 0b101],
        'X' => [0b101, 0b101, 0b010, 0b101, 0b101],
        'Y' => [0b101, 0b101, 0b010, 0b010, 0b010],
        'Z' => [0b111, 0b001, 0b010, 0b100, 0b111],
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b110, 0b001, 0b010, 0b100, 0b111],
        '3' => [0b111, 0b001, 0b011, 0b001, 0b111],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        '6' => [0b011, 0b100, 0b111, 0b101, 0b111],
        '7' => [0b111, 0b001, 0b010, 0b010, 0b010],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b110],
        '$' => [0b010, 0b111, 0b110, 0b011, 0b111],
        '!' => [0b010, 0b010, 0b010, 0b000, 0b010],
        '?' => [0b110, 0b001, 0b010, 0b000, 0b010],
        ':' => [0b000, 0b010, 0b000, 0b010, 0b000],
        '-' => [0b000, 0b000, 0b111, 0b000, 0b000],
        '+' => [0b000, 0b010, 0b111, 0b010, 0b000],
        '.' => [0b000, 0b000, 0b000, 0b000, 0b010],
        ',' => [0b000, 0b000, 0b000, 0b010, 0b100],
        '/' => [0b001, 0b001, 0b010, 0b100, 0b100],
        '\'' => [0b010, 0b010, 0b000, 0b000, 0b000],
        '"' => [0b101, 0b101, 0b000, 0b000, 0b000],
        '*' => [0b101, 0b010, 0b111, 0b010, 0b101],
        '(' => [0b010, 0b100, 0b100, 0b100, 0b010],
        ')' => [0b010, 0b001, 0b001, 0b001, 0b010],
        '<' => [0b001, 0b010, 0b100, 0b010, 0b001],
        '>' => [0b100, 0b010, 0b001, 0b010, 0b100],
        '%' => [0b101, 0b001, 0b010, 0b100, 0b101],
        _ => [0b000, 0b000, 0b000, 0b000, 0b000],
    }
}

#[cfg(test)]
pub mod png {
    //! Minimal uncompressed PNG writer so frames can be eyeballed as images.
    use super::Canvas;

    fn crc32(data: &[u8]) -> u32 {
        let mut table = [0u32; 256];
        for i in 0..256u32 {
            let mut c = i;
            for _ in 0..8 {
                c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            }
            table[i as usize] = c;
        }
        let mut crc = 0xFFFF_FFFFu32;
        for &b in data {
            crc = table[((crc ^ b as u32) & 0xff) as usize] ^ (crc >> 8);
        }
        crc ^ 0xFFFF_FFFF
    }

    fn adler32(data: &[u8]) -> u32 {
        let (mut a, mut b) = (1u32, 0u32);
        for &d in data {
            a = (a + d as u32) % 65521;
            b = (b + a) % 65521;
        }
        (b << 16) | a
    }

    fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        let mut body = Vec::with_capacity(4 + data.len());
        body.extend_from_slice(kind);
        body.extend_from_slice(data);
        out.extend_from_slice(&body);
        out.extend_from_slice(&crc32(&body).to_be_bytes());
    }

    /// Write the canvas as a PNG, each logical pixel as a `scale` x `scale`
    /// block, pixel rows doubled in height to mimic terminal cell aspect.
    pub fn write(c: &Canvas, path: &str, scale: usize) {
        let w = c.w as usize * scale;
        let h = c.h as usize * scale;
        let mut raw = Vec::with_capacity((w * 3 + 1) * h);
        for y in 0..h {
            raw.push(0u8);
            for x in 0..w {
                let p = c.get((x / scale) as i32, (y / scale) as i32).unwrap();
                raw.extend_from_slice(&[p.0, p.1, p.2]);
            }
        }
        // zlib stream with stored (uncompressed) deflate blocks.
        let mut z = vec![0x78, 0x01];
        for (i, block) in raw.chunks(65535).enumerate() {
            let last = (i + 1) * 65535 >= raw.len();
            z.push(if last { 1 } else { 0 });
            let len = block.len() as u16;
            z.extend_from_slice(&len.to_le_bytes());
            z.extend_from_slice(&(!len).to_le_bytes());
            z.extend_from_slice(block);
        }
        z.extend_from_slice(&adler32(&raw).to_be_bytes());

        let mut out = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        let mut ihdr = Vec::new();
        ihdr.extend_from_slice(&(w as u32).to_be_bytes());
        ihdr.extend_from_slice(&(h as u32).to_be_bytes());
        ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
        chunk(&mut out, b"IHDR", &ihdr);
        chunk(&mut out, b"IDAT", &z);
        chunk(&mut out, b"IEND", &[]);
        std::fs::write(path, out).unwrap();
    }
}

// ------------------------------------------------------------ title font

/// A 5x7 face for titles only: wide enough that M, N and W read as
/// themselves, which the 3x5 face cannot manage. Covers the letters the
/// title screen uses; anything else is blank.
fn big_glyph(ch: char) -> [u8; 7] {
    match ch.to_ascii_uppercase() {
        'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'I' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111],
        'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'C' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        _ => [0; 7],
    }
}

impl Canvas {
    pub fn big_text(&mut self, x: i32, y: i32, s: &str, scale: i32, c: Rgb) {
        let mut cx = x;
        for ch in s.chars() {
            let g = big_glyph(ch);
            for (row, bits) in g.iter().enumerate() {
                for col in 0..5 {
                    if bits & (0b10000 >> col) != 0 {
                        self.rect(cx + col * scale, y + row as i32 * scale, scale, scale, c);
                    }
                }
            }
            cx += 6 * scale;
        }
    }

    pub fn big_text_width(s: &str, scale: i32) -> i32 {
        (s.chars().count() as i32 * 6 - 1) * scale
    }
}

impl Canvas {
    /// Copy `src` rotated a quarter turn anticlockwise so its top edge faces
    /// left, with its top-left landing at (dx, dy). Pixels equal to `skip`
    /// are treated as transparent.
    pub fn blit_rot_ccw(&mut self, src: &Canvas, dx: i32, dy: i32, skip: Rgb) {
        for y in 0..src.h {
            for x in 0..src.w {
                if let Some(c) = src.get(x, y) {
                    if c != skip {
                        self.put(dx + y, dy + (src.w - 1 - x), c);
                    }
                }
            }
        }
    }
}
