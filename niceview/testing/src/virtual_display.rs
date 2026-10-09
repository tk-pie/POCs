use eframe::egui::{Color32, Pos2, Rect, Sense, Ui, Vec2};
use niceview_lib::{KeyboardDisplay, color::Color};

// How many actual pixels each virtual pixel should be.
const SCALE: usize = 4;
const WHITE_COLOR: Color32 = Color32::from_rgb(160, 230, 180);
const BLACK_COLOR: Color32 = Color32::from_rgb(20, 40, 30);

pub struct VirtualDisplay {
    pixels: [[Color; VirtualDisplay::WIDTH]; VirtualDisplay::HEIGHT],
}

impl VirtualDisplay {
    pub fn new() -> Self {
        Self {
            pixels: [[Color::White; VirtualDisplay::WIDTH]; VirtualDisplay::HEIGHT],
        }
    }
}

impl VirtualDisplay {
    pub fn update(&self, ui: &mut Ui) {
        // let painter = ui.painter();
        let draw_area_size = Vec2::new(
            (VirtualDisplay::WIDTH * SCALE) as f32,
            (VirtualDisplay::HEIGHT * SCALE) as f32,
        ) + Vec2::splat(1.0);
        let (_response, painter) = ui.allocate_painter(draw_area_size, Sense::hover());

        for (y, row) in self.pixels.iter().enumerate() {
            for (x, col) in row.iter().enumerate() {
                let start_pos = Pos2::new((x * SCALE) as f32, (y * SCALE) as f32);
                let rect = Rect::from_min_size(start_pos, Vec2::new(SCALE as f32, SCALE as f32));
                let color = convert_color(col);

                painter.rect_filled(rect, 0.0, color);
            }
        }
    }
}

fn convert_color(color: &Color) -> Color32 {
    match color {
        Color::Black => BLACK_COLOR,
        Color::White => WHITE_COLOR,
    }
}

impl KeyboardDisplay for VirtualDisplay {
    async fn clear_display(&mut self) {
        todo!()
    }

    async fn flush(&mut self) {
        todo!()
    }

    fn fill_white(&mut self) {
        todo!()
    }

    fn draw_pixel(&mut self, x: usize, y: usize, color: Color) {
        self.pixels[y][x] = color;
    }

    async fn write(&mut self, data: &[u8]) {
        todo!()
    }
}
