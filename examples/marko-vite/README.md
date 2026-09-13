# Apple App Clip Code with Marko.js & Vite

An example demonstrating how to integrate the pure WebAssembly Apple App Clip Code generator & decoder into a **Marko 6** application with **Vite 6/8** and `@marko/vite`.

---

## Features

- **Marko 6 Tags API**: Uses modern Marko 6 reactive primitives (`<let>`, `<const>`, `<lifecycle>`, `<if>`, `<for>`).
- **Reactive `<appclip-code>` Component**: Auto-discovered custom tag that renders bit-accurate Apple App Clip Code SVGs reactively.
- **`<appclip-decoder>` Component**: In-browser scanner that decodes both standard URL App Clip codes (v0/v1) and Version 8 optical binary tokens (Vision Pro ZEISS Rx, Apple Pay).
- **Fast Vite Bundling**: Full HMR in development and optimized production bundling with `@marko/vite` (`linked: false` for SPA mode).
- **Client-side PNG & SVG Export**: Direct download of vector SVG and 800px high-resolution raster PNG without external server dependencies.

---

## Project Structure

```
examples/marko-vite/
├── index.html                  # HTML entry point with mount target
├── vite.config.js              # Vite config using @marko/vite
├── package.json
├── public/
│   └── appclipcode_wasm.wasm   # Standalone WebAssembly binary (870 KB)
└── src/
    ├── main.js                 # Mounts App.marko to #app
    ├── style.css               # Shared styles
    ├── lib/
    │   └── appclipcode.js      # ES module wrapper for the WASM engine
    ├── tags/
    │   ├── appclip-code.marko  # Reusable App Clip Code renderer
    │   └── appclip-decoder.marko # SVG Decoder / Scanner component
    └── App.marko               # Main application template
```

---

## Quick Start

### 1. Install Dependencies

```bash
cd examples/marko-vite
npm install
```

### 2. Run Development Server

```bash
npm run dev
```

Open [http://localhost:5173](http://localhost:5173) in your browser.

### 3. Build for Production

```bash
npm run build
```

The production-ready assets will be generated in `dist/`. You can preview the build with:

```bash
npm run preview
```

---

## Usage in Marko Components

### Rendering an App Clip Code

Simply drop the `<appclip-code>` tag into any `.marko` file:

```marko
<appclip-code
  url="https://example.com"
  template=0
  codeType="cam"
  size=300
/>
```

#### Component Properties (`input`):

| Property | Type | Default | Description |
|---|---|---|---|
| `url` | `string` | `"https://example.com"` | URL to encode (must start with `https://`) |
| `template` | `number` | `0` | Preset color palette index (`0` to `17`) |
| `codeType` | `string` | `"cam"` | Code design: `"cam"` (Camera center logo) or `"nfc"` (NFC phone icon) |
| `size` | `number` | `340` | Display width in pixels |

### Decoding an SVG Barcode

Use the `<appclip-decoder>` tag:

```marko
<appclip-decoder />
```

Or call the decoder API directly from JavaScript:

```javascript
import { init, read_svg } from "./lib/appclipcode.js";

await init();
const payload = read_svg(svgXmlString);
// For v0/v1: returns "https://example.com"
// For v8:    returns "a3e637e080069b1c344007b4df14f288752b3407"
```

---

## How it Works

1. **WASM Loading**:
   The WebAssembly module is located in `public/appclipcode_wasm.wasm`. Vite serves it statically, and [`src/lib/appclipcode.js`](./src/lib/appclipcode.js) initializes the module using `WebAssembly.instantiateStreaming` (or fallback buffer instantiation).

2. **Marko Lifecycle**:
   Components use `<lifecycle onMount() { ... } onUpdate() { ... } />` to ensure the WebAssembly engine is initialized before calling `generate_svg()` or `read_svg()`.

3. **SVG Rendering**:
   The raw SVG returned by WASM is injected into the DOM via Marko's unescaped HTML placeholder `$!{svg}`, which compiles to Marko's high-performance DOM update routines.
