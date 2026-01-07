//! Image resize implementation.

use std::path::Path;
use std::sync::Arc;

use crate::image::{encoder, resize};
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

  // Disable decode limits to allow large source images; rely on system memory limits instead.
  let mut reader = ::image::ImageReader::open(input_path)?;
  reader.no_limits();
  let original = reader.decode()?.to_rgba8();

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
