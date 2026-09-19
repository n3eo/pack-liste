use anyhow::Result;
use clap::Parser;

mod app;
mod config;
mod models;
mod quantity;
mod services;
mod ui;

#[derive(Parser, Debug)]
#[command(name = "pack-liste")]
#[command(author = "Nico")]
#[command(version = "0.1.0")]
#[command(about = "A terminal-based packing list application with dynamic quantity calculation")]
struct Args {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Force TUI mode (even in non-interactive environments)
    #[arg(long)]
    force_tui: bool,

    /// Packing list file to continue (saved under ~/.config/pack-liste/trips/)
    #[arg(short, long)]
    file: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    if args.debug {
        tracing_subscriber::fmt()
            .with_env_filter("pack_liste=debug")
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter("pack_liste=info")
            .init();
    }

    // Run the application
    if let Err(e) = app::run(args.force_tui, args.file) {
        eprintln!("Application error: {}", e);
        anyhow::bail!(e);
    }

    Ok(())
}
