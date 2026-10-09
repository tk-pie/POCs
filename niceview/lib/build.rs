use build_xbm::generate_for_xbm_file;
use eg_font_converter::{FontConverter, Mapping};

use crate::build_xbm::generate_mod_file;

const RAW_FONT_DIR: &'static str = "./resources/fonts/bdf";
const FONT_OUTPUT_DIR: &'static str = "./src/typography/generated/";

mod build_xbm;

/// Parse, compile and re-store fonts.
fn main() {
    load_font("Tamzen10x20r", "tamzen_10x20r");
    load_font("Tamzen10x20b", "tamzen_10x20b");
    load_font("5x7", "small_5x7");

    generate_for_xbm_file("battery_full", "battery_full");
    generate_for_xbm_file("battery_three_quarter", "battery_three_quarter");
    generate_for_xbm_file("battery_half", "battery_half");
    generate_for_xbm_file("battery_quarter", "battery_quarter");
    generate_for_xbm_file("bluetooth", "bluetooth_connected");
    generate_for_xbm_file("no_bluetooth", "bluetooth_disconnected");
    generate_mod_file();

    println!("cargo:rerun-if-changed=build.rs")
}

fn load_font<'a>(file_name: &'a str, output_name: &'a str) {
    let font = FontConverter::new(format!("{RAW_FONT_DIR}/{file_name}.bdf"), &output_name)
        .glyphs(Mapping::Iso8859_1)
        .missing_glyph_substitute(' ')
        .convert_eg_bdf()
        .expect("Failed to convert large font");

    font.save(FONT_OUTPUT_DIR).expect("Failed to save font");
}
