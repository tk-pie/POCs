use eg_bdf::BdfFont;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::{Dimensions, DrawTarget, Point, Size},
};

use crate::KeyboardDisplay;

/// Generated files.
#[doc(hidden)]
#[allow(non_upper_case_globals)]
pub mod generated;

/// Icons.
pub mod icons;

/// A variation of text style.
#[allow(missing_docs)]
pub enum TextVariant {
    Regular,
    Large,
    LargeBold,
}

pub fn font_for_variant<'a>(variant: TextVariant) -> BdfFont<'a> {
    match variant {
        TextVariant::Regular => generated::small_5x7::small_5x7,
        TextVariant::Large => generated::tamzen_10x20r::tamzen_10x20r,
        TextVariant::LargeBold => generated::tamzen_10x20b::tamzen_10x20b,
    }
}

pub struct TextDisplayer<'a, T: ?Sized>(&'a mut T);

impl<'a, T> TextDisplayer<'a, T>
where
    T: KeyboardDisplay + ?Sized,
{
    pub fn new(v: &'a mut T) -> Self {
        Self(v)
    }
}

// TODO: Integer castings?
impl<'a, T: KeyboardDisplay + ?Sized> Dimensions for TextDisplayer<'a, T> {
    fn bounding_box(&self) -> embedded_graphics::primitives::Rectangle {
        embedded_graphics::primitives::Rectangle::new(
            Point::zero(),
            Size::new(T::WIDTH as u32, T::HEIGHT as u32),
        )
    }
}

impl<'a, T: KeyboardDisplay + ?Sized> DrawTarget for TextDisplayer<'a, T> {
    type Color = BinaryColor;

    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        for p in pixels {
            if p.1.is_on() {
                let Point { x, y } = p.0;
                self.0
                    .draw_pixel(x as usize, y as usize, crate::color::Color::Black);
            }
        }

        Ok(())
    }
}
