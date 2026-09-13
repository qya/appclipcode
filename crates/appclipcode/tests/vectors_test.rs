use appclipcode::{
    compress_url, decompress_url, generate, generate_with_template, read_svg,
    CodeType, Options,
};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

fn find_testdata_file(relative_path: &str) -> Option<PathBuf> {
    let candidates = [
        Path::new(relative_path).to_path_buf(),
        Path::new("../../").join(relative_path),
        Path::new("testdata").join(relative_path),
        Path::new("../../testdata").join(relative_path),
    ];
    for c in candidates {
        if c.exists() {
            return Some(c);
        }
    }
    None
}

#[test]
fn test_apple_compare_bytes() {
    let tests = [
        ("https://example.com", "0000000000000000000000008e33db36"),
        ("https://a.co", "00000000000000000000000000004e2f"),
        ("https://www.apple.com", "000000000000000000000478a3cee226"),
        ("https://example.com/path", "000000000000000000238cf6cdb5d582"),
        ("https://appclip.example.com", "000000000000000000000000ae33db36"),
        ("https://app.nextdns.io?p=", "0000000000000000d02424a9f120a6e4"),
        ("https://app.nextdns.io?p=a", "000000000000001a0484953e2414dc85"),
        ("https://app.nextdns.io?p=a&p1=b", "000000000001a0484953e2414dc85031"),
        ("https://app.nextdns.io?p=a&", "000000000902424a9f120a6e3ccda046"),
        ("https://app.nextdns.io?p=abcdef", "000000003409092a7c4829b90b858ed5"),
        ("https://app.nextdns.io?&p=a&&p1=b", "000000000001a0484953e2414dc85031"),
        ("https://app.nextdns.io?x=a", "0000000000902424a9f120a6e720cea5"),
        ("https://app.nextdns.io/bag", "000000000000003409092a7c4829b809"),
        ("https://app.nextdns.io/biz", "000000000000003409092a7c4829b80a"),
        ("https://app.nextdns.io/cat", "000000000000003409092a7c4829b812"),
        ("https://app.nextdns.io/shop?p=a", "0000000000003409092a7c4829b87785"),
        ("https://app.nextdns.io/use", "000000000000003409092a7c4829b894"),
        ("https://app.nextdns.io/a?p=abcdef", "00048121254f89053722fb320b858ed5"),
        ("https://app.nextdns.io/id?p=a&p1=b", "0000000003409092a7c4829b83e85031"),
        ("https://app.nextdns.io/id?p=abcdef", "00000068121254f89053707d0b858ed5"),
        ("https://qr.netflix.com/C/AAAA", "0000000116d5992f39664d1bfa2cb2cb"),
    ];

    for (url, expected_hex) in tests {
        let compressed = compress_url(url).unwrap_or_else(|e| panic!("Failed {}: {}", url, e));
        let actual_hex: String = compressed.iter().map(|b| format!("{:02x}", b)).collect();
        assert_eq!(
            actual_hex, expected_hex,
            "URL compression mismatch for {}",
            url
        );

        let decompressed = decompress_url(&compressed).unwrap();
        let norm_decomp = decompressed.replace("/?", "?");
        let norm_url = url.replace("/?", "?").replace("&&", "&").replace("?&", "?");
        assert_eq!(norm_decomp, norm_url, "Decompress mismatch for {}", url);
    }
}

#[derive(Deserialize)]
struct VectorEntry {
    url: String,
    file: Option<String>,
}

#[test]
fn test_comprehensive_vectors_roundtrip() {
    let file = find_testdata_file("comprehensive_vectors.json").expect("comprehensive_vectors.json not found");
    let data = fs::read_to_string(&file).unwrap();
    let entries: Vec<VectorEntry> = serde_json::from_str(&data).unwrap();
    assert_eq!(entries.len(), 126);

    for (i, entry) in entries.iter().enumerate() {
        let svg = generate_with_template(&entry.url, 0, None)
            .unwrap_or_else(|e| panic!("Failed to generate SVG for vector #{}: {}: {}", i, entry.url, e));

        let decoded_url = read_svg(&svg)
            .unwrap_or_else(|e| panic!("Failed to read SVG for vector #{}: {}: {}", i, entry.url, e));

        assert_eq!(
            decoded_url, entry.url,
            "Vector #{} round-trip mismatch: got {}, want {}",
            i, decoded_url, entry.url
        );
    }
}

#[test]
fn test_apple_reference_svg_decoding() {
    let file = find_testdata_file("comprehensive_vectors.json").expect("comprehensive_vectors.json not found");
    let data = fs::read_to_string(&file).unwrap();
    let entries: Vec<VectorEntry> = serde_json::from_str(&data).unwrap();

    let comp_dir = find_testdata_file("apple_comprehensive").expect("apple_comprehensive dir not found");

    for (i, entry) in entries.iter().enumerate() {
        if let Some(ref svg_filename) = entry.file {
            let svg_path = comp_dir.join(svg_filename);
            if !svg_path.exists() {
                continue;
            }
            let apple_svg = fs::read_to_string(&svg_path).unwrap();
            let decoded_url = read_svg(&apple_svg)
                .unwrap_or_else(|e| panic!("Failed to read Apple reference SVG #{}: {}: {}", i, svg_filename, e));

            assert_eq!(
                decoded_url, entry.url,
                "Apple reference SVG #{} mismatch: got {}, want {}",
                i, decoded_url, entry.url
            );
        }
    }
}

