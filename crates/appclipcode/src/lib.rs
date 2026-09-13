pub mod color;
pub mod codec;
pub mod svg;
#[cfg(feature = "png")]
pub mod raster;

pub use color::{find_third_color, parse_hex_color, template_by_index, templates, Color, Palette, Template};
pub use codec::{
    compress_url, decode_barcode, decode_payload, decompress_url, encode_payload, hex_encode,
    DecodedBarcode,
};
pub use svg::{extract_bits, read_svg, read_svg_barcode, render_svg, CodeType};

#[cfg(feature = "png")]
pub use raster::render_png;

#[derive(Debug, Clone, Copy, Default)]
pub struct Options {
    pub code_type: CodeType,
}

pub fn generate(
    raw_url: &str,
    foreground: &str,
    background: &str,
    opts: Option<Options>,
) -> Result<String, String> {
    let fg = parse_hex_color(foreground).map_err(|e| format!("foreground: {}", e))?;
    let bg = parse_hex_color(background).map_err(|e| format!("background: {}", e))?;
    let third = find_third_color(fg, bg);

    let pal = Palette {
        foreground: fg,
        background: bg,
        third,
    };

    generate_with_palette(raw_url, pal, opts)
}

pub fn generate_with_template(
    raw_url: &str,
    template_index: usize,
    opts: Option<Options>,
) -> Result<String, String> {
    let pal = template_by_index(template_index)?;
    generate_with_palette(raw_url, pal, opts)
}

pub fn generate_with_palette(
    raw_url: &str,
    pal: Palette,
    opts: Option<Options>,
) -> Result<String, String> {
    let code_type = opts.unwrap_or_default().code_type;

    // Step 1: Compress URL to 16 bytes
    let compressed = compress_url(raw_url).map_err(|e| format!("compress URL: {}", e))?;

    // Step 2: Encode payload to bits
    let all_bits = encode_payload(&compressed).map_err(|e| format!("encode payload: {}", e))?;

    // Step 3: Render SVG
    let svg = render_svg(&all_bits, &pal, raw_url, code_type);
    Ok(svg)
}

#[cfg(feature = "png")]
pub fn generate_png(
    raw_url: &str,
    foreground: &str,
    background: &str,
    opts: Option<Options>,
    size: Option<u32>,
) -> Result<Vec<u8>, String> {
    let svg = generate(raw_url, foreground, background, opts)?;
    render_png(&svg, size)
}

#[cfg(feature = "png")]
pub fn generate_png_with_template(
    raw_url: &str,
    template_index: usize,
    opts: Option<Options>,
    size: Option<u32>,
) -> Result<Vec<u8>, String> {
    let svg = generate_with_template(raw_url, template_index, opts)?;
    render_png(&svg, size)
}
