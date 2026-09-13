# appclipcode

Pure Rust encoder and decoder for Apple App Clip Codes with 100% bit-accurate fidelity.

## Features

- **Multi-Context URL Compression**: Huffman coding, TLD tables, and path wordbooks bit-identical to Apple's implementation.
- **Trie Precomputation**: Efficiently serialized dictionary trie packed into 622 KB via preorder bitpacking and miniz_oxide zlib.
- **Reed-Solomon Codec**: Systematic encoder and decoder over GF(16) and GF(256) with error correction.
- **Dual-Version Decoding**:
  - Version 0 & 1: Standard URL App Clip codes.
  - Version 8: 160-bit (20-byte) raw cryptographic optical tokens (Vision Pro ZEISS Rx, Apple Pay, camera pairing).
- **SVG & Raster Rendering**: Exact vector path generation with Camera and NFC styles, plus optional PNG rasterization via `resvg`/`tiny-skia`.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
appclipcode = { version = "0.1.0", path = "../appclipcode", features = ["png"] }
```

## Cargo Features

- `default = []`: Core library (URL compressor/decompressor, RS codec, SVG generator and decoder).
- `png`: Enables `render_png` and `generate_png` rasterization using pure Rust `resvg` and `tiny-skia`.

## Quick Example

```rust
use appclipcode::{generate_with_template, read_svg, read_svg_barcode, CodeType, Options};

fn main() -> Result<(), String> {
    // Generate an SVG
    let svg = generate_with_template("https://example.com", 0, None)?;

    // Decode URL
    let url = read_svg(&svg)?;
    assert_eq!(url, "https://example.com");

    // Inspect barcode structure
    let barcode = read_svg_barcode(&svg)?;
    println!("Version: {}, Payload len: {}", barcode.version, barcode.payload.len());

    Ok(())
}
```
