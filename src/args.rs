use std::path::PathBuf;

use clap::Parser;

/// Command line arguments for launching the hex viewer.
#[derive(Parser, Debug)]
#[command(author, version, about = "Simple TUI hex viewer")]
pub struct Args {
    /// Path to the target file.
    pub path: PathBuf,

    /// Bytes per row (8-32, default: 16).
    #[arg(long = "width", short = 'w', default_value_t = 16, value_parser = clap::value_parser!(usize))]
    pub bytes_per_row: usize,

    /// Launch in GUI mode.
    #[arg(long = "gui")]
    pub gui: bool,

    /// Enable verbose debug logging (stderr).
    #[arg(long = "debug")]
    pub debug: bool,
}

impl Args {
    /// Clamp bytes per row to the supported range.
    pub fn clamped_bytes_per_row(&self) -> usize {
        self.bytes_per_row.clamp(8, 32)
    }
}
