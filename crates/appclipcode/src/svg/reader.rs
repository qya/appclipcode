use crate::codec::{decode_barcode, hex_encode, DecodedBarcode};
use super::renderer::{CENTER_X, CENTER_Y, DEG_2_RAD, RING_BIT_COUNTS, RING_GAP_ANGLES, RING_RADII};

pub fn read_svg(svg_data: &str) -> Result<String, String> {
    let barcode = read_svg_barcode(svg_data)?;
    if let Some(url) = barcode.url {
        Ok(url)
    } else {
        Ok(hex_encode(&barcode.payload))
    }
}

pub fn read_svg_barcode(svg_data: &str) -> Result<DecodedBarcode, String> {
    let bits = extract_bits(svg_data)?;
    decode_barcode(&bits)
}

pub fn extract_bits(svg: &str) -> Result<Vec<bool>, String> {
    let mut gap_bits = vec![true; 128];
    let mut color_bits = Vec::new();
    let mut offset = 0;

    for ri in 0..5 {
        let r = RING_RADII[ri];
        let n = RING_BIT_COUNTS[ri];
        let ba = 360.0 / (n as f64);
        let hg = RING_GAP_ANGLES[ri];

        #[derive(Clone, Copy)]
        struct Pt {
            x: f64,
            y: f64,
        }

        let mut starts = Vec::with_capacity(n);
        for i in 0..n {
            let a0 = ((i as f64) * ba + hg) * DEG_2_RAD;
            starts.push(Pt {
                x: CENTER_X + r * a0.cos(),
                y: CENTER_Y + r * a0.sin(),
            });
        }

        let snap = |px: f64, py: f64, pts: &[Pt]| -> usize {
            let mut best = 0;
            let mut best_d = f64::MAX;
            for (i, p) in pts.iter().enumerate() {
                let d = (px - p.x) * (px - p.x) + (py - p.y) * (py - p.y);
                if d < best_d {
                    best_d = d;
                    best = i;
                }
            }
            best
        };

        let ring_tag = format!("name=\"ring-{}\"", ri + 1);
        let idx = svg.find(&ring_tag)
            .ok_or_else(|| format!("ring {} not found in SVG", ri + 1))?;
        let end = svg[idx..].find("</g>")
            .ok_or_else(|| format!("ring {} closing tag not found", ri + 1))?;
        let chunk = &svg[idx..idx + end];

        struct ArcInfo {
            start_pos: usize,
            color: usize,
        }
        let mut ring_arcs: Vec<ArcInfo> = Vec::new();

        // Parse path tags in chunk
        for path_segment in chunk.split("<path ") {
            if !path_segment.contains("d=\"") {
                continue;
            }

            // Extract d attribute
            let d_start = match path_segment.find("d=\"") {
                Some(p) => p + 3,
                None => continue,
            };
            let d_end = match path_segment[d_start..].find('"') {
                Some(p) => d_start + p,
                None => continue,
            };
            let d_val = &path_segment[d_start..d_end];

            // Extract data-color attribute
            let color = if let Some(c_start) = path_segment.find("data-color=\"") {
                let rest = &path_segment[c_start + "data-color=\"".len()..];
                if let Some(c_end) = rest.find('"') {
                    rest[..c_end].parse::<usize>().unwrap_or(0)
                } else {
                    0
                }
            } else {
                0
            };

            // Parse d format: "M ex ey A rx ry 0 largeArc 0 sx sy"
            // sx is parts[9], sy is parts[10]
            let parts: Vec<&str> = d_val.split_whitespace().collect();
            if parts.len() >= 11 && parts[0] == "M" && parts[3] == "A" {
                if let (Ok(ax), Ok(ay)) = (parts[9].parse::<f64>(), parts[10].parse::<f64>()) {
                    let start_pos = snap(ax, ay, &starts);
                    ring_arcs.push(ArcInfo { start_pos, color });
                }
            }
        }


        for a in ring_arcs {
            gap_bits[offset + a.start_pos] = false;
            color_bits.push(a.color == 1);
        }

        offset += n;
    }

    let mut result = Vec::with_capacity(128 + color_bits.len());
    result.extend_from_slice(&gap_bits);
    result.extend_from_slice(&color_bits);
    Ok(result)
}
