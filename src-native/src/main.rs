use std::io::{self, BufRead, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

use clap::{Args, Parser, Subcommand, ValueEnum};

use image_map::protocol::{RequestMessage, ResponseMessage};
use image_map::tile::{ProgressEvent, TileConfig, generate_tiles};

#[derive(Debug, Parser)]
#[command(
  name = "image-map",
  version,
  about = "High-performance image tile generator"
)]
struct Cli {
  #[command(subcommand)]
  command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
  /// Run NDJSON stdio protocol mode.
  Stdio,
  /// Generate tiles directly (no protocol).
  Generate(GenerateCliArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliOrigin {
  TopLeft,
  Center,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliFormat {
  Png,
  Jpg,
  Webp,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliResizeFilter {
  Lanczos3,
  CatmullRom,
  Mitchell,
  Hamming,
  Bilinear,
  Box,
  Gaussian,
}

#[derive(Debug, Args)]
struct GenerateCliArgs {
  /// Input image path.
  #[arg(long)]
  input: String,
  /// Output directory.
  #[arg(long)]
  output: String,
  /// Tile size in pixels.
  #[arg(long, default_value_t = 256)]
  tile_size: u32,
  /// Output format(s). Repeatable: `--format webp --format jpg`.
  #[arg(long = "format", value_enum, default_value = "webp")]
  formats: Vec<CliFormat>,
  /// Tile origin/alignment.
  #[arg(long, value_enum, default_value = "top-left")]
  origin: CliOrigin,
  /// Minimum zoom level.
  #[arg(long, default_value_t = 0)]
  min_zoom: u8,
  /// Maximum zoom level.
  #[arg(long, default_value_t = 0)]
  max_zoom: u8,
  /// Resize filter for downscaling between zoom levels.
  #[arg(long, value_enum, default_value = "catmull-rom")]
  resize_filter: CliResizeFilter,
  /// Enable downscale sharpening.
  #[arg(long, default_value_t = true)]
  downscale_sharpen: bool,
  /// Gaussian blur sigma for downscale sharpening.
  #[arg(long, default_value_t = 0.5)]
  downscale_sharpen_sigma: f32,
  /// Unsharp amount for downscale sharpening.
  #[arg(long, default_value_t = 0.35)]
  downscale_sharpen_amount: f32,
  /// Threshold for downscale sharpening (0-255).
  #[arg(long, default_value_t = 2)]
  downscale_sharpen_threshold: u8,
}

fn main() -> Result<(), image_map::ImageMapError> {
  let cli = Cli::parse();

  match cli.command.unwrap_or(Command::Stdio) {
    Command::Stdio => run_stdio_protocol(),
    Command::Generate(args) => run_generate(args),
  }
}

fn run_generate(args: GenerateCliArgs) -> Result<(), image_map::ImageMapError> {
  let config = TileConfig {
    tile_size: args.tile_size,
    formats: args.formats.into_iter().map(into_format).collect(),
    origin: into_origin(args.origin),
    min_zoom: args.min_zoom,
    max_zoom: args.max_zoom,
    resize_filter: into_resize_filter(args.resize_filter),
    downscale_sharpen: image_map::protocol::DownscaleSharpenOptions {
      enabled: args.downscale_sharpen,
      sigma: args.downscale_sharpen_sigma,
      amount: args.downscale_sharpen_amount,
      threshold: args.downscale_sharpen_threshold,
    },
  };

  let progress = Arc::new(move |ev: ProgressEvent| {
    eprintln!("{}/{} {}", ev.current, ev.total, ev.message);
  });

  let result = generate_tiles(
    Path::new(&args.input),
    Path::new(&args.output),
    &config,
    Some(progress),
  )?;

  println!("{}", serde_json::to_string(&result)?);
  Ok(())
}

fn run_stdio_protocol() -> Result<(), image_map::ImageMapError> {
  let stdin = io::stdin();
  let stdout = Arc::new(Mutex::new(io::stdout()));

  for line in stdin.lock().lines() {
    let line = line?;
    let line = line.trim();
    if line.is_empty() {
      continue;
    }

    match serde_json::from_str::<RequestMessage>(line) {
      Ok(RequestMessage::Generate {
        id,
        input,
        output,
        options,
      }) => {
        let config = TileConfig::from(&options);

        let progress = make_progress_writer(stdout.clone(), id.clone());
        match generate_tiles(
          Path::new(&input),
          Path::new(&output),
          &config,
          Some(progress),
        ) {
          Ok(result) => {
            write_message(&stdout, &ResponseMessage::Complete { id, result })?;
          }
          Err(e) => {
            write_message(
              &stdout,
              &ResponseMessage::Error {
                id,
                error: e.to_string(),
              },
            )?;
          }
        }
      }
      Err(e) => {
        let id = extract_id(line).unwrap_or_else(|| "unknown".to_string());
        write_message(
          &stdout,
          &ResponseMessage::Error {
            id,
            error: format!("Invalid request: {e}"),
          },
        )?;
      }
    }
  }

  Ok(())
}

fn make_progress_writer(
  stdout: Arc<Mutex<io::Stdout>>,
  id: String,
) -> Arc<dyn Fn(ProgressEvent) + Send + Sync> {
  Arc::new(move |ev: ProgressEvent| {
    let msg = ResponseMessage::Progress {
      id: id.clone(),
      current: ev.current,
      total: ev.total,
      message: ev.message.clone(),
    };

    let Ok(line) = serde_json::to_string(&msg) else {
      return;
    };

    let mut out = stdout.lock().expect("stdout mutex poisoned");
    let _ = writeln!(out, "{line}");
    let _ = out.flush();
  })
}

fn write_message(
  stdout: &Arc<Mutex<io::Stdout>>,
  msg: &ResponseMessage,
) -> Result<(), image_map::ImageMapError> {
  let line = serde_json::to_string(msg)?;
  let mut out = stdout.lock().expect("stdout mutex poisoned");
  writeln!(out, "{line}")?;
  out.flush()?;
  Ok(())
}

fn extract_id(line: &str) -> Option<String> {
  let value: serde_json::Value = serde_json::from_str(line).ok()?;
  value.get("id")?.as_str().map(|s| s.to_string())
}

fn into_origin(origin: CliOrigin) -> image_map::protocol::Origin {
  match origin {
    CliOrigin::TopLeft => image_map::protocol::Origin::TopLeft,
    CliOrigin::Center => image_map::protocol::Origin::Center,
  }
}

fn into_format(format: CliFormat) -> image_map::protocol::TileFormat {
  match format {
    CliFormat::Png => image_map::protocol::TileFormat::Png,
    CliFormat::Jpg => image_map::protocol::TileFormat::Jpg,
    CliFormat::Webp => image_map::protocol::TileFormat::Webp,
  }
}

fn into_resize_filter(filter: CliResizeFilter) -> image_map::protocol::ResizeFilter {
  match filter {
    CliResizeFilter::Lanczos3 => image_map::protocol::ResizeFilter::Lanczos3,
    CliResizeFilter::CatmullRom => image_map::protocol::ResizeFilter::CatmullRom,
    CliResizeFilter::Mitchell => image_map::protocol::ResizeFilter::Mitchell,
    CliResizeFilter::Hamming => image_map::protocol::ResizeFilter::Hamming,
    CliResizeFilter::Bilinear => image_map::protocol::ResizeFilter::Bilinear,
    CliResizeFilter::Box => image_map::protocol::ResizeFilter::Box,
    CliResizeFilter::Gaussian => image_map::protocol::ResizeFilter::Gaussian,
  }
}
