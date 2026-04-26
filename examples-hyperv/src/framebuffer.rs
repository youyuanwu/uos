/// Simple framebuffer text renderer using an 8x16 bitmap font.
/// Renders ASCII characters directly to the bootloader-provided framebuffer.

pub struct FramebufferWriter {
    buf: *mut u8,
    width: usize,
    height: usize,
    stride: usize,
    bpp: usize,
    col: usize,
    row: usize,
}

unsafe impl Send for FramebufferWriter {}

impl FramebufferWriter {
    /// Create a new writer from bootloader framebuffer info.
    ///
    /// # Safety
    /// `buf` must point to a valid framebuffer mapping for the lifetime of this writer.
    pub unsafe fn new(
        buf: *mut u8,
        width: usize,
        height: usize,
        stride: usize,
        bpp: usize,
    ) -> Self {
        Self {
            buf,
            width,
            height,
            stride,
            bpp,
            col: 0,
            row: 0,
        }
    }

    /// Maximum columns and rows of text.
    fn cols(&self) -> usize {
        self.width / 8
    }
    fn rows(&self) -> usize {
        self.height / 16
    }

    /// Write a single character at the current cursor position.
    fn put_char(&mut self, c: u8) {
        if c == b'\n' {
            self.col = 0;
            self.row += 1;
            if self.row >= self.rows() {
                self.scroll();
                self.row = self.rows() - 1;
            }
            return;
        }
        if c == b'\r' {
            self.col = 0;
            return;
        }

        if self.col >= self.cols() {
            self.col = 0;
            self.row += 1;
            if self.row >= self.rows() {
                self.scroll();
                self.row = self.rows() - 1;
            }
        }

        let glyph = get_glyph(c);
        let px = self.col * 8;
        let py = self.row * 16;

        for gy in 0..16 {
            let row_bits = glyph[gy];
            for gx in 0..8 {
                let on = (row_bits >> (7 - gx)) & 1 != 0;
                let pixel_offset = ((py + gy) * self.stride + (px + gx)) * self.bpp;
                unsafe {
                    let p = self.buf.add(pixel_offset);
                    if on {
                        // White pixel (works for both BGR and RGB)
                        for i in 0..self.bpp.min(3) {
                            *p.add(i) = 0xFF;
                        }
                    } else {
                        // Black pixel
                        for i in 0..self.bpp.min(4) {
                            *p.add(i) = 0x00;
                        }
                    }
                }
            }
        }

        self.col += 1;
    }

    /// Scroll the screen up by one text row (16 pixels).
    fn scroll(&mut self) {
        let row_bytes = self.stride * self.bpp * 16;
        let total_bytes = self.stride * self.bpp * self.height;
        unsafe {
            core::ptr::copy(self.buf.add(row_bytes), self.buf, total_bytes - row_bytes);
            // Clear last row
            core::ptr::write_bytes(self.buf.add(total_bytes - row_bytes), 0, row_bytes);
        }
    }

    /// Write a string.
    pub fn write_str(&mut self, s: &str) {
        for b in s.bytes() {
            self.put_char(b);
        }
    }
}

impl core::fmt::Write for FramebufferWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

/// Basic 8x16 bitmap font — ASCII 0x20..0x7F.
/// Uses a minimal built-in font. Unrecognized chars render as '?'.
fn get_glyph(c: u8) -> [u8; 16] {
    if c < 0x20 || c > 0x7E {
        return FONT[(b'?' - 0x20) as usize];
    }
    FONT[(c - 0x20) as usize]
}

