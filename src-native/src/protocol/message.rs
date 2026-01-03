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
