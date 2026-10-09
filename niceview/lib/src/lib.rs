#![no_std]
#![no_main]
#![deny(missing_docs)]

//! Lib for generic display handling.

use color::Color;

/// Colors.
pub mod color;

/// Image handling.
mod image;
mod images;
/// Text rendering.
mod typography;
pub use typography::TextVariant;

use typography::font_for_variant;

pub use crate::images::generated::Icon;

/// A display used on the keyboard.
#[allow(async_fn_in_trait)]
pub trait KeyboardDisplay {
    /// Display width in pixels.
    const WIDTH: usize = 160;

    /// Display height in pixels.
    const HEIGHT: usize = 68;

    /// Send a clear command to the screen.
    /// Effectively equivalent to [Self::fill_white] + [Self::flush] but faster.
    async fn clear_display(&mut self);

    /// Flush the current write buffer to the device.
    async fn flush(&mut self);

    /// Fill the entire screen with white.
    fn fill_white(&mut self);

    /// Set the color for a particular pixel.
    fn draw_pixel(&mut self, x: usize, y: usize, color: Color);

    /// Write data to the display.
    async fn write(&mut self, data: &[u8]);

    /// Draw a circle.
    fn draw_circle(&mut self, center_x: usize, center_y: usize, color: Color, radius: usize) {
        let center_x = center_x as isize;
        let center_y = center_y as isize;

        for x in 0..Self::WIDTH {
            for y in 0..Self::HEIGHT {
                let distance = {
                    let (x, y) = (x as isize, y as isize);
                    (center_x - x).abs() + (center_y - y).abs()
                };

                if distance <= radius as isize {
                    self.draw_pixel(x, y, color);
                }
            }
        }
    }

    /// Draw some text starting at the provided x and y positions.
    fn draw_text(&mut self, text: &str, start_x: usize, start_y: usize, text_variant: TextVariant) {
        use eg_bdf::BdfTextStyle;
        use embedded_graphics::{pixelcolor::BinaryColor, prelude::*, text::Text};
        use typography::TextDisplayer;

        // TODO: Look into if we can support alignment and/or baseline.
        let font = font_for_variant(text_variant);
        let text_style = BdfTextStyle::new(&font, BinaryColor::On);
        let mut drawable_display = TextDisplayer::new(self);

        // TODO: Handle error.
        let _ = Text::new(text, Point::new(start_x as i32, start_y as i32), text_style)
            .draw(&mut drawable_display);
    }

    /// Draw an icon starting at the provided x and y positions.
    fn draw_icon(&mut self, icon: Icon, start_x: usize, start_y: usize) {
        for (x, y, bits) in icon.get_image().into_iter() {
            for i in 0..8 {
                let x = start_x + x * 8 + i;
                let y = start_y + y;
                if (bits >> i) & 1 == 1 {
                    self.draw_pixel(x, y, Color::Black);
                }
            }
        }
    }
}
