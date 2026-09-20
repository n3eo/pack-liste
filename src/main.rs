use anyhow::Result;
use clap::Parser;

pub(crate) mod app;
pub(crate) mod config;
pub(crate) mod models;
pub(crate) mod quantity;
pub(crate) mod ui;

#[derive(Parser, Debug)]
#[command(name = "pack-liste")]
#[command(author = "Nico")]
#[command(version = "0.1.0")]
#[command(about = "A terminal-based packing list application with dynamic quantity calculation")]
struct Args {
    /// Force TUI mode (even in non-interactive environments)
    #[arg(long)]
    force_tui: bool,

    /// Packing list file to open or continue
    /// (relative paths resolve against the current folder)
    file: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Run the application; the returned error is reported once by
    // the Result returned from main
    app::run(args.force_tui, args.file)
}
