use eframe::egui;
use embassy_time::Instant;
use niceview_lib::color::Color;
use niceview_lib::video_decoder::VideoDecoder;
use niceview_lib::{DISP_HEIGHT, DISP_WIDTH, Icon, KeyboardDisplay};
use virtual_display::VirtualDisplay;

pub mod virtual_display;

#[cfg(feature = "bad_apple")]
const BAD_APPLE_COMPRESSED: &[u8; 1669901] = include_bytes!("../resources/video/ba.bin");

const FRAMES_PER_SECOND: u64 = 30;
const FRAME_DURATION_MS: u64 = 1000 / FRAMES_PER_SECOND;

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Display testing",
        native_options,
        Box::new(|cc| Ok(Box::new(DisplayApp::new(cc)))),
    )
    .expect("EGUI error");
}

struct DisplayApp {
    display: VirtualDisplay,
    last: Instant,
    #[cfg(feature = "bad_apple")]
    frames: VideoDecoder<{ DISP_WIDTH }, { DISP_HEIGHT }>,
}

impl DisplayApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        #[cfg(feature = "bad_apple")]
        let frames = {
            let mut f = VideoDecoder::new(BAD_APPLE_COMPRESSED);
            f.get_frame();
            f
        };

        let mut display = VirtualDisplay::new();

        display.draw_text("TK-Pie", 2, 20, niceview_lib::TextVariant::LargeBold);

        display.draw_text(
            "Current layout: Layer 1x2",
            4,
            28,
            niceview_lib::TextVariant::Regular,
        );

        display.draw_text("5%", 132, 15, niceview_lib::TextVariant::Regular);
        // display.draw_icon(Icon::BatteryCritical, 143, 9);

        display.draw_icon(Icon::BluetoothConnected, 136, 24);
        display.draw_icon(Icon::BluetoothDisconnected, 146, 24);

        display.draw_icon(Icon::BatteryQuarter, 10, 50);
        display.draw_icon(Icon::BatteryHalf, 30, 50);
        display.draw_icon(Icon::BatteryThreeQuarter, 50, 50);
        display.draw_icon(Icon::BatteryFull, 70, 50);

        Self {
            display,
            last: Instant::now(),
            #[cfg(feature = "bad_apple")]
            frames,
        }
    }
}

impl eframe::App for DisplayApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!(
                "Display (size: {}x{})",
                VirtualDisplay::WIDTH,
                VirtualDisplay::HEIGHT
            ));
            #[cfg(feature = "bad_apple")]
            {
                for x in 0..DISP_WIDTH {
                    for y in 0..DISP_HEIGHT {
                        let pixel = self.frames.buffer[x][y];
                        if pixel {
                            self.display.draw_pixel(x, y, Color::White);
                        } else {
                            self.display.draw_pixel(x, y, Color::Black);
                        }
                    }
                }
                let now = Instant::now();
                let delta = now - self.last;
                if delta.as_millis() > FRAME_DURATION_MS {
                    self.last = now;
                    self.frames.get_frame();
                }
                ui.ctx().request_repaint();
            }
            self.display.update(ui);
        });
    }
}
