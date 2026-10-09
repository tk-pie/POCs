use proc_macro2::{Ident, Literal, Span, TokenStream};
use rust_format::{Formatter, RustFmt};
use std::{
    fs,
    io::Write,
    path::Path,
    str::{FromStr, Lines},
};

use quote::{ToTokens, quote};

#[path = "src/image.rs"]
mod my_image;

const RAW_IMAGE_DIR: &'static str = "./resources/bitmap";
const IMAGE_OUTPUT_DIR: &'static str = "./src/images/generated/";

const MOD_FILE_NAME: &'static str = "mod.rs";
const RUST_FILE_EXTENSION: &'static str = ".rs";
const XBM_FILE_EXTENSION: &'static str = ".xbm";

const C_DEFINE_PREFIX: &'static str = "#define";
const C_STATIC_UNASIGNED_CHAR: &'static str = "static unsigned char";

pub fn generate_for_xbm_file(input_file_name: &'static str, output_name: &'static str) {
    let file_path_name = format!("{RAW_IMAGE_DIR}/{input_file_name}{XBM_FILE_EXTENSION}");
    let file_path = Path::new(&file_path_name);
    let file_content = fs::read_to_string(file_path).expect("Failed to read xbm file");

    let mut lines = file_content.lines();

    let width = next_starting_with(&mut lines, C_DEFINE_PREFIX)
        .expect("width define")
        .split(" ")
        .last()
        .expect("Expected a define part?")
        .parse::<usize>()
        .expect("Failed to parse image width");
    let height = next_starting_with(&mut lines, C_DEFINE_PREFIX)
        .expect("height define")
        .split(" ")
        .last()
        .expect("Expected a define part?")
        .parse::<usize>()
        .expect("Failed to parse image height");

    // Skip the actual static unsigned char line as it contains no data.
    next_starting_with(&mut lines, C_STATIC_UNASIGNED_CHAR).expect("Unable to find xbm data");

    let remaing = lines.collect::<String>();
    let pixels = remaing
        .strip_suffix("};")
        .expect("Failed to strip suffix of xbm file");

    let image_bytes = pixels
        .replace("\n", "")
        .replace(" ", "")
        .split(",")
        .map(|h| h.trim_start_matches("0x"))
        .map(|hex_bits| u8::from_str_radix(hex_bits, 16).expect("Failed to parse hex to u16"))
        .collect::<Vec<u8>>();

    let byte_width = convert_image_width_to_byte_width(width);

    let rows: TokenStream = image_bytes
        .chunks_exact(byte_width)
        .map(|s| {
            let a = s
                .iter()
                .map(|u| {
                    // TODO: This is a workaround in order to get the literal in hexadecimal format but there should be a better way of doing this.
                    let lit = Literal::from_str(&format!("0x{u:02x}"))
                        .expect("String to be valid (we just created it?)");
                    quote! { #lit, }
                })
                .collect::<TokenStream>();
            quote! { [ #a ], }
        })
        .collect();

    let output = quote! {
        use crate::image::Image;

        pub const IMAGE_DATA: Image<#width, #byte_width, #height>  = Image {
            rows: [ #rows ]
        };
    }
    .to_string();

    let formatted_output = RustFmt::default()
        .format_str(&output)
        .expect("Failed to format output");

    let output_file_path = format!("{IMAGE_OUTPUT_DIR}/{output_name}{RUST_FILE_EXTENSION}");
    let mut file =
        fs::File::create(output_file_path).expect("Failed to create image rust file output");

    file.write_all(formatted_output.as_bytes())
        .expect("Failed to write rust file for image");
}

pub fn generate_mod_file() {
    let file_names = fs::read_dir(IMAGE_OUTPUT_DIR)
        .expect("Failed to read image dir")
        .filter_map(|entry| {
            let entry = entry.expect("Failed to read file entry");
            let path = entry.path();

            if !path.is_file() {
                return None;
            }

            let name = path
                .file_name()
                .expect("Failed to read file name")
                .to_string_lossy()
                .to_string();

            if name == MOD_FILE_NAME {
                return None;
            }

            name.strip_suffix(RUST_FILE_EXTENSION)
                .map(|n| n.to_string())
        })
        .collect::<Vec<_>>();

    let mods = file_names
        .iter()
        .map(|name| {
            let name = Ident::new(name, Span::call_site());
            quote! {
                #[allow(missing_docs)]
                mod #name;
            }
        })
        .collect::<TokenStream>();

    let enum_variant_names = file_names
        .iter()
        .map(|name| {
            (
                name,
                name.split("_")
                    .map(|part| {
                        part.to_lowercase()
                            .chars()
                            .enumerate()
                            .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
                            .collect::<String>()
                    })
                    .collect::<String>(),
            )
        })
        .map(|(name, enum_name)| {
            (
                Ident::new(name, Span::call_site()).to_token_stream(),
                Ident::new(&enum_name, Span::call_site()).to_token_stream(),
            )
        })
        .collect::<Vec<_>>();

    let enum_variant_decls = enum_variant_names
        .iter()
        .map(|(_, enum_name)| enum_name)
        .map(|name| quote! { #name, })
        .collect::<TokenStream>();

    let enum_match_arms = enum_variant_names
        .iter()
        .map(|(name, enum_name)| {
            quote! { Icon::#enum_name => #name::IMAGE_DATA.to_iterator(), }
        })
        .collect::<TokenStream>();

    let enum_content = quote! {
        #[allow(missing_docs)]
        pub enum Icon {
            #enum_variant_decls
        }

        impl Icon {
            #[allow(missing_docs)]
            pub fn get_image<'a>(&self) -> ImageIterator<'a> {
                match &self {
                    #enum_match_arms
                }
            }
        }
    };

    let content = quote! {
        use crate::image::ImageIterator;

        #mods

        #enum_content
    };

    let prettyfied = RustFmt::new()
        .format_str(&content.to_string())
        .expect("Failed to format mod output");

    let mut file = fs::File::create(format!("{IMAGE_OUTPUT_DIR}{MOD_FILE_NAME}"))
        .expect("Failed to open/create mod.rs file");

    file.write_all(prettyfied.as_bytes())
        .expect("Failed to read mod.rs file content");
}

fn convert_image_width_to_byte_width(image_width: usize) -> usize {
    (image_width + ((8 - (image_width % 8)) % 8)) / 8
}

fn next_starting_with<'a>(lines: &'a mut Lines<'_>, prefix: &'static str) -> Result<&'a str, ()> {
    while let Some(l) = lines.next() {
        if l.starts_with(prefix) {
            return Ok(l);
        }
    }

    Err(())
}
