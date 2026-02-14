use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use ::image::{Rgba, RgbaImage};
use rayon::prelude::*;

use crate::image::{decoder, encoder, resize};
use crate::protocol::{GenerateResult, Origin};
use crate::tile::config::TileConfig;
use crate::{ImageMapError, Result};

/// Progress event emitted during generation.
#[derive(Debug, Clone)]
pub struct ProgressEvent {
  /// Current progress value.
  pub current: u64,
  /// Total progress value.
  pub total: u64,
  /// Human-readable message.
  pub message: String,
}

/// Generate ZXY tiles for an input image.
pub fn generate_tiles(
  input_path: &Path,
  output_dir: &Path,
  config: &TileConfig,
  on_progress: Option<Arc<dyn Fn(ProgressEvent) + Send + Sync>>,
) -> Result<GenerateResult> {
  validate_config(config)?;

  fs::create_dir_all(output_dir)?;

  let original = decoder::decode_rgba8(input_path, config.auto_orient)?;
  let total_files = estimate_total_files(original.width(), original.height(), config);

  let progress_current = AtomicU64::new(0);
  let progress_step = (total_files / 100).max(1);

  let formats = Arc::new(config.formats.clone());

  let report = |current: u64, message: String| {
    if let Some(cb) = &on_progress {
      cb(ProgressEvent {
        current,
        total: total_files,
        message,
      });
    }
  };

  let mut level_img = original;
  for z in 0..=config.max_zoom {
    let width = level_img.width();
    let height = level_img.height();
    let (tiles_x, tiles_y) = tile_grid_dimensions(width, height, config.tile_size);
    let (offset_x, offset_y) = origin_offsets(
      width,
      height,
      tiles_x,
      tiles_y,
      config.tile_size,
      config.origin,
    );

    if z >= config.min_zoom {
      report(
        progress_current.load(Ordering::Relaxed),
        format!("Generating z={z}..."),
      );

      let z_dir = output_dir.join(z.to_string());
      fs::create_dir_all(&z_dir)?;

      // Pre-create x directories to avoid racy mkdirs in the parallel loop.
      for x in 0..tiles_x {
        fs::create_dir_all(z_dir.join(x.to_string()))?;
      }

      let z_dir = Arc::new(z_dir);
      let tile_size = config.tile_size;
      let img_ref = &level_img;

      (0..tiles_x)
        .into_par_iter()
        .try_for_each(|x| -> Result<()> {
          let x_dir = z_dir.join(x.to_string());

          for y in 0..tiles_y {
            let tile = extract_tile(img_ref, tile_size, offset_x, offset_y, x, y);

            for format in formats.iter().copied() {
              let file_name = format!("{y}.{}", format.extension());
              let tile_path = x_dir.join(file_name);

              encoder::write_tile(&tile_path, format, &tile)?;

              let current = progress_current.fetch_add(1, Ordering::Relaxed) + 1;
              if current == total_files || current % progress_step == 0 {
                report(current, format!("z={z} x={x} y={y}"));
              }
            }
          }

          Ok(())
        })?;
    }

    // Prepare next zoom level.
    if z < config.max_zoom {
      level_img = resize::resize_half_with_options(
        &level_img,
        config.resize_filter,
        &config.downscale_sharpen,
      )?;
    }
  }

  let tiles_generated = progress_current.load(Ordering::Relaxed);

  Ok(GenerateResult {
    tiles_generated,
    output_dir: output_dir.to_string_lossy().to_string(),
  })
}

fn validate_config(config: &TileConfig) -> Result<()> {
  if config.tile_size == 0 {
    return Err(ImageMapError::InvalidOptions(
      "tileSize must be greater than 0".to_string(),
    ));
  }
  if config.formats.is_empty() {
    return Err(ImageMapError::InvalidOptions(
      "formats must contain at least one format".to_string(),
    ));
  }
  if config.min_zoom > config.max_zoom {
    return Err(ImageMapError::InvalidOptions(
      "minZoom must be <= maxZoom".to_string(),
    ));
  }
  if config.downscale_sharpen.enabled {
    if !config.downscale_sharpen.sigma.is_finite() || config.downscale_sharpen.sigma <= 0.0 {
      return Err(ImageMapError::InvalidOptions(
        "downscaleSharpen.sigma must be a positive number".to_string(),
      ));
    }
    if !config.downscale_sharpen.amount.is_finite() || config.downscale_sharpen.amount < 0.0 {
      return Err(ImageMapError::InvalidOptions(
        "downscaleSharpen.amount must be a non-negative number".to_string(),
      ));
    }
  }

  Ok(())
}

fn estimate_total_files(width: u32, height: u32, config: &TileConfig) -> u64 {
  let mut total = 0u64;
  let mut w = width;
  let mut h = height;

  let formats_len = config.formats.len() as u64;
  for z in 0..=config.max_zoom {
    if z >= config.min_zoom {
      let (tiles_x, tiles_y) = tile_grid_dimensions(w, h, config.tile_size);
      total += tiles_x as u64 * tiles_y as u64 * formats_len;
    }

    w = (w / 2).max(1);
    h = (h / 2).max(1);
  }

  total
}

fn tile_grid_dimensions(width: u32, height: u32, tile_size: u32) -> (u32, u32) {
  (ceil_div(width, tile_size), ceil_div(height, tile_size))
}

fn origin_offsets(
  width: u32,
  height: u32,
  tiles_x: u32,
  tiles_y: u32,
  tile_size: u32,
  origin: Origin,
) -> (i64, i64) {
  match origin {
    Origin::TopLeft => (0, 0),
    Origin::Center => {
      let canvas_w = tiles_x as i64 * tile_size as i64;
      let canvas_h = tiles_y as i64 * tile_size as i64;
      let offset_x = (canvas_w - width as i64) / 2;
      let offset_y = (canvas_h - height as i64) / 2;
      (offset_x, offset_y)
    }
  }
}

fn extract_tile(
  image: &RgbaImage,
  tile_size: u32,
  offset_x: i64,
  offset_y: i64,
  x: u32,
  y: u32,
) -> RgbaImage {
  let mut tile = RgbaImage::from_pixel(tile_size, tile_size, Rgba([0, 0, 0, 0]));

  let img_w = image.width() as i64;
  let img_h = image.height() as i64;
  let tile_size_i = tile_size as i64;

  let img_x0 = x as i64 * tile_size_i - offset_x;
  let img_y0 = y as i64 * tile_size_i - offset_y;

  let src_x_start = img_x0.max(0);
  let src_y_start = img_y0.max(0);
  let src_x_end = (img_x0 + tile_size_i).min(img_w);
  let src_y_end = (img_y0 + tile_size_i).min(img_h);

  for src_y in src_y_start..src_y_end {
    for src_x in src_x_start..src_x_end {
      let dest_x = (src_x - img_x0) as u32;
      let dest_y = (src_y - img_y0) as u32;
      let pixel = image.get_pixel(src_x as u32, src_y as u32);
      tile.put_pixel(dest_x, dest_y, *pixel);
    }
  }

  tile
}

fn ceil_div(dividend: u32, divisor: u32) -> u32 {
  (dividend + divisor - 1) / divisor
}
