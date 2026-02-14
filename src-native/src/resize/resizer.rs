//! Image resize implementation.

use std::path::Path;
use std::sync::Arc;

use crate::image::{decoder, encoder, resize};
use crate::protocol::{ResizeOptions, ResizeResult};
use crate::tile::ProgressEvent;
use crate::{ImageMapError, Result};

/// Resize an image (without tiling) and write to output path.
pub fn resize_image(
  input_path: &Path,
  output_path: &Path,
  options: &ResizeOptions,
  on_progress: Option<Arc<dyn Fn(ProgressEvent) + Send + Sync>>,
) -> Result<ResizeResult> {
  let report = |current: u64, total: u64, message: String| {
    if let Some(cb) = &on_progress {
      cb(ProgressEvent {
        current,
        total,
        message,
      });
    }
  };

  report(0, 3, "Loading image...".to_string());

  let original = decoder::decode_rgba8(input_path, options.auto_orient)?;

  let original_width = original.width();
  let original_height = original.height();

  if original_width == 0 || original_height == 0 {
    return Err(ImageMapError::InvalidOptions(
      "input image has zero dimensions".to_string(),
    ));
  }

  report(1, 3, "Calculating dimensions...".to_string());

  let (new_width, new_height) =
    resize::calculate_target_dimensions(original_width, original_height, &options.mode)?;

  report(2, 3, format!("Resizing to {}x{}...", new_width, new_height));

  let resized = resize::resize_with_options(
    &original,
    new_width,
    new_height,
    options.resize_filter,
    &options.sharpen,
  )?;

  // Ensure parent directory exists.
  if let Some(parent) = output_path.parent() {
    if !parent.as_os_str().is_empty() {
      std::fs::create_dir_all(parent)?;
    }
  }

  encoder::write_tile(output_path, options.format, &resized)?;

  report(3, 3, "Done.".to_string());

  Ok(ResizeResult {
    output_path: output_path.to_string_lossy().to_string(),
    original_width,
    original_height,
    width: new_width,
    height: new_height,
  })
}

#[cfg(test)]
mod tests {
  use std::fs;

  use ::image::{DynamicImage, ImageBuffer, ImageFormat, Rgb};
  use tempfile::tempdir;

  use super::*;
  use crate::protocol::{DownscaleSharpenOptions, ResizeFilter, ResizeMode, TileFormat};

  #[test]
  fn resize_image_applies_exif_orientation_by_default()
  -> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let input_path = temp_dir.path().join("input.jpg");
    let output_path = temp_dir.path().join("output.png");

    let rgb = ImageBuffer::from_fn(3, 2, |x, y| Rgb([(x * 40) as u8, (y * 70) as u8, 200]));
    DynamicImage::ImageRgb8(rgb).save_with_format(&input_path, ImageFormat::Jpeg)?;

    let jpeg = fs::read(&input_path)?;
    fs::write(&input_path, inject_exif_orientation(&jpeg, 6))?;

    let options = ResizeOptions {
      mode: ResizeMode::LongEdge { pixels: 300 },
      format: TileFormat::Png,
      auto_orient: true,
      resize_filter: ResizeFilter::CatmullRom,
      sharpen: DownscaleSharpenOptions::default(),
    };

    let result = resize_image(&input_path, &output_path, &options, None)?;
    assert_eq!(result.original_width, 2);
    assert_eq!(result.original_height, 3);
    assert_eq!(result.width, 200);
    assert_eq!(result.height, 300);

    Ok(())
  }

  #[test]
  fn resize_image_keeps_original_orientation_when_disabled()
  -> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let input_path = temp_dir.path().join("input.jpg");
    let output_path = temp_dir.path().join("output.png");

    let rgb = ImageBuffer::from_fn(3, 2, |x, y| Rgb([(x * 40) as u8, (y * 70) as u8, 200]));
    DynamicImage::ImageRgb8(rgb).save_with_format(&input_path, ImageFormat::Jpeg)?;

    let jpeg = fs::read(&input_path)?;
    fs::write(&input_path, inject_exif_orientation(&jpeg, 6))?;

    let options = ResizeOptions {
      mode: ResizeMode::LongEdge { pixels: 300 },
      format: TileFormat::Png,
      auto_orient: false,
      resize_filter: ResizeFilter::CatmullRom,
      sharpen: DownscaleSharpenOptions::default(),
    };

    let result = resize_image(&input_path, &output_path, &options, None)?;
    assert_eq!(result.original_width, 3);
    assert_eq!(result.original_height, 2);
    assert_eq!(result.width, 300);
    assert_eq!(result.height, 200);

    Ok(())
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
