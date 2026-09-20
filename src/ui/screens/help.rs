use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::ui::help::help_content;

/// Render the help screen listing every binding.
pub(crate) fn draw_help_screen(f: &mut ratatui::prelude::Frame, size: Rect) {
    let block = Block::default()
        .title("Help - Navigation & Controls")
        .borders(Borders::ALL)
        .style(Style::new().bg(Color::Black).fg(Color::White));
    f.render_widget(block, size);

    let inner = Rect {
        x: size.x + 1,
        y: size.y + 1,
        width: size.width - 2,
        height: size.height - 2,
    };

    let help_text = help_content();
    let help_para = Paragraph::new(help_text).wrap(Wrap { trim: true });
    f.render_widget(help_para, inner);
}
