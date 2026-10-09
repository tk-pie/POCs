//! An icon that we can draw.
//!
//! IMAGE_WIDTH = The actual pixel width of the image.
//! BYTE_WIDTH = The number of bytes wide the image is, the following should always hold:
//! ```
//! BYTE_WIDTH = (IMAGE_WIDTH + (8 - (IMAGE_WIDTH % 8))) / 8
//! ```
//!
//! HEIGHT = the height of the image in pixels.
#[allow(unused)]
pub struct Image<const IMAGE_WIDTH: usize, const BYTE_WIDTH: usize, const HEIGHT: usize> {
    pub rows: [[u8; BYTE_WIDTH]; HEIGHT],
}

impl<const IMAGE_WIDTH: usize, const BYTE_WIDTH: usize, const HEIGHT: usize>
    Image<IMAGE_WIDTH, BYTE_WIDTH, HEIGHT>
{
    #[allow(unused)]
    pub fn to_iterator<'a>(&'static self) -> ImageIterator<'a> {
        ImageIterator {
            byte_width: BYTE_WIDTH,
            index: 0,
            data: self.rows.as_flattened(),
        }
    }
}

#[derive(Debug, Clone)]

pub struct ImageIterator<'a> {
    byte_width: usize,
    index: usize,
    data: &'a [u8],
}

impl<'a> Iterator for ImageIterator<'a> {
    type Item = (usize, usize, u8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.data.len() {
            return None;
        }

        let b = self.data[self.index];

        let x = self.index % self.byte_width;
        let y = (self.index - x) / self.byte_width;

        self.index += 1;

        Some((x, y, b))
    }
}
