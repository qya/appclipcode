# appclipcode-cli (`appclipcodegen`)

Command-line tool to generate and scan Apple App Clip Codes (SVG and PNG).

## Installation

```bash
cargo install --path crates/appclipcode-cli
```

Or run directly from the workspace:

```bash
cargo run -p appclipcode-cli -- --help
```

## Commands & Usage

### 1. Generate App Clip Code

```bash
# Generate default template (Template 0: white on black)
appclipcodegen gen https://example.com -o code.svg

# Generate with a preset template (0 to 17)
appclipcodegen gen https://example.com --index 4 -o code.svg

# Generate with custom hex colors
appclipcodegen gen https://example.com --fg 007AFF --bg FFFFFF -o code.svg

# Generate NFC code instead of Camera code
appclipcodegen gen https://example.com --type nfc -o code.svg

# Generate 512x512 PNG
appclipcodegen gen https://example.com -o code.png --size 512
```

### 2. Scan / Decode SVG

```bash
# Decode URL or payload directly
appclipcodegen scan code.svg

# Detailed metadata inspection (version, inverted polarity, raw hex bytes, URL)
appclipcodegen scan testdata/issue8/barcode1.svg --info
```

### 3. List Templates

```bash
appclipcodegen templates
```
