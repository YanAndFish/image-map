use crate::protocol::{DownscaleSharpenOptions, GenerateOptions, Origin, ResizeFilter, TileFormat};

/// Tile generation configuration.
#[derive(Debug, Clone)]
pub struct TileConfig {
  /// The width/height in pixels of each output tile.
  pub tile_size: u32,
  /// The output formats to write for each tile.
  pub formats: Vec<TileFormat>,
  /// The origin/alignment of the tile grid.
  pub origin: Origin,
  /// The minimum zoom level to generate.
  pub min_zoom: u8,
  /// The maximum zoom level to generate.
  pub max_zoom: u8,
  /// Whether to auto-orient input image pixels using EXIF orientation metadata.
  pub auto_orient: bool,
  /// Resize filter for building lower zoom levels.
  pub resize_filter: ResizeFilter,
  /// Downscale sharpening configuration.
  pub downscale_sharpen: DownscaleSharpenOptions,
}

impl From<&GenerateOptions> for TileConfig {
  fn from(options: &GenerateOptions) -> Self {
    Self {
      tile_size: options.tile_size,
      formats: options.formats.clone(),
      origin: options.origin,
      min_zoom: options.min_zoom,
      max_zoom: options.max_zoom,
      auto_orient: options.auto_orient,
      resize_filter: options.resize_filter,
      downscale_sharpen: options.downscale_sharpen.clone(),
    }
  }
}
