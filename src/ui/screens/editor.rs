use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::ui::hints::{HintSurface, hint_entries, hint_line};
use crate::ui::state::AppDrawState;

pub(crate) fn draw_config_editor_screen(
    f: &mut ratatui::prelude::Frame,
    size: Rect,
    state: &mut AppDrawState,
) {
    let [title_area, editor_area, footer_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(size)
    else {
        return;
    };

    let title = Paragraph::new(format!("Edit Config - {}", "config.yaml"))
        .style(Style::new().bold().fg(Color::Green));
    f.render_widget(title, title_area);

    let block = Block::default()
        .title("config.yaml (YAML)")
        .borders(Borders::ALL);
    f.render_widget(block, editor_area);

    let inner = Rect {
        x: editor_area.x + 1,
        y: editor_area.y + 1,
        width: editor_area.width.saturating_sub(2),
        height: editor_area.height.saturating_sub(2),
    };

    let (cursor_line, cursor_col) = state.config_editor_cursor;
    let viewport_height = inner.height as usize;

    // Keep the cursor line inside the viewport
    let mut scroll = state.config_editor_scroll;
    if cursor_line < scroll {
        scroll = cursor_line;
    } else if cursor_line >= scroll + viewport_height {
        scroll = cursor_line + 1 - viewport_height;
    }
    scroll = scroll.min(
        state
            .config_editor_text
            .len()
            .saturating_sub(viewport_height),
    );
    state.config_editor_scroll = scroll;

    let lines: Vec<Line> = state
        .config_editor_text
        .iter()
        .enumerate()
        .skip(scroll)
        .take(viewport_height)
        .map(|(i, text)| {
            let is_cursor_line = i == cursor_line;
            let mut line = Line::from(vec![
                Span::raw(format!("{:>4} ", i + 1)).fg(Color::DarkGray),
                Span::raw(text.clone()),
            ]);
            if is_cursor_line {
                line = line.style(Style::new().bg(Color::DarkGray));
            }
            line
        })
        .collect();

    let para = Paragraph::new(lines);
    f.render_widget(para, inner);

    // Cursor: a block on the cursor cell, drawn after the text
    let cursor_in_view = cursor_line >= scroll && cursor_line < scroll + viewport_height;
    if cursor_in_view {
        let cursor_y = inner.y + (cursor_line - scroll) as u16;
        let cursor_x = inner.x + 5 + cursor_col as u16;
        if cursor_x < inner.x + inner.width {
            let cell = Rect {
                x: cursor_x,
                y: cursor_y,
                width: 1,
                height: 1,
            };
            f.render_widget(
                Paragraph::new(" ").style(Style::new().bg(Color::Yellow).fg(Color::Yellow)),
                cell,
            );
        }
    }

    // Footer: status/error and key hints
    let footer = match &state.config_editor_error {
        Some(error) => Line::from(vec![
            Span::raw(" ").fg(Color::Red),
            Span::raw(error.clone()).fg(Color::Red).bold(),
        ]),
        None => hint_line(hint_entries(HintSurface::RawEditor)),
    };
    f.render_widget(Paragraph::new(footer), footer_area);
}
