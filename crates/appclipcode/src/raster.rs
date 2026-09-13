#[cfg(feature = "png")]
pub fn render_png(svg_str: &str, size: Option<u32>) -> Result<Vec<u8>, String> {
    use resvg::usvg;

    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg_str, &opt)
        .map_err(|e| format!("failed to parse SVG for rendering: {}", e))?;

    let target_size = size.unwrap_or(800);
    let original_size = tree.size();
    let scale_x = (target_size as f32) / original_size.width();
    let scale_y = (target_size as f32) / original_size.height();

    let mut pixmap = tiny_skia::Pixmap::new(target_size, target_size)
        .ok_or_else(|| "failed to allocate pixmap".to_string())?;

    let transform = tiny_skia::Transform::from_scale(scale_x, scale_y);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    pixmap.encode_png().map_err(|e| format!("failed to encode PNG: {}", e))
}