/// Minimal 8x16 bitmap font covering printable ASCII.
/// Each character is 16 bytes (one byte per row, MSB-first).
static FONT: [[u8; 16]; 95] = {
    let mut f = [[0u8; 16]; 95];
    // Space (0x20)
    f[0] = [0; 16];
    // '!' (0x21)
    f[1] = [
        0, 0, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00, 0x18, 0x18, 0, 0, 0, 0,
    ];
    // '"' (0x22)
    f[2] = [0, 0, 0x6C, 0x6C, 0x6C, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    // '#' .. '~' — fill with a recognizable default (vertical bar pattern for unimplemented)
    // For a real implementation, embed a full font. Here we provide key chars:
    // '0'-'9' (0x30-0x39 → index 16-25)
    f[16] = [
        0, 0, 0x3C, 0x66, 0x66, 0x6E, 0x76, 0x66, 0x66, 0x66, 0x3C, 0, 0, 0, 0, 0,
    ]; // 0
    f[17] = [
        0, 0, 0x18, 0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x7E, 0, 0, 0, 0, 0,
    ]; // 1
    f[18] = [
        0, 0, 0x3C, 0x66, 0x06, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x7E, 0, 0, 0, 0, 0,
    ]; // 2
    f[19] = [
        0, 0, 0x3C, 0x66, 0x06, 0x1C, 0x06, 0x06, 0x06, 0x66, 0x3C, 0, 0, 0, 0, 0,
    ]; // 3
    f[20] = [
        0, 0, 0x0C, 0x1C, 0x3C, 0x6C, 0x6C, 0x7E, 0x0C, 0x0C, 0x0C, 0, 0, 0, 0, 0,
    ]; // 4
    f[21] = [
        0, 0, 0x7E, 0x60, 0x60, 0x7C, 0x06, 0x06, 0x06, 0x66, 0x3C, 0, 0, 0, 0, 0,
    ]; // 5
    f[22] = [
        0, 0, 0x1C, 0x30, 0x60, 0x7C, 0x66, 0x66, 0x66, 0x66, 0x3C, 0, 0, 0, 0, 0,
    ]; // 6
    f[23] = [
        0, 0, 0x7E, 0x06, 0x0C, 0x0C, 0x18, 0x18, 0x18, 0x18, 0x18, 0, 0, 0, 0, 0,
    ]; // 7
    f[24] = [
        0, 0, 0x3C, 0x66, 0x66, 0x3C, 0x66, 0x66, 0x66, 0x66, 0x3C, 0, 0, 0, 0, 0,
    ]; // 8
    f[25] = [
        0, 0, 0x3C, 0x66, 0x66, 0x66, 0x3E, 0x06, 0x06, 0x0C, 0x38, 0, 0, 0, 0, 0,
    ]; // 9
    // ':' (0x3A → index 26)
    f[26] = [0, 0, 0, 0, 0x18, 0x18, 0, 0, 0x18, 0x18, 0, 0, 0, 0, 0, 0];
    // '=' (0x3D → index 29)
    f[29] = [0, 0, 0, 0, 0, 0x7E, 0, 0x7E, 0, 0, 0, 0, 0, 0, 0, 0];
    // '?' (0x3F → index 31)
    f[31] = [
        0, 0, 0x3C, 0x66, 0x06, 0x0C, 0x18, 0x18, 0x18, 0x00, 0x18, 0x18, 0, 0, 0, 0,
    ];
    // 'A'-'Z' (0x41-0x5A → index 33-58)
    f[33] = [
        0, 0, 0x18, 0x3C, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x66, 0x66, 0, 0, 0, 0, 0,
    ]; // A
    f[34] = [
        0, 0, 0x7C, 0x66, 0x66, 0x7C, 0x66, 0x66, 0x66, 0x66, 0x7C, 0, 0, 0, 0, 0,
    ]; // B
    f[35] = [
        0, 0, 0x3C, 0x66, 0x60, 0x60, 0x60, 0x60, 0x60, 0x66, 0x3C, 0, 0, 0, 0, 0,
    ]; // C
    f[36] = [
        0, 0, 0x78, 0x6C, 0x66, 0x66, 0x66, 0x66, 0x66, 0x6C, 0x78, 0, 0, 0, 0, 0,
    ]; // D
    f[37] = [
        0, 0, 0x7E, 0x60, 0x60, 0x7C, 0x60, 0x60, 0x60, 0x60, 0x7E, 0, 0, 0, 0, 0,
    ]; // E
    f[38] = [
        0, 0, 0x7E, 0x60, 0x60, 0x7C, 0x60, 0x60, 0x60, 0x60, 0x60, 0, 0, 0, 0, 0,
    ]; // F
    f[39] = [
        0, 0, 0x3C, 0x66, 0x60, 0x60, 0x6E, 0x66, 0x66, 0x66, 0x3E, 0, 0, 0, 0, 0,
    ]; // G
    f[40] = [
        0, 0, 0x66, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x66, 0x66, 0x66, 0, 0, 0, 0, 0,
    ]; // H
    f[41] = [
        0, 0, 0x3C, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0, 0, 0, 0, 0,
    ]; // I
    f[46] = [
        0, 0, 0x66, 0x6C, 0x78, 0x70, 0x78, 0x6C, 0x66, 0x66, 0x66, 0, 0, 0, 0, 0,
    ]; // K (index 43)
    // Fix: proper indices
    // 'H' is 0x48 → index 40, 'K' is 0x4B → index 43
    f[43] = [
        0, 0, 0x66, 0x6C, 0x78, 0x70, 0x78, 0x6C, 0x66, 0x66, 0x66, 0, 0, 0, 0, 0,
    ]; // K
    f[44] = [
        0, 0, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x7E, 0, 0, 0, 0, 0,
    ]; // L
    f[45] = [
        0, 0, 0x63, 0x77, 0x7F, 0x6B, 0x63, 0x63, 0x63, 0x63, 0x63, 0, 0, 0, 0, 0,
    ]; // M
    f[46] = [
        0, 0, 0x66, 0x76, 0x7E, 0x7E, 0x6E, 0x66, 0x66, 0x66, 0x66, 0, 0, 0, 0, 0,
    ]; // N
    f[47] = [
        0, 0, 0x3C, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0, 0, 0, 0, 0,
    ]; // O
    f[48] = [
        0, 0, 0x7C, 0x66, 0x66, 0x7C, 0x60, 0x60, 0x60, 0x60, 0x60, 0, 0, 0, 0, 0,
    ]; // P
    f[50] = [
        0, 0, 0x7C, 0x66, 0x66, 0x7C, 0x6C, 0x66, 0x66, 0x66, 0x66, 0, 0, 0, 0, 0,
    ]; // R
    f[51] = [
        0, 0, 0x3C, 0x66, 0x60, 0x3C, 0x06, 0x06, 0x06, 0x66, 0x3C, 0, 0, 0, 0, 0,
    ]; // S
    f[52] = [
        0, 0, 0x7E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0, 0, 0, 0, 0,
    ]; // T
    f[53] = [
        0, 0, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0, 0, 0, 0, 0,
    ]; // U
    f[54] = [
        0, 0, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x18, 0, 0, 0, 0, 0,
    ]; // V
    // 'a'-'z' (0x61-0x7A → index 65-90)
    f[65] = [
        0, 0, 0, 0, 0, 0x3C, 0x06, 0x3E, 0x66, 0x66, 0x3E, 0, 0, 0, 0, 0,
    ]; // a
    f[66] = [
        0, 0, 0x60, 0x60, 0x60, 0x7C, 0x66, 0x66, 0x66, 0x66, 0x7C, 0, 0, 0, 0, 0,
    ]; // b
    f[67] = [
        0, 0, 0, 0, 0, 0x3C, 0x66, 0x60, 0x60, 0x66, 0x3C, 0, 0, 0, 0, 0,
    ]; // c
    f[68] = [
        0, 0, 0x06, 0x06, 0x06, 0x3E, 0x66, 0x66, 0x66, 0x66, 0x3E, 0, 0, 0, 0, 0,
    ]; // d
    f[69] = [
        0, 0, 0, 0, 0, 0x3C, 0x66, 0x7E, 0x60, 0x66, 0x3C, 0, 0, 0, 0, 0,
    ]; // e
    f[70] = [
        0, 0, 0x1C, 0x30, 0x30, 0x7C, 0x30, 0x30, 0x30, 0x30, 0x30, 0, 0, 0, 0, 0,
    ]; // f
    f[72] = [
        0, 0, 0x60, 0x60, 0x60, 0x7C, 0x66, 0x66, 0x66, 0x66, 0x66, 0, 0, 0, 0, 0,
    ]; // h
    f[73] = [
        0, 0, 0x18, 0, 0, 0x38, 0x18, 0x18, 0x18, 0x18, 0x3C, 0, 0, 0, 0, 0,
    ]; // i
    f[76] = [
        0, 0, 0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, 0, 0, 0, 0, 0,
    ]; // l
    f[77] = [
        0, 0, 0, 0, 0, 0x76, 0x7F, 0x6B, 0x6B, 0x63, 0x63, 0, 0, 0, 0, 0,
    ]; // m
    f[78] = [
        0, 0, 0, 0, 0, 0x7C, 0x66, 0x66, 0x66, 0x66, 0x66, 0, 0, 0, 0, 0,
    ]; // n
    f[79] = [
        0, 0, 0, 0, 0, 0x3C, 0x66, 0x66, 0x66, 0x66, 0x3C, 0, 0, 0, 0, 0,
    ]; // o
    f[80] = [
        0, 0, 0, 0, 0, 0x7C, 0x66, 0x66, 0x66, 0x7C, 0x60, 0x60, 0x60, 0, 0, 0,
    ]; // p
    f[82] = [
        0, 0, 0, 0, 0, 0x3E, 0x66, 0x60, 0x60, 0x60, 0x60, 0, 0, 0, 0, 0,
    ]; // r
    f[83] = [
        0, 0, 0, 0, 0, 0x3E, 0x60, 0x3C, 0x06, 0x06, 0x7C, 0, 0, 0, 0, 0,
    ]; // s
    f[84] = [
        0, 0, 0x18, 0x18, 0x18, 0x7E, 0x18, 0x18, 0x18, 0x18, 0x0E, 0, 0, 0, 0, 0,
    ]; // t
    f[85] = [
        0, 0, 0, 0, 0, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3E, 0, 0, 0, 0, 0,
    ]; // u
    f[86] = [
        0, 0, 0, 0, 0, 0x66, 0x66, 0x66, 0x3C, 0x18, 0x18, 0, 0, 0, 0, 0,
    ]; // v
    f[87] = [
        0, 0, 0, 0, 0, 0x63, 0x63, 0x6B, 0x7F, 0x36, 0x36, 0, 0, 0, 0, 0,
    ]; // w
    f[88] = [
        0, 0, 0, 0, 0, 0x66, 0x66, 0x3C, 0x3C, 0x66, 0x66, 0, 0, 0, 0, 0,
    ]; // x
    f[89] = [
        0, 0, 0, 0, 0, 0x66, 0x66, 0x66, 0x3E, 0x06, 0x06, 0x3C, 0, 0, 0, 0,
    ]; // y
    // '.' (0x2E → index 14)
    f[14] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0x18, 0x18, 0, 0, 0, 0, 0];
    // ',' (0x2C → index 12)
    f[12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0x18, 0x18, 0x08, 0x10, 0, 0, 0];
    // '-' (0x2D → index 13)
    f[13] = [0, 0, 0, 0, 0, 0, 0, 0x7E, 0, 0, 0, 0, 0, 0, 0, 0];
    // '_' (0x5F → index 63)
    f[63] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x7E, 0, 0];
    // ' ' already [0]
    // '[' (0x5B → index 59)
    f[59] = [
        0, 0, 0x1E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x1E, 0, 0, 0, 0, 0,
    ];
    // ']' (0x5D → index 61)
    f[61] = [
        0, 0, 0x78, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x78, 0, 0, 0, 0, 0,
    ];
    // '(' (0x28 → index 8)
    f[8] = [
        0, 0, 0x0C, 0x18, 0x18, 0x30, 0x30, 0x30, 0x18, 0x18, 0x0C, 0, 0, 0, 0, 0,
    ];
    // ')' (0x29 → index 9)
    f[9] = [
        0, 0, 0x30, 0x18, 0x18, 0x0C, 0x0C, 0x0C, 0x18, 0x18, 0x30, 0, 0, 0, 0, 0,
    ];
    // '/' (0x2F → index 15)
    f[15] = [
        0, 0, 0x02, 0x06, 0x0C, 0x0C, 0x18, 0x18, 0x30, 0x30, 0x60, 0, 0, 0, 0, 0,
    ];
    // '+' (0x2B → index 11)
    f[11] = [
        0, 0, 0, 0, 0x18, 0x18, 0x7E, 0x18, 0x18, 0, 0, 0, 0, 0, 0, 0,
    ];
    // '*' (0x2A → index 10)
    f[10] = [
        0, 0, 0, 0, 0x66, 0x3C, 0xFF, 0x3C, 0x66, 0, 0, 0, 0, 0, 0, 0,
    ];
    f
};
