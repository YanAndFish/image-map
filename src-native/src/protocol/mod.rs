//! NDJSON protocol definitions for stdio communication.

pub mod message;

pub use message::{
  DownscaleSharpenOptions, GenerateOptions, GenerateResult, Origin, RequestMessage, ResizeFilter,
  ResponseMessage, TileFormat,
};
