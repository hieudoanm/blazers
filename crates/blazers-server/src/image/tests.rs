use super::*;
use std::path::PathBuf;

fn test_image_path(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("blazers_test_images");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

#[test]
fn supports_jpeg() {
    assert!(is_supported_image("image/jpeg"));
}

#[test]
fn supports_png() {
    assert!(is_supported_image("image/png"));
}

#[test]
fn supports_gif() {
    assert!(is_supported_image("image/gif"));
}

#[test]
fn supports_webp() {
    assert!(is_supported_image("image/webp"));
}

#[test]
fn rejects_text_plain() {
    assert!(!is_supported_image("text/plain"));
}

#[test]
fn rejects_empty_string() {
    assert!(!is_supported_image(""));
}

#[test]
fn rejects_case_insensitive() {
    assert!(!is_supported_image("IMAGE/JPEG"));
}

#[test]
fn test_thumbnail_generates_jpeg_bytes() {
    let path = test_image_path("jpeg_bytes.png");
    let img = image::RgbImage::new(100, 200);
    img.save(&path).unwrap();

    let result = generate_thumbnail(&path, 50);
    assert!(result.is_ok());
    let bytes = result.unwrap();
    assert!(!bytes.is_empty());
    assert_eq!(bytes[0], 0xFF);
    assert_eq!(bytes[1], 0xD8);

    std::fs::remove_file(&path).unwrap();
}

#[test]
fn test_thumbnail_non_existent_file() {
    let path = Path::new("/tmp/nonexistent_image_xyz.png");
    let result = generate_thumbnail(path, 50);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("open image"));
}

#[test]
fn test_thumbnail_dimensions_portrait() {
    let path = test_image_path("portrait.png");
    let img = image::RgbImage::new(100, 200);
    img.save(&path).unwrap();

    let bytes = generate_thumbnail(&path, 50).unwrap();
    let thumb = image::load_from_memory(&bytes).unwrap();
    assert_eq!(thumb.width(), 25);
    assert_eq!(thumb.height(), 50);

    std::fs::remove_file(&path).unwrap();
}

#[test]
fn test_thumbnail_dimensions_landscape() {
    let path = test_image_path("landscape.png");
    let img = image::RgbImage::new(200, 100);
    img.save(&path).unwrap();

    let bytes = generate_thumbnail(&path, 50).unwrap();
    let thumb = image::load_from_memory(&bytes).unwrap();
    assert_eq!(thumb.width(), 50);
    assert_eq!(thumb.height(), 25);

    std::fs::remove_file(&path).unwrap();
}

#[test]
fn test_thumbnail_dimensions_square() {
    let path = test_image_path("square.png");
    let img = image::RgbImage::new(100, 100);
    img.save(&path).unwrap();

    let bytes = generate_thumbnail(&path, 50).unwrap();
    let thumb = image::load_from_memory(&bytes).unwrap();
    assert_eq!(thumb.width(), 50);
    assert_eq!(thumb.height(), 50);

    std::fs::remove_file(&path).unwrap();
}

#[test]
fn test_thumbnail_different_max_dim() {
    let path = test_image_path("different_max_dim.png");
    let img = image::RgbImage::new(200, 400);
    img.save(&path).unwrap();

    let bytes_small = generate_thumbnail(&path, 25).unwrap();
    let thumb_small = image::load_from_memory(&bytes_small).unwrap();
    assert_eq!(thumb_small.width(), 12);
    assert_eq!(thumb_small.height(), 25);

    let bytes_large = generate_thumbnail(&path, 100).unwrap();
    let thumb_large = image::load_from_memory(&bytes_large).unwrap();
    assert_eq!(thumb_large.width(), 50);
    assert_eq!(thumb_large.height(), 100);

    std::fs::remove_file(&path).unwrap();
}

#[test]
fn test_thumbnail_min_dimension_clamped() {
    let path = test_image_path("min_dim.png");
    let img = image::RgbImage::new(1000, 1);
    img.save(&path).unwrap();

    let bytes = generate_thumbnail(&path, 50).unwrap();
    let thumb = image::load_from_memory(&bytes).unwrap();
    assert_eq!(thumb.width(), 50);
    assert_eq!(thumb.height(), 1);

    std::fs::remove_file(&path).unwrap();
}
