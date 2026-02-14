//! Image decoding helpers with optional EXIF auto-orientation.

use std::path::Path;

use ::image::metadata::Orientation;
use ::image::{DynamicImage, ImageDecoder, ImageReader, RgbaImage};

use crate::Result;

/// Decode an image into RGBA8 and optionally apply EXIF orientation to pixel data.
pub fn decode_rgba8(input_path: &Path, auto_orient: bool) -> Result<RgbaImage> {
  // Disable decode limits to allow large source images; rely on system memory limits instead.
  let mut reader = ImageReader::open(input_path)?;
  reader.no_limits();

  let mut decoder = reader.into_decoder()?;
  let orientation = orientation_from_result(auto_orient, decoder.orientation());

  let mut image = DynamicImage::from_decoder(decoder)?;
  apply_orientation(&mut image, auto_orient, orientation);

  Ok(image.to_rgba8())
}

/// Apply orientation transform to decoded image when auto-orient is enabled.
fn apply_orientation(image: &mut DynamicImage, auto_orient: bool, orientation: Orientation) {
  if auto_orient {
    image.apply_orientation(orientation);
  }
}

/// Resolve decoder orientation with fallback behavior required by the API contract.
fn orientation_from_result(
  auto_orient: bool,
  orientation: ::image::ImageResult<Orientation>,
) -> Orientation {
  if !auto_orient {
    return Orientation::NoTransforms;
  }

  orientation.unwrap_or(Orientation::NoTransforms)
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::io;

  use ::image::{DynamicImage, ImageBuffer, ImageFormat, Rgb};
  use tempfile::tempdir;

  use super::*;

  #[test]
  fn decode_rgba8_applies_exif_orientation_when_enabled()
  -> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let input_path = temp_dir.path().join("input.jpg");

    let rgb = ImageBuffer::from_fn(3, 2, |x, y| Rgb([(x * 50) as u8, (y * 80) as u8, 180]));
    DynamicImage::ImageRgb8(rgb).save_with_format(&input_path, ImageFormat::Jpeg)?;

    let jpeg = fs::read(&input_path)?;
    let jpeg_with_orientation = inject_exif_orientation(&jpeg, 6);
    fs::write(&input_path, jpeg_with_orientation)?;

    let oriented = decode_rgba8(&input_path, true)?;
    assert_eq!(oriented.width(), 2);
    assert_eq!(oriented.height(), 3);

    let non_oriented = decode_rgba8(&input_path, false)?;
    assert_eq!(non_oriented.width(), 3);
    assert_eq!(non_oriented.height(), 2);

    Ok(())
  }

  #[test]
  fn orientation_errors_fallback_to_no_transforms() {
    let err = ::image::ImageError::IoError(io::Error::other("mock orientation error"));
    let orientation = orientation_from_result(true, Err(err));
    assert_eq!(orientation, Orientation::NoTransforms);
  }

  #[test]
  fn auto_orient_disabled_always_uses_no_transforms() {
    let orientation = orientation_from_result(false, Ok(Orientation::Rotate90));
    assert_eq!(orientation, Orientation::NoTransforms);
  }

  #[test]
  fn apply_orientation_rotates_pixels_for_rotate90() {
    let pixels = vec![10, 0, 0, 255, 20, 0, 0, 255, 30, 0, 0, 255, 40, 0, 0, 255];
    let image = ::image::RgbaImage::from_raw(2, 2, pixels).expect("valid RGBA image");
    let mut dynamic = DynamicImage::ImageRgba8(image);

    apply_orientation(&mut dynamic, true, Orientation::Rotate90);
    let oriented = dynamic.to_rgba8();

    assert_eq!(oriented.width(), 2);
    assert_eq!(oriented.height(), 2);
    assert_eq!(oriented.get_pixel(0, 0).0, [30, 0, 0, 255]);
    assert_eq!(oriented.get_pixel(1, 0).0, [10, 0, 0, 255]);
    assert_eq!(oriented.get_pixel(0, 1).0, [40, 0, 0, 255]);
    assert_eq!(oriented.get_pixel(1, 1).0, [20, 0, 0, 255]);
  }

  /// Insert an EXIF APP1 segment with orientation into a JPEG byte stream.
  fn inject_exif_orientation(jpeg: &[u8], orientation: u16) -> Vec<u8> {
    assert!((1..=8).contains(&orientation));
    assert!(jpeg.len() >= 2);
    assert_eq!(jpeg[0], 0xFF);
    assert_eq!(jpeg[1], 0xD8);

    let exif_payload = exif_payload_with_orientation(orientation);
    let segment_len = (exif_payload.len() + 2) as u16;

    let mut out = Vec::with_capacity(jpeg.len() + 4 + exif_payload.len());
    out.extend_from_slice(&jpeg[0..2]);
    out.extend_from_slice(&[0xFF, 0xE1]);
    out.extend_from_slice(&segment_len.to_be_bytes());
    out.extend_from_slice(&exif_payload);
    out.extend_from_slice(&jpeg[2..]);
    out
  }

  /// Build a minimal Exif payload containing a single orientation tag in IFD0.
  fn exif_payload_with_orientation(orientation: u16) -> Vec<u8> {
    let mut payload = Vec::with_capacity(32);
    payload.extend_from_slice(b"Exif\0\0");
    payload.extend_from_slice(&[0x4D, 0x4D, 0x00, 0x2A, 0x00, 0x00, 0x00, 0x08]);
    payload.extend_from_slice(&[0x00, 0x01]);
    payload.extend_from_slice(&[0x01, 0x12]);
    payload.extend_from_slice(&[0x00, 0x03]);
    payload.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);
    payload.extend_from_slice(&[(orientation >> 8) as u8, orientation as u8, 0x00, 0x00]);
    payload.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    payload
  }
}