#[test]
fn test_random_vectors_roundtrip() {
    let file = find_testdata_file("random_vectors.json").expect("random_vectors.json not found");
    let data = fs::read_to_string(&file).unwrap();
    let entries: Vec<VectorEntry> = serde_json::from_str(&data).unwrap();
    assert_eq!(entries.len(), 94);

    for (i, entry) in entries.iter().enumerate() {
        let compressed = compress_url(&entry.url)
            .unwrap_or_else(|e| panic!("Failed to compress {}: {}", entry.url, e));
        let decompressed = decompress_url(&compressed)
            .unwrap_or_else(|e| panic!("Failed to decompress {}: {}", entry.url, e));

        assert_eq!(
            decompressed, entry.url,
            "Random vector #{} mismatch: got {}, want {}",
            i, decompressed, entry.url
        );
    }
}

#[test]
fn test_svg_structure_and_nfc() {
    let svg_cam = generate("https://example.com", "FFFFFF", "000000", None).unwrap();
    assert!(svg_cam.contains("data-design=\"Fingerprint\""));
    assert!(svg_cam.contains("data-payload=\"https://example.com\""));
    assert!(svg_cam.contains("viewBox=\"0 0 800 800\""));
    assert!(svg_cam.contains("data-logo-type=\"Camera\""));
    assert!(svg_cam.contains("stroke:#ffffff"));
    assert!(svg_cam.contains("fill:#000000"));

    let svg_nfc = generate(
        "https://example.com",
        "FFFFFF",
        "000000",
        Some(Options {
            code_type: CodeType::NFC,
        }),
    )
    .unwrap();
    assert!(svg_nfc.contains("data-logo-type=\"phone\""));
}

#[test]
fn test_png_generation() {
    #[cfg(feature = "png")]
    {
        let png = appclipcode::generate_png_with_template("https://example.com", 0, None, Some(400))
            .unwrap();
        assert!(!png.is_empty());
        assert_eq!(&png[0..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
    }
}

#[test]
fn test_version_8_decoding_issue8() {
    let file = find_testdata_file("issue8/barcode1.svg").expect("issue8/barcode1.svg not found");
    let svg = fs::read_to_string(&file).unwrap();

    let hex_result = read_svg(&svg).expect("Failed to decode issue8/barcode1.svg");
    assert_eq!(hex_result, "a3e637e080069b1c344007b4df14f288752b3407");

    let barcode = appclipcode::read_svg_barcode(&svg).expect("Failed to parse barcode");
    assert_eq!(barcode.version, 8);
    assert_eq!(barcode.inverted, false);
    assert_eq!(barcode.payload.len(), 20);
    assert_eq!(barcode.url, None);
    assert_eq!(appclipcode::hex_encode(&barcode.payload), "a3e637e080069b1c344007b4df14f288752b3407");
}

#[test]
fn test_version_8_decoding_vision_pro_rx() {
    let file = find_testdata_file("issue1/barcode1.svg").expect("issue1/barcode1.svg not found");
    let svg = fs::read_to_string(&file).unwrap();

    let hex_result = read_svg(&svg).expect("Failed to decode issue1/barcode1.svg");
    assert_eq!(hex_result, "91529bc49025fa01d91e9ef9e56125db48ddaf58");

    let barcode = appclipcode::read_svg_barcode(&svg).expect("Failed to parse barcode");
    assert_eq!(barcode.version, 8);
    assert_eq!(barcode.inverted, false);
    assert_eq!(barcode.payload.len(), 20);
    assert_eq!(barcode.url, None);
    assert_eq!(appclipcode::hex_encode(&barcode.payload), "91529bc49025fa01d91e9ef9e56125db48ddaf58");
}

#[test]
fn test_version_8_decoding_apple_pay() {
    let file = find_testdata_file("issue1/barcode2.svg").expect("issue1/barcode2.svg not found");
    let svg = fs::read_to_string(&file).unwrap();

    let hex_result = read_svg(&svg).expect("Failed to decode issue1/barcode2.svg");
    assert_eq!(hex_result, "c2723a51b050adcb9de356ecd09b67f8f21abbb1");

    let barcode = appclipcode::read_svg_barcode(&svg).expect("Failed to parse barcode");
    assert_eq!(barcode.version, 8);
    assert_eq!(barcode.inverted, false);
    assert_eq!(barcode.payload.len(), 20);
    assert_eq!(barcode.url, None);
    assert_eq!(appclipcode::hex_encode(&barcode.payload), "c2723a51b050adcb9de356ecd09b67f8f21abbb1");
}


