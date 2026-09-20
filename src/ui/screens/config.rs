use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::ui::hints::{HintSurface, hint_entries, hint_line};
use crate::ui::state::{AppDrawState, ConfigSection, ModifierField};

pub(crate) fn draw_config_screen(
    f: &mut ratatui::prelude::Frame,
    size: Rect,
    state: &AppDrawState,
) {
    let [title_area, content_area, instructions_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(size)
    else {
        return;
    };

    // Title
    let title =
        Paragraph::new("Configuration - Modifiers").style(Style::new().bold().fg(Color::Green));
    f.render_widget(title, title_area);

    // Modifier editor popup takes precedence
    if let Some(editor) = &state.modifier_editor {
        let popup_width = 50.min(size.width.saturating_sub(4));
        let popup_area = Rect {
            x: size.x + (size.width.saturating_sub(popup_width)) / 2,
            y: size.y + 5,
            width: popup_width,
            height: 7,
        };

        let block = Block::default()
            .title(if editor.index.is_some() {
                "Edit Modifier"
            } else {
                "Add Modifier"
            })
            .borders(Borders::ALL)
            .style(Style::new().bg(Color::Black).fg(Color::White));
        f.render_widget(block, popup_area);

        let inner = Rect {
            x: popup_area.x + 1,
            y: popup_area.y + 1,
            width: popup_area.width.saturating_sub(2),
            height: popup_area.height.saturating_sub(2),
        };

        let field_style = Style::new().fg(Color::Yellow).bold();
        let normal_style = Style::new().fg(Color::White);
        let name_line = Line::from(vec![
            Span::raw("Name: ").style(if editor.field == ModifierField::Name {
                field_style
            } else {
                normal_style
            }),
            Span::raw(&editor.name),
        ]);
        let multiplier_line = Line::from(vec![
            Span::raw("Multiplier: ").style(if editor.field == ModifierField::Multiplier {
                field_style
            } else {
                normal_style
            }),
            Span::raw(&editor.multiplier),
        ]);
        let hint = hint_line(hint_entries(HintSurface::ModifierEditor));

        let para = Paragraph::new(vec![name_line, multiplier_line, hint]);
        f.render_widget(para, inner);
        return;
    }

    // Section tabs
    let tabs = Line::from(vec![
        Span::raw(if state.config_section == ConfigSection::TravelStyles {
            "[Travel Styles]"
        } else {
            " Travel Styles "
        })
        .style(if state.config_section == ConfigSection::TravelStyles {
            Style::new().fg(Color::Yellow).bold()
        } else {
            Style::new().fg(Color::DarkGray)
        }),
        Span::raw(" | "),
        Span::raw(if state.config_section == ConfigSection::Weather {
            "[Weather]"
        } else {
            " Weather "
        })
        .style(if state.config_section == ConfigSection::Weather {
            Style::new().fg(Color::Yellow).bold()
        } else {
            Style::new().fg(Color::DarkGray)
        }),
    ]);
    f.render_widget(
        Paragraph::new(tabs),
        Rect {
            x: content_area.x,
            y: content_area.y,
            width: content_area.width,
            height: 1,
        },
    );

    // Modifier list of the focused section
    let list_area = Rect {
        x: content_area.x,
        y: content_area.y + 2,
        width: content_area.width,
        height: content_area.height.saturating_sub(2),
    };

    let modifiers = match state.config_section {
        ConfigSection::TravelStyles => &state.modifiers.travel_styles,
        ConfigSection::Weather => &state.modifiers.weather,
    };

    // Window the list so the cursor stays visible
    let viewport_height = list_area.height as usize;
    let mut scroll = state
        .config_scroll
        .min(modifiers.len().saturating_sub(viewport_height));
    if state.config_cursor >= scroll + viewport_height {
        scroll = state.config_cursor + 1 - viewport_height;
    } else if state.config_cursor < scroll {
        scroll = state.config_cursor;
    }

    let lines: Vec<Line> = modifiers
        .iter()
        .enumerate()
        .skip(scroll)
        .take(viewport_height)
        .map(|(i, m)| {
            let is_cursor = i == state.config_cursor;
            let mut line = Line::from(vec![
                Span::raw("  "),
                Span::raw(format!("{:<20}", m.name)),
                Span::raw(format!(" x{:.2}", m.multiplier)),
            ]);
            if is_cursor {
                line = line.style(Style::new().fg(Color::Yellow).bg(Color::DarkGray));
            }
            line
        })
        .collect();

    let list_block = Block::default()
        .title(state.config_section.title())
        .borders(Borders::ALL);
    f.render_widget(list_block, list_area);
    f.render_widget(
        Paragraph::new(lines),
        Rect {
            x: list_area.x + 1,
            y: list_area.y + 1,
            width: list_area.width.saturating_sub(2),
            height: list_area.height.saturating_sub(2),
        },
    );

    // Instructions
    let instructions =
        Paragraph::new(hint_line(hint_entries(HintSurface::Config))).wrap(Wrap { trim: true });
    f.render_widget(instructions, instructions_area);
}
