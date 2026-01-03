//! NDJSON protocol definitions for stdio communication.

pub mod message;

pub use message::{
  GenerateOptions, GenerateResult, Origin, RequestMessage, ResponseMessage, TileFormat,
};
