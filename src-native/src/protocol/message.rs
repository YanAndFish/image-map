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
  /// Completion event.
  #[serde(rename = "complete")]
  Complete {
    /// Request id for correlating progress / completion events.
    id: String,
    /// Result payload.
    result: GenerateResult,
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
