use std::path::Path;

use ::image::{DynamicImage, ImageFormat, Rgb, RgbImage, RgbaImage};

use crate::{Result, protocol::TileFormat};

/// Encode and write a tile image to a file.
pub fn write_tile(path: &Path, format: TileFormat, tile: &RgbaImage) -> Result<()> {
  match format {
    TileFormat::Png => {
      DynamicImage::ImageRgba8(tile.clone()).save_with_format(path, ImageFormat::Png)?;
    }
    TileFormat::Webp => {
      DynamicImage::ImageRgba8(tile.clone()).save_with_format(path, ImageFormat::WebP)?;
    }
    TileFormat::Jpg => {
      let rgb = rgba_to_rgb_over_white(tile);
      DynamicImage::ImageRgb8(rgb).save_with_format(path, ImageFormat::Jpeg)?;
    }
  }

  Ok(())
}

fn rgba_to_rgb_over_white(tile: &RgbaImage) -> RgbImage {
  let mut out = RgbImage::new(tile.width(), tile.height());

  for (x, y, pixel) in tile.enumerate_pixels() {
    let [r, g, b, a] = pixel.0;
    let a = a as u32;

    let r = (r as u32 * a + 255 * (255 - a)) / 255;
    let g = (g as u32 * a + 255 * (255 - a)) / 255;
    let b = (b as u32 * a + 255 * (255 - a)) / 255;

    out.put_pixel(x, y, Rgb([r as u8, g as u8, b as u8]));
  }

  out
}
