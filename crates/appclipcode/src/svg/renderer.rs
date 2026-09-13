use std::f64::consts::PI;
use crate::color::Palette;
use super::assets::{CAMERA_LOGO_PATHS, PHONE_OUTER_PATH, PHONE_SCREEN_PATH};

pub const RING_RADII: [f64; 5] = [177.2016, 224.1012, 271.0008, 317.9004, 364.8];
pub const RING_ROTATIONS: [f64; 5] = [-78.0, -85.0, -70.0, -63.0, -70.0];
pub const RING_BIT_COUNTS: [usize; 5] = [17, 23, 26, 29, 33];
pub const RING_GAP_ANGLES: [f64; 5] = [7.5, 5.6, 5.0, 4.2, 3.5];

pub const CENTER_X: f64 = 400.0;
pub const CENTER_Y: f64 = 400.0;
pub const BG_RADIUS: f64 = 400.0;
pub const STROKE_WIDTH: f64 = 23.5;
pub const DEG_2_RAD: f64 = PI / 180.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CodeType {
    #[default]
    Camera,
    NFC,
}

impl CodeType {
    pub fn from_str_opt(s: Option<&str>) -> Self {
        match s {
            Some("nfc") | Some("NFC") => CodeType::NFC,
            _ => CodeType::Camera,
        }
    }
}

pub fn render_svg(bits: &[bool], pal: &Palette, url: &str, code_type: CodeType) -> String {
    let mut sb = String::with_capacity(16 * 1024);

    sb.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    sb.push_str(&format!(
        "<svg data-design=\"Fingerprint\" data-payload=\"{}\" viewBox=\"0 0 800 800\" xmlns=\"http://www.w3.org/2000/svg\">\n",
        escape_xml(url)
    ));
    sb.push_str("    <title>App Clip Code</title>\n");

    // Background circle
    sb.push_str(&format!(
        "    <circle cx=\"{:.6}\" cy=\"{:.6}\" id=\"Background\" r=\"{:.6}\" style=\"fill:{}\"/>\n",
        CENTER_X, CENTER_Y, BG_RADIUS, pal.background.hex()
    ));

    // Markers (5 rings)
    sb.push_str("    <g id=\"Markers\">\n");

    let gap_bits = &bits[..128];
    let color_stream = if bits.len() > 128 { &bits[128..] } else { &[] };

    let mut gap_offset = 0;
    let mut color_idx = 0;

    for ring in 0..5 {
        let n = RING_BIT_COUNTS[ring];
        let ring_gap = &gap_bits[gap_offset..gap_offset + n];
        gap_offset += n;

        // Build per-position state: -1 = invisible, 0 = foreground, 1 = third color
        let mut pos_state = vec![0isize; n];
        for i in 0..n {
            if !ring_gap[i] {
                let mut color = 0;
                if color_idx < color_stream.len() && color_stream[color_idx] {
                    color = 1;
                }
                pos_state[i] = color;
                color_idx += 1;
            } else {
                pos_state[i] = -1;
            }
        }

        sb.push_str(&format!(
            "        <g name=\"ring-{}\" transform=\"rotate({:.0} {:.0} {:.0})\">\n",
            ring + 1, RING_ROTATIONS[ring], CENTER_X, CENTER_Y
        ));

        write_ring_arcs_from_state(&mut sb, ring, &pos_state, pal);

        sb.push_str("        </g>\n");
    }
    sb.push_str("    </g>\n");

    // Center logo
    write_logo(&mut sb, code_type, pal);

    sb.push_str("</svg>\n");

    sb
}

struct ArcSegment {
    data_color: isize,
    start_bit: usize,
    count: usize,
}

fn write_ring_arcs_from_state(sb: &mut String, ring_idx: usize, pos_state: &[isize], pal: &Palette) {
    let n = RING_BIT_COUNTS[ring_idx];
    let radius = RING_RADII[ring_idx];
    let bit_angle = 360.0 / (n as f64);
    let gap_angle = RING_GAP_ANGLES[ring_idx];

    let mut arcs: Vec<ArcSegment> = Vec::new();
    for i in 0..n {
        if pos_state[i] == -1 {
            continue;
        }
        let mut span = 1;
        while i + span < n && pos_state[i + span] == -1 {
            span += 1;
        }
        if i + span == n {
            for j in 0..n {
                if pos_state[j] == -1 {
                    span += 1;
                } else {
                    break;
                }
            }
        }
        arcs.push(ArcSegment {
            data_color: pos_state[i],
            start_bit: i,
            count: span,
        });
    }

    for a in arcs {
        let start_angle = (a.start_bit as f64) * bit_angle + gap_angle;
        let end_angle = ((a.start_bit + a.count) as f64) * bit_angle - gap_angle;

        let sx = CENTER_X + radius * (start_angle * DEG_2_RAD).cos();
        let sy = CENTER_Y + radius * (start_angle * DEG_2_RAD).sin();
        let ex = CENTER_X + radius * (end_angle * DEG_2_RAD).cos();
        let ey = CENTER_Y + radius * (end_angle * DEG_2_RAD).sin();

        let mut arc_span = end_angle - start_angle;
        if arc_span < 0.0 {
            arc_span += 360.0;
        }
        let large_arc = if arc_span > 180.0 { 1 } else { 0 };

        let stroke_color = if a.data_color == 1 {
            pal.third
        } else {
            pal.foreground
        };

        sb.push_str(&format!(
            "            <path d=\"M {:.6} {:.6} A {:.6} {:.6} 0 {} 0 {:.6} {:.6}\" data-color=\"{}\" style=\"fill:none;stroke:{};stroke-linecap:round;stroke-miterlimit:10;stroke-width:{:.6}px\"/>\n",
            ex, ey, radius, radius, large_arc, sx, sy, a.data_color, stroke_color.hex(), STROKE_WIDTH
        ));
    }
}

fn write_logo(sb: &mut String, code_type: CodeType, pal: &Palette) {
    match code_type {
        CodeType::NFC => {
            sb.push_str("    <g id=\"Logo\" data-logo-type=\"phone\" transform=\"translate(293.400000 293.400000) scale(1.980000 1.980000)\">\n");
            sb.push_str(&format!(
                "        <path id=\"outer_circle\" d=\"{}\" style=\"fill:{}\"/>\n",
                PHONE_OUTER_PATH, pal.foreground.hex()
            ));
            sb.push_str(&format!(
                "        <path id=\"phone_screen\" d=\"{}\" style=\"fill:{};isolation:isolate\"/>\n",
                PHONE_SCREEN_PATH, pal.third.hex()
            ));
        }
        CodeType::Camera => {
            sb.push_str("    <g id=\"Logo\" data-logo-type=\"Camera\" transform=\"translate(293.275699 293.275699) scale(1.874000 1.874000)\">\n");
            for p in CAMERA_LOGO_PATHS.iter() {
                sb.push_str(&format!(
                    "        <path d=\"{}\" style=\"fill:{}\"/>\n",
                    p, pal.foreground.hex()
                ));
            }
        }
    }
    sb.push_str("    </g>\n");
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
