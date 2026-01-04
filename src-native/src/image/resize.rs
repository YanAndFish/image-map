use fast_image_resize as fir;
use fir::images::Image;

use ::image::{Rgba, RgbaImage};

use crate::protocol::{DownscaleSharpenOptions, ResizeFilter};
use crate::{ImageMapError, Result};

/// Resize an RGBA8 image to a specific size using Lanczos3.
pub fn resize_rgba8(image: &RgbaImage, new_width: u32, new_height: u32) -> Result<RgbaImage> {
  resize_rgba8_with_filter(image, new_width, new_height, fir::FilterType::Lanczos3)
}

/// Resize an RGBA8 image to 50% (half) using default downscale settings.
pub fn resize_half_and_sharpen(image: &RgbaImage) -> Result<RgbaImage> {
  let defaults = DownscaleSharpenOptions::default();
  resize_half_with_options(image, ResizeFilter::default(), &defaults)
}

/// Resize an RGBA8 image to 50% (half) with configurable filter and sharpening.
pub fn resize_half_with_options(
  image: &RgbaImage,
  filter: ResizeFilter,
  sharpen: &DownscaleSharpenOptions,
) -> Result<RgbaImage> {
  let new_width = (image.width() / 2).max(1);
  let new_height = (image.height() / 2).max(1);

  let resized = resize_rgba8_with_filter(image, new_width, new_height, to_fir_filter(filter))?;
  if !sharpen.enabled || sharpen.amount <= 0.0 || sharpen.sigma <= 0.0 {
    return Ok(resized);
  }

  Ok(unsharp_rgba8(
    &resized,
    sharpen.sigma,
    sharpen.amount,
    sharpen.threshold,
  ))
}

/// Resize an RGBA8 image using the provided convolution filter.
fn resize_rgba8_with_filter(
  image: &RgbaImage,
  new_width: u32,
  new_height: u32,
  filter: fir::FilterType,
) -> Result<RgbaImage> {
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

  let options = fir::ResizeOptions::new().resize_alg(fir::ResizeAlg::Convolution(filter));
  let mut resizer = fir::Resizer::new();
  resizer
    .resize(&src, &mut dst, Some(&options))
    .map_err(|e| ImageMapError::Resize(e.to_string()))?;

  let out = RgbaImage::from_raw(new_width, new_height, dst.into_vec())
    .ok_or_else(|| ImageMapError::Resize("failed to construct resized image buffer".to_string()))?;

  Ok(out)
}

/// Apply a mild unsharp mask tuned for downscaled RGBA8 images.
fn unsharp_rgba8(image: &RgbaImage, sigma: f32, amount: f32, threshold: u8) -> RgbaImage {
  if amount <= 0.0 {
    return image.clone();
  }

  let blurred = ::image::imageops::blur(image, sigma);
  let (width, height) = image.dimensions();
  let mut out = RgbaImage::new(width, height);

  for y in 0..height {
    for x in 0..width {
      let orig = image.get_pixel(x, y).0;
      let blur = blurred.get_pixel(x, y).0;
      let mut out_pixel = [0u8; 4];

      for channel in 0..3 {
        let o = orig[channel] as f32;
        let b = blur[channel] as f32;
        let diff = o - b;

        if diff.abs() < threshold as f32 {
          out_pixel[channel] = orig[channel];
        } else {
          let sharpened = (o + diff * amount).clamp(0.0, 255.0).round();
          out_pixel[channel] = sharpened as u8;
        }
      }

      out_pixel[3] = orig[3];
      out.put_pixel(x, y, Rgba(out_pixel));
    }
  }

  out
}

/// Map protocol resize filters to fast_image_resize filters.
fn to_fir_filter(filter: ResizeFilter) -> fir::FilterType {
  match filter {
    ResizeFilter::Lanczos3 => fir::FilterType::Lanczos3,
    ResizeFilter::CatmullRom => fir::FilterType::CatmullRom,
    ResizeFilter::Mitchell => fir::FilterType::Mitchell,
    ResizeFilter::Hamming => fir::FilterType::Hamming,
    ResizeFilter::Bilinear => fir::FilterType::Bilinear,
    ResizeFilter::Box => fir::FilterType::Box,
    ResizeFilter::Gaussian => fir::FilterType::Gaussian,
  }
}
