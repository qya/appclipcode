# appclipcode-wasm

Standalone WebAssembly bindings for the Apple App Clip Code generator and decoder.

## Overview

- **C ABI exports**: Simple pointer and buffer-based ABI without `wasm-bindgen` bloat or complex JS glue.
- **Zero npm dependencies**: Pure WebAssembly module runnable directly in any browser or WebAssembly runtime.
- **Size**: ~850 KB release binary.

## Building

From the workspace root:

```bash
./build-wasm.sh
```

Or manually:

```bash
cargo build -p appclipcode-wasm --target wasm32-unknown-unknown --release
```

The resulting binary will be in `target/wasm32-unknown-unknown/release/appclipcode_wasm.wasm`.

## Exported Functions

- `appclip_generate_svg(url_ptr, url_len, template_index, code_type)`
- `appclip_generate_svg_custom(url_ptr, url_len, fg_ptr, fg_len, bg_ptr, bg_len, code_type)`
- `appclip_read_svg(svg_ptr, svg_len)`
- `appclip_templates_count()`
- `appclip_get_template(index, fg_ptr, bg_ptr, third_ptr)`
- Memory management: `appclip_alloc`, `appclip_dealloc`, `appclip_free_buffer`, `appclip_get_last_error`
