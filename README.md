# App Clip Code in Pure Rust & WebAssembly

A pure Rust and WebAssembly implementation of Apple's App Clip Code generator and decoder, reverse-engineered for 100% bit-accurate fidelity with Apple's format.

## Features

- **Bit-identical URL compression**: Reproduces Apple's multi-context Huffman coding, TLD tables, and path wordbooks for accepted `https://` URLs.
- **High-density trie compression**: Compact bitpacked preorder tree with zlib compression (1.7 MB source data compressed to ~622 KB binary footprint).
- **Reed-Solomon error correction**: Full systematic encoder and decoder over GF(16) and GF(256) with single and double error correction.
- **Optical & payload decoder**: Decodes both standard URL App Clip codes (Version 0 & 1) and Version 8 optical binary tokens (160-bit / 20-byte cryptographic credentials used in Apple Vision Pro ZEISS Optical Inserts, Apple Pay, and camera pairing).
- **Circular vector rendering**: Generates exact 5-ring fingerprint SVGs with preset templates (0–17) or custom colors, supporting Camera and NFC styles.
- **Native SVG & PNG export**: High-performance SVG generation and pure Rust rasterization to PNG via `resvg`/`tiny-skia` with configurable resolutions.
- **Zero-dependency WebAssembly**: Lean, standalone WebAssembly binary (850 KB) running client-side in browsers, Node.js, and edge runtimes without Node.js or C dependencies.
- **Bidirectional**: Complete end-to-end support for generating, rendering, parsing, and decoding App Clip Codes.

---

## Workspace Layout

```
.
├── crates/
│   ├── appclipcode/        # Core codec, trie, Reed-Solomon, SVG renderer, and SVG decoder
│   ├── appclipcode-cli/    # Native CLI binary `appclipcodegen` (SVG/PNG generator & scanner)
│   └── appclipcode-wasm/   # Standalone WebAssembly bindings (C ABI / zero npm dependencies)
├── examples/
│   ├── html/               # Interactive vanilla HTML5 / ES module browser demo
│   └── marko-vite/         # Modern Marko 6 (.marko) + Vite 6/8 reactive demo application
├── testdata/               # 126 Apple reference SVGs, 94 random vectors, and Version 8 fixtures
└── build-wasm.sh           # One-step script to build WASM and sync with examples
```

---

## Getting Started

### CLI Tool (`appclipcodegen`)

#### Build & Run

```bash
cargo build --release -p appclipcode-cli
./target/release/appclipcodegen --help
```

#### Generate SVG

```bash
# Generate using default template (Template 0: white on black)
cargo run -p appclipcode-cli -- gen https://example.com -o code.svg

# Generate with a preset template (0-17)
cargo run -p appclipcode-cli -- gen https://example.com --index 4 -o code.svg

# Generate with custom colors (hex)
cargo run -p appclipcode-cli -- gen https://example.com --fg 007AFF --bg FFFFFF -o code.svg

# Generate NFC code instead of Camera code
cargo run -p appclipcode-cli -- gen https://example.com --type nfc -o code.svg
```

#### Generate PNG

```bash
# Generate a high-resolution 512x512 PNG
cargo run -p appclipcode-cli -- gen https://example.com -o code.png --size 512
```

#### Scan / Decode SVG

Scan an App Clip Code SVG to extract the URL or raw payload:

```bash
cargo run -p appclipcode-cli -- scan code.svg
# Output: https://example.com
```

Use `--info` (or `-v`) for detailed metadata inspection (version, inverted polarity, raw hex payload, decompressed URL):

```bash
cargo run -p appclipcode-cli -- scan testdata/issue8/barcode1.svg --info
```
Output:
```text
Version:  8
Inverted: false
Payload:  a3e637e080069b1c344007b4df14f288752b3407 (20 bytes)
```

#### List Available Templates

```bash
cargo run -p appclipcode-cli -- templates
```

---

### Rust Library Usage

Add `appclipcode` to your `Cargo.toml`:

```toml
[dependencies]
appclipcode = { path = "crates/appclipcode", features = ["png"] }
```

#### Encoding & Rendering

```rust
use appclipcode::{
    generate, generate_with_template, render_png,
    CodeType, Options, Palette, Color,
};

fn main() -> Result<(), String> {
    // 1. Generate SVG with a preset template (0-17)
    let svg = generate_with_template("https://example.com", 0, None)?;

    // 2. Generate SVG with custom colors & NFC center icon
    let custom_svg = generate(
        "https://example.com",
        "FFFFFF", // foreground
        "007AFF", // background
        Some(Options { code_type: CodeType::NFC }),
    )?;

    // 3. Render SVG directly to PNG bytes (requires `png` feature)
    let png_bytes = render_png(&svg, 800)?;
    std::fs::write("code.png", png_bytes).map_err(|e| e.to_string())?;

    Ok(())
}
```

#### Decoding SVGs & Optical Barcodes

```rust
use appclipcode::{read_svg, read_svg_barcode};

fn main() -> Result<(), String> {
    let svg_content = std::fs::read_to_string("code.svg").unwrap();

    // High-level: Returns URL string for v0/v1, or hex string for v8
    let result = read_svg(&svg_content)?;
    println!("Result: {}", result);

    // Structured: Inspect full barcode details
    let barcode = read_svg_barcode(&svg_content)?;
    println!("Version: {}", barcode.version);       // 0, 1, or 8
    println!("Inverted: {}", barcode.inverted);     // true if inverted polarity
    println!("Payload bytes: {:02x?}", barcode.payload);
    if let Some(url) = barcode.url {
        println!("Decompressed URL: {}", url);
    }

    Ok(())
}
```

---

### WebAssembly

The WebAssembly build produces a lean standalone `.wasm` module with a clean C ABI, requiring zero npm dependencies.

#### Build WebAssembly Module

Run the build script to compile the WASM binary and distribute it to both examples:

```bash
./build-wasm.sh
```

This generates `target/wasm32-unknown-unknown/release/appclipcode_wasm.wasm` (~850 KB) and copies it into `examples/html/` and `examples/marko-vite/public/`.

#### Running the Interactive Demos

1. **Vanilla HTML / JS Demo**:
   ```bash
   cd examples/html
   python3 -m http.server 8080
   ```
   Open [http://localhost:8080](http://localhost:8080) to interactively generate codes, switch palettes, customize colors, download SVG/PNG, and drag-and-drop SVGs to decode.

2. **Marko 6 + Vite 8 Demo**:
   ```bash
   cd examples/marko-vite
   npm install
   npm run dev
   ```
   Open [http://localhost:5173](http://localhost:5173) for a reactive web application featuring reusable `<appclip-code>` and `<appclip-decoder>` custom tags.

---

## Testing & Verification

Run the comprehensive test suite:

```bash
cargo test --workspace --features png
```

Verification includes:
- 126 official Apple reference SVGs verified bit-for-bit.
- 94 randomized end-to-end roundtrip test vectors.
- Version 8 optical barcode decoding tests (Apple Pay, Vision Pro ZEISS Optical Inserts, and camera pairing codes).
- Full SVG parsing and Reed-Solomon error correction validation.
