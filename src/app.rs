use anyhow::Result;
use std::io::{self, IsTerminal};

pub fn run(force_tui: bool) -> Result<()> {
    // Check if we're in a terminal or force TUI mode
    if io::stdout().is_terminal() || force_tui {
        // Run TUI mode
        use crate::ui::app::App;
        let mut app = App::new()?;
        app.run()?;
    } else {
        // Non-interactive mode
        println!("Pack-Liste: Terminal-based packing list application");
        println!("To use the interactive TUI, run this in a terminal or use --force-tui.");
        println!("For CLI options, use --help");
    }

    Ok(())
}
