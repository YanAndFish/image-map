use serde::{Deserialize, Serialize};

/// Request messages (from Node.js to Rust) in NDJSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RequestMessage {
  /// Generate tiles for an input image.
  #[serde(rename = "generate")]
  Generate {
    /// Request id for correlating progress / completion events.
    id: String,
    /// Input image path.
    input: String,
    /// Output directory.
    output: String,
    /// Generation options.
    options: GenerateOptions,
  },
  /// Resize an image (no tiling).
  #[serde(rename = "resize")]
  Resize {
    /// Request id for correlating progress / completion events.
    id: String,
    /// Input image path.
    input: String,
    /// Output file path.
    output: String,
    /// Resize options.
    options: ResizeOptions,
  },
}

/// Response messages (from Rust to Node.js) in NDJSON.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ResponseMessage {
  /// Progress update.
  #[serde(rename = "progress")]
  Progress {
    /// Request id for correlating progress / completion events.
    id: String,
    /// Current progress value.
    current: u64,
    /// Total progress value.
    total: u64,
    /// Human-readable message.
    message: String,
  },
  /// Completion event (for tile generation).
  #[serde(rename = "complete")]
  Complete {
    /// Request id for correlating progress / completion events.
    id: String,
    /// Result payload.
    result: GenerateResult,
  },
  /// Resize completion event.
  #[serde(rename = "resizeComplete")]
  ResizeComplete {
    /// Request id for correlating progress / completion events.
    id: String,
    /// Result payload.
    result: ResizeResult,
  },
  /// Error event.
  #[serde(rename = "error")]
  Error {
    /// Request id for correlating progress / completion events.
    id: String,
    /// Error message.
    error: String,
  },
}

/// Options for generating tiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateOptions {
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
  #[serde(default = "default_auto_orient")]
  pub auto_orient: bool,
  /// Resize filter for building lower zoom levels.
  #[serde(default)]
  pub resize_filter: ResizeFilter,
  /// Downscale sharpening configuration.
  #[serde(default)]
  pub downscale_sharpen: DownscaleSharpenOptions,
}

/// Tile origin/alignment.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Origin {
  /// Align tiles to the top-left corner (web-map standard).
  TopLeft,
  /// Center-align the image within the tile grid.
  Center,
}

/// Resize filters used for downscaling between zoom levels.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ResizeFilter {
  /// Lanczos3 filter (high-quality, can ring).
  Lanczos3,
  /// Catmull-Rom bicubic filter (sharper).
  CatmullRom,
  /// Mitchell-Netravali bicubic filter (softer).
  Mitchell,
  /// Hamming filter (balanced quality).
  Hamming,
  /// Bilinear filter (fast, softer).
  Bilinear,
  /// Box filter (fastest, blocky).
  Box,
  /// Gaussian filter (soft).
  Gaussian,
}

impl Default for ResizeFilter {
  fn default() -> Self {
    Self::CatmullRom
  }
}

/// Downscale sharpening configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownscaleSharpenOptions {
  /// Whether downscale sharpening is enabled.
  #[serde(default = "default_downscale_sharpen_enabled")]
  pub enabled: bool,
  /// Gaussian blur sigma for unsharp mask.
  #[serde(default = "default_downscale_sharpen_sigma")]
  pub sigma: f32,
  /// Unsharp mask amount multiplier.
  #[serde(default = "default_downscale_sharpen_amount")]
  pub amount: f32,
  /// Threshold for minimal brightness change that will be sharpened.
  #[serde(default = "default_downscale_sharpen_threshold")]
  pub threshold: u8,
}

impl Default for DownscaleSharpenOptions {
  fn default() -> Self {
    Self {
      enabled: default_downscale_sharpen_enabled(),
      sigma: default_downscale_sharpen_sigma(),
      amount: default_downscale_sharpen_amount(),
      threshold: default_downscale_sharpen_threshold(),
    }
  }
}

fn default_auto_orient() -> bool {
  true
}

fn default_downscale_sharpen_enabled() -> bool {
  true
}

fn default_downscale_sharpen_sigma() -> f32 {
  0.5
}

fn default_downscale_sharpen_amount() -> f32 {
  0.35
}

fn default_downscale_sharpen_threshold() -> u8 {
  2
}

/// Tile output formats.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TileFormat {
  /// PNG (lossless).
  Png,
  /// JPG (lossy).
  #[serde(alias = "jpeg")]
  Jpg,
  /// WebP.
  Webp,
}

impl TileFormat {
  /// File extension for this format.
  pub fn extension(self) -> &'static str {
    match self {
      Self::Png => "png",
      Self::Jpg => "jpg",
      Self::Webp => "webp",
    }
  }
}

/// Result payload for a completed generate request.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateResult {
  /// Total number of tile files generated.
  pub tiles_generated: u64,
  /// Output directory.
  pub output_dir: String,
}

/// Resize mode specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ResizeMode {
  /// Resize by percentage (0-100 means shrink, >100 means enlarge).
  #[serde(rename = "percentage")]
  Percentage {
    /// Percentage value (e.g., 50 means 50% of original size).
    value: f64,
  },
  /// Resize by specifying the long edge in pixels.
  #[serde(rename = "longEdge")]
  LongEdge {
    /// Target long edge size in pixels.
    pixels: u32,
  },
  /// Resize by specifying both width and height (fit within, keep aspect ratio).
  #[serde(rename = "fit")]
  Fit {
    /// Maximum width in pixels.
    width: u32,
    /// Maximum height in pixels.
    height: u32,
  },
}

/// Options for resizing an image (without tiling).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResizeOptions {
  /// The resize mode specifying how to calculate output dimensions.
  pub mode: ResizeMode,
  /// Output format for the resized image.
  pub format: TileFormat,
  /// Whether to auto-orient input image pixels using EXIF orientation metadata.
  #[serde(default = "default_auto_orient")]
  pub auto_orient: bool,
  /// Resize filter for downscaling.
  #[serde(default)]
  pub resize_filter: ResizeFilter,
  /// Sharpening configuration for downscaling.
  #[serde(default)]
  pub sharpen: DownscaleSharpenOptions,
}

/// Result payload for a completed resize request.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResizeResult {
  /// Output file path.
  pub output_path: String,
  /// Input image width after EXIF auto-orientation (if enabled).
  pub original_width: u32,
  /// Input image height after EXIF auto-orientation (if enabled).
  pub original_height: u32,
  /// Resized image width.
  pub width: u32,
  /// Resized image height.
  pub height: u32,
}
