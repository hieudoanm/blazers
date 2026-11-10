use std::path::Path;

pub fn generate_thumbnail(path: &Path, max_dim: u32) -> Result<Vec<u8>, String> {
    let img = image::open(path).map_err(|e| format!("open image: {e}"))?;
    let (w, h) = (img.width(), img.height());
    let (new_w, new_h) = if w > h {
        (max_dim, (h * max_dim / w).max(1))
    } else {
        ((w * max_dim / h).max(1), max_dim)
    };
    let resized = img.resize_exact(new_w, new_h, image::imageops::FilterType::CatmullRom);
    let mut buf = std::io::Cursor::new(Vec::new());
    resized
        .write_to(&mut buf, image::ImageFormat::Jpeg)
        .map_err(|e| format!("encode: {e}"))?;
    Ok(buf.into_inner())
}

pub fn is_supported_image(mime: &str) -> bool {
    matches!(
        mime,
        "image/jpeg" | "image/png" | "image/webp" | "image/gif"
    )
}

#[cfg(test)]
mod tests;
