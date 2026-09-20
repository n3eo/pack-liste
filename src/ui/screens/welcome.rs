use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::ui::hints::{HintSurface, hint_entries, hint_line};
use crate::ui::state::{AppDrawState, Focus};

// Capitalize the first character of a value for display
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

pub(crate) fn draw_welcome_screen(
    f: &mut ratatui::prelude::Frame,
    size: Rect,
    state: &AppDrawState,
) {
    let [title_area, content_area, instructions_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(12),
            Constraint::Length(4),
        ])
        .split(size)
    else {
        return;
    };

    // Title
    let title = Paragraph::new("Welcome to Pack-Liste")
        .style(Style::new().bold().fg(Color::Green))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(title, title_area);

    // Content - trip parameters
    let params_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(3),
        ])
        .split(content_area);

    // Inner area of a bordered block (one cell inside each border)
    let inner = |area: Rect| -> Rect {
        Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        }
    };

    // Trip file path (persist target for the compiled packing list, required)
    let file_block = Block::default()
        .title("Packing List File (Required)")
        .borders(Borders::ALL)
        .border_style(
            if state.focus == Focus::TripFile || state.editing_trip_file {
                Style::new().fg(Color::Yellow)
            } else {
                Style::new()
            },
        );

    let file_text = if state.editing_trip_file {
        vec![
            Line::from(format!(" {}_", state.trip_file_input)),
            Line::from(" Enter to confirm | Esc to cancel".dim()),
        ]
    } else {
        let shown = if state.trip_file_path.as_os_str().is_empty() {
            "(required - set a path to persist your packing list)".dim()
        } else {
            state.trip_file_path.display().to_string().into()
        };
        vec![
            Line::from(vec![Span::raw(" "), shown]),
            Line::from(" Enter to edit (relative paths resolve against the current folder)".dim()),
        ]
    };
    let file_para = Paragraph::new(file_text);
    f.render_widget(file_block, params_layout[0]);
    f.render_widget(file_para, inner(params_layout[0]));

    // Weekdays
    let weekdays_block = Block::default()
        .title("Trip Duration (Weekdays)")
        .borders(Borders::ALL)
        .border_style(if state.focus == Focus::Weekdays {
            Style::new().fg(Color::Yellow)
        } else {
            Style::new()
        });

    let weekdays_text = vec![
        Line::from(format!(" {} Days", state.weekdays.get())),
        Line::from(" Left/Right or +/- to adjust".dim()),
    ];
    let weekdays_para = Paragraph::new(weekdays_text);
    f.render_widget(weekdays_block, params_layout[1]);
    f.render_widget(weekdays_para, inner(params_layout[1]));

    // Travel Style
    let style_block = Block::default()
        .title("Travel Style")
        .borders(Borders::ALL)
        .border_style(if state.focus == Focus::TravelStyle {
            Style::new().fg(Color::Yellow)
        } else {
            Style::new()
        });

    let style_text = vec![
        Line::from(format!(" {}", capitalize_first(&state.travel_style))),
        Line::from(" Left/Right to cycle through options".dim()),
    ];
    let style_para = Paragraph::new(style_text);
    f.render_widget(style_block, params_layout[2]);
    f.render_widget(style_para, inner(params_layout[2]));

    // Weather
    let weather_block = Block::default()
        .title("Weather")
        .borders(Borders::ALL)
        .border_style(if state.focus == Focus::Weather {
            Style::new().fg(Color::Yellow)
        } else {
            Style::new()
        });

    let weather_text = vec![
        Line::from(format!(" {}", capitalize_first(&state.weather))),
        Line::from(" Left/Right to cycle through options".dim()),
    ];
    let weather_para = Paragraph::new(weather_text);
    f.render_widget(weather_block, params_layout[3]);
    f.render_widget(weather_para, inner(params_layout[3]));

    // Instructions
    let instructions =
        Paragraph::new(hint_line(hint_entries(HintSurface::Welcome))).wrap(Wrap { trim: true });
    f.render_widget(instructions, instructions_area);

    // Resume prompt for the remembered trip file
    if state.resume_prompt {
        let popup_width = 60.min(size.width.saturating_sub(4));
        let popup_height = 6.min(size.height);
        let popup_area = Rect {
            x: size.x + (size.width.saturating_sub(popup_width)) / 2,
            y: size.y + (size.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        let block = Block::default()
            .title("Resume Previous Trip?")
            .borders(Borders::ALL)
            .style(Style::new().bg(Color::Black).fg(Color::White));
        f.render_widget(block, popup_area);

        let inner = Rect {
            x: popup_area.x + 1,
            y: popup_area.y + 1,
            width: popup_area.width.saturating_sub(2),
            height: popup_area.height.saturating_sub(2),
        };

        let lines = vec![
            Line::from(vec![
                Span::raw("Load "),
                Span::raw(
                    state
                        .remembered_trip_file
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default(),
                )
                .fg(Color::Cyan)
                .bold(),
                Span::raw("?"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("Y/Enter").bold().fg(Color::Yellow),
                Span::raw(" resume | "),
                Span::raw("N/Esc").bold().fg(Color::Red),
                Span::raw(" start fresh"),
            ]),
        ];
        f.render_widget(Paragraph::new(lines), inner);
    }
}
