mod app;
mod args;
mod gui;
mod tui;

use std::fs;

use anyhow::{Context, Result};
use clap::Parser;

use crate::{app::App, args::Args};

fn main() -> Result<()> {
    let args = Args::parse();
    let bytes = fs::read(&args.path)
        .with_context(|| format!("Failed to read input file: {}", args.path.display()))?;
    let file_name = args.path.display().to_string();

    let app = App::new(file_name, bytes, args.clamped_bytes_per_row());

    if args.gui {
        gui::run(app, args.debug)
    } else {
        let mut app = app;
        tui::run(&mut app)
    }
}
