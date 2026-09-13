# App Clip Code - Vanilla HTML & WebAssembly Demo

A lightweight, zero-dependency browser interface demonstrating real-time Apple App Clip Code generation and optical barcode decoding powered by WebAssembly.

## Features

- **Zero Build Step**: Pure HTML5, CSS3, and ES modules. No Node.js, Webpack, or bundler required to run.
- **Instant Generation**: Compresses URLs and renders bit-accurate circular SVG codes directly in the browser thread in microseconds.
- **Color Customization**:
  - 18 Apple preset color templates (Template 0 to 17).
  - Custom 6-digit hex foreground and background color inputs with automatic optical third-color computation.
- **Style Toggle**: Switch between standard Camera central glyph and NFC phone badge glyph.
- **Export Options**:
  - One-click vector **SVG** download.
  - High-resolution 800×800 **PNG** client-side rasterization and download.
- **In-Browser SVG Decoder**:
  - Drag-and-drop or upload any App Clip Code `.svg`.
  - Automatically decodes both standard URL codes (v0/v1) and Version 8 optical binary tokens (Apple Pay, Vision Pro ZEISS Optical Inserts, and camera pairing).

---

## Quick Start

Serve this directory using any static web server:

### Using Python 3

```bash
cd examples/html
python3 -m http.server 8080
```

### Using Node.js (`npx serve`)

```bash
cd examples/html
npx serve .
```

Open [http://localhost:8080](http://localhost:8080) in your browser.

---

## File Structure

```
examples/html/
├── index.html            # Main UI, styling, and client-side application logic
├── appclipcode.js        # ES module wrapper for WebAssembly memory and exports
├── appclipcode_wasm.wasm # Precompiled standalone WebAssembly binary (~850 KB)
└── README.md             # This documentation
```

---

## JavaScript API Usage

You can import and use `appclipcode.js` in any web application:

```javascript
import {
  init,
  generate_svg,
  generate_custom_svg,
  read_svg,
  get_templates,
} from "./appclipcode.js";

// 1. Initialize the WASM binary
await init("./appclipcode_wasm.wasm");

// 2. Generate SVG with a preset template (0-17) and code type ('cam' or 'nfc')
const svg = generate_svg("https://example.com", 0, "cam");

// 3. Generate SVG with custom hex colors
const customSvg = generate_custom_svg("https://example.com", "FFFFFF", "007AFF", "nfc");

// 4. Decode an SVG string (returns URL or hex payload)
const decoded = read_svg(svg);
console.log("Decoded:", decoded);

// 5. Query all 18 preset templates
const templates = get_templates();
```

---

## Rebuilding the WASM Module

To recompile the WebAssembly binary from the Rust source:

```bash
# From repository root:
./build-wasm.sh
```

This compiles `crates/appclipcode-wasm` with `--target wasm32-unknown-unknown --release` and automatically copies the updated `appclipcode_wasm.wasm` here.
