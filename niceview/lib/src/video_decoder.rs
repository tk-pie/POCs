use core::slice::Iter;

use miniz_oxide::inflate::decompress_to_vec_with_limit;

/// A video decoder for our in-house video compression.
/// The compression format takes advantage of the b&w
/// output to use one bit per pixel when decompressed.
///
/// TODO: use block compression to save on repeated frames.
///
/// The video file is a stream of packets.
/// Each packet starts with two bytes representing
/// the compressed sized of the DEFLATE'ed blob that follows.
/// Once INFLATE'ed, each byte is a packed array of booleans
///
/// WIDTH = Width of the display in pixels.
/// HEIGH = Height of the display in pixels.
///

pub struct VideoDecoder<const WIDTH: usize, const HEIGHT: usize> {
    /// The current decoded buffer
    pub buffer: [[bool; HEIGHT]; WIDTH],
    source: &'static [u8],
    offset: usize,
}

impl<const WIDTH: usize, const HEIGHT: usize> VideoDecoder<WIDTH, HEIGHT> {
    /// Create new decoder with sources
    pub fn new(source: &'static [u8]) -> Self {
        Self {
            buffer: [[false; HEIGHT]; WIDTH],
            source,
            offset: 0,
        }
    }

    /// Decode a frame and update the buffer
    pub fn get_frame(&mut self) {
        let mut size: usize = (self.source[self.offset + 1] as usize) << 8;
        size += (self.source[self.offset] as usize);
        self.offset += 2;
        // Block is 4 bytes size plus compressed blob
        let bytes = &self.source[self.offset..(size + self.offset)];
        let frame =
            decompress_to_vec_with_limit(bytes, WIDTH * HEIGHT).expect("Decompression error");
        let mut pixels = BitResevoir::new(frame.iter());

        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                self.buffer[x][y] = pixels.get().unwrap_or(false);
            }
        }

        self.offset += size as usize;
        if self.offset >= self.source.len() - 200 {
            self.offset = 0;
        }
    }
}

struct BitResevoir<'a, I: Iterator<Item = &'a u8>> {
    source: I,
    pool: u8,
    bit_cursor: u8,
}

impl<'a, I: Iterator<Item = &'a u8>> BitResevoir<'a, I> {
    fn new(iter: I) -> Self {
        Self {
            source: iter,
            pool: 0,
            bit_cursor: 8,
        }
    }

    fn get(&mut self) -> Option<bool> {
        // Source new bits when pool is empty
        if self.bit_cursor >= 8 {
            self.pool = *self.source.next()?;
            self.bit_cursor = 0;
        }

        let v = self.pool & (1 << self.bit_cursor) != 0;
        self.bit_cursor += 1;
        Some(v)
    }
}
