use fast_image_resize as fir;
use fir::images::Image;

use ::image::RgbaImage;

use crate::{ImageMapError, Result};

/// Resize an RGBA8 image to a specific size.
pub fn resize_rgba8(image: &RgbaImage, new_width: u32, new_height: u32) -> Result<RgbaImage> {
  let width = image.width();
  let height = image.height();

  if width == 0 || height == 0 {
    return Err(ImageMapError::InvalidOptions(
      "input image has zero width/height".to_string(),
    ));
  }
  if new_width == 0 || new_height == 0 {
    return Err(ImageMapError::InvalidOptions(
      "target image has zero width/height".to_string(),
    ));
  }

  let src = Image::from_vec_u8(
    width,
    height,
    image.clone().into_raw(),
    fir::PixelType::U8x4,
  )
  .map_err(|e| ImageMapError::Resize(e.to_string()))?;

  let mut dst = Image::new(new_width, new_height, fir::PixelType::U8x4);

  let options =
    fir::ResizeOptions::new().resize_alg(fir::ResizeAlg::Convolution(fir::FilterType::Lanczos3));
  let mut resizer = fir::Resizer::new();
  resizer
    .resize(&src, &mut dst, Some(&options))
    .map_err(|e| ImageMapError::Resize(e.to_string()))?;

  let out = RgbaImage::from_raw(new_width, new_height, dst.into_vec())
    .ok_or_else(|| ImageMapError::Resize("failed to construct resized image buffer".to_string()))?;

  Ok(out)
}

/// Resize an RGBA8 image to 50% (half) and apply a simple sharpen filter.
pub fn resize_half_and_sharpen(image: &RgbaImage) -> Result<RgbaImage> {
  let new_width = (image.width() / 2).max(1);
  let new_height = (image.height() / 2).max(1);

  let resized = resize_rgba8(image, new_width, new_height)?;
  Ok(sharpen_rgba8(&resized))
}

/// Apply a simple 3x3 sharpen kernel on an RGBA8 image.
pub fn sharpen_rgba8(image: &RgbaImage) -> RgbaImage {
  const KERNEL: [f32; 9] = [0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0];
  ::image::imageops::filter3x3(image, &KERNEL)
}
