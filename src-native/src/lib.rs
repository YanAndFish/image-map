//! Rust core for `image-map`.

pub mod image;
pub mod protocol;
pub mod resize;
pub mod tile;

use std::fmt;

/// Result type for `image-map`.
pub type Result<T> = std::result::Result<T, ImageMapError>;

/// Error type for `image-map`.
#[derive(Debug)]
pub enum ImageMapError {
  /// IO related errors.
  Io(std::io::Error),
  /// Image decoding / encoding errors.
  Image(::image::ImageError),
  /// JSON serialization / deserialization errors.
  Json(serde_json::Error),
  /// Resize related errors.
  Resize(String),
  /// Invalid user-provided options.
  InvalidOptions(String),
}

impl fmt::Display for ImageMapError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Io(e) => write!(f, "IO error: {e}"),
      Self::Image(e) => write!(f, "Image error: {e}"),
      Self::Json(e) => write!(f, "JSON error: {e}"),
      Self::Resize(e) => write!(f, "Resize error: {e}"),
      Self::InvalidOptions(e) => write!(f, "Invalid options: {e}"),
    }
  }
}

impl std::error::Error for ImageMapError {}

impl From<std::io::Error> for ImageMapError {
  fn from(value: std::io::Error) -> Self {
    Self::Io(value)
  }
}

impl From<::image::ImageError> for ImageMapError {
  fn from(value: ::image::ImageError) -> Self {
    Self::Image(value)
  }
}

impl From<serde_json::Error> for ImageMapError {
  fn from(value: serde_json::Error) -> Self {
    Self::Json(value)
  }
}
