use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
};

use crate::models::item::Item;
use crate::ui::hints::{
    HintSurface, collapsed_hint_line, hint_entries, hint_rows, hint_rows_fit_budget,
};
use crate::ui::state::{AppDrawState, FocusSide, PanelRow, StatusKind, group_items_by_category};

/// The main screen's layout mode (spec 006): a pure function of the
/// viewport, recomputed per frame — no persisted state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LayoutMode {
    /// Two panels side by side (at or above the width threshold)
    SideBySide,
    /// Both panels full-width, stacked vertically (narrow terminals)
    Stacked,
    /// Below the minimum usable size: only the resize notice
    TooSmall,
}

/// The side-by-side threshold (plan D2): two 39-column minimum
/// panels plus margins — at or above 80 columns the panels fit
/// side by side; below it they stack.
const SIDE_BY_SIDE_MIN_WIDTH: u16 = 80;
/// The minimum usable height: below it the notice replaces the
/// layout (FR-008's framing — the smallest size the layouts serve).
const MIN_HEIGHT: u16 = 24;
/// The minimum usable width even for the stacked mode: the rows
/// need room for the selection symbol, name, and quantity.
const MIN_WIDTH: u16 = 40;

/// Decide the layout mode from the viewport (pure, stable at the
/// boundary: the same inputs always give the same mode).
pub(crate) fn layout_mode(width: u16, height: u16) -> LayoutMode {
    if width < MIN_WIDTH || height < MIN_HEIGHT {
        LayoutMode::TooSmall
    } else if width >= SIDE_BY_SIDE_MIN_WIDTH {
        LayoutMode::SideBySide
    } else {
        LayoutMode::Stacked
    }
}

/// Render the main screen: the all-items panel on the left and the
/// compiled packing list on the right.
pub(crate) fn draw_main_screen(
    f: &mut ratatui::prelude::Frame,
    size: Rect,
    state: &mut AppDrawState,
) {
    // Fullscreen compiled-list view (spec 007): the same renderer at
    // the full main-screen area — presentation-only
    if state.fullscreen {
        draw_selected_items_panel(f, size, state);
        return;
    }

    // The layout adapts to the viewport (spec 006): side by side,
    // stacked, or the too-small notice
    match layout_mode(size.width, size.height) {
        LayoutMode::TooSmall => {
            draw_too_small_notice(f, size);
        }
        LayoutMode::SideBySide => {
            // Split screen: left (all items), right (selected items only)
            let [left_area, right_area] = *Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(size)
            else {
                return;
            };
            draw_all_items_panel(f, left_area, state);
            draw_selected_items_panel(f, right_area, state);
        }
        LayoutMode::Stacked => {
            // Both panels full-width, stacked vertically (plan D1:
            // the same renderers, different areas)
            let [top_area, bottom_area] = *Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(size)
            else {
                return;
            };
            draw_all_items_panel(f, top_area, state);
            draw_selected_items_panel(f, bottom_area, state);
        }
    }
}

/// The too-small notice (spec 006, US4): the only thing drawn below
/// the minimum usable size; all state is untouched.
fn draw_too_small_notice(f: &mut ratatui::prelude::Frame, size: Rect) {
    // The message truncates to the width — the notice itself must
    // never overflow (US4/AC3)
    let full = "Terminal too small — resize to at least 40x24";
    let short = "Too small";
    let text = if size.width as usize >= full.chars().count() {
        full
    } else if size.width as usize >= short.chars().count() {
        short
    } else {
        ""
    };
    let line = Line::from(text).fg(Color::Yellow);
    let area = Rect {
        x: 0,
        y: size.height / 2,
        width: size.width,
        height: 1,
    };
    f.render_widget(Paragraph::new(line), area);
}

fn draw_all_items_panel(f: &mut ratatui::prelude::Frame, area: Rect, state: &mut AppDrawState) {
    // The hint footer's height adapts to the panel width: entries are
    // packed into as few rows as fit, so every binding stays visible.
    // When the packed rows would squeeze the list below its minimum
    // (spec 006, US5), the hints collapse to the single `? Help` line.
    let entries = hint_entries(HintSurface::LeftPanel);
    let hint_lines = if hint_rows_fit_budget(entries, area.width as usize, 3) {
        hint_rows(entries, area.width as usize)
    } else {
        vec![collapsed_hint_line()]
    };
    let hints_height = hint_lines.len() as u16;
    // One extra row for the status line when a message is pending
    let status_height = u16::from(state.status.is_some());
    let [title_area, list_area, status_area, hints_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(status_height),
            Constraint::Length(hints_height),
        ])
        .split(area)
    else {
        return;
    };

    // Title with side indicator
    let title = if state.focus_side == FocusSide::Left {
        Paragraph::new("All Items [LEFT]")
            .style(Style::new().bold().fg(Color::Green))
            .block(Block::default().borders(Borders::BOTTOM))
    } else {
        Paragraph::new("All Items [LEFT]")
            .style(Style::new().bold().fg(Color::White))
            .block(Block::default().borders(Borders::BOTTOM))
    };
    f.render_widget(title, title_area);

    // Group items by category
    let categories_map =
        group_items_by_category(state.packing_list.items.iter().enumerate(), |item| {
            item.categories.clone()
        });

    // Cursor position within the panel's rows (only when the left side is focused)
    let current_row = if state.focus_side == FocusSide::Left {
        Some(state.selected_index)
    } else {
        None
    };

    // Rows in display order (category headers + items of expanded categories)
    let rows = state.left_rows();
    let total_rows = rows.len();

    // Visible window: keep the cursor row in view, otherwise keep the current offset
    let viewport_height = list_area.height as usize;
    let mut scroll = state
        .left_scroll
        .min(total_rows.saturating_sub(viewport_height));
    if let Some(cursor) = current_row {
        if cursor < scroll {
            scroll = cursor;
        } else if cursor >= scroll + viewport_height {
            scroll = cursor + 1 - viewport_height;
        }
    }
    scroll = scroll.min(total_rows.saturating_sub(viewport_height));
    state.left_scroll = scroll;

    let mut row_index = 0;

    for (category, items) in &categories_map {
        let is_collapsed = state.collapsed_categories.contains(category);

        // Category header
        let category_symbol = if is_collapsed { "▶ " } else { "▼ " };
        let category_line = Line::from(vec![
            Span::raw(category_symbol).fg(Color::Yellow),
            Span::raw(truncate_row_text(
                category,
                category_symbol,
                "",
                list_area.width as usize,
            ))
            .bold()
            .fg(Color::Cyan),
        ]);

        let category_para =
            Paragraph::new(vec![category_line]).block(Block::default().borders(Borders::NONE));

        if row_index >= scroll && row_index - scroll < viewport_height {
            let category_area = Rect {
                x: list_area.x,
                y: list_area.y + (row_index - scroll) as u16,
                width: list_area.width,
                height: 1,
            };
            f.render_widget(category_para, category_area);

            if current_row == Some(row_index) {
                // Highlight the category header the cursor is on
                let highlight = Block::default().style(Style::new().bg(Color::DarkGray));
                f.render_widget(highlight, category_area);
            }
        }

        row_index += 1;

        // Draw items if category is not collapsed
        if !is_collapsed {
            for (_, item) in items {
                let is_selected = item.selected;
                let is_current = current_row == Some(row_index);

                let item_style = if is_current {
                    Style::new().fg(Color::Yellow).bg(Color::DarkGray)
                } else {
                    Style::new()
                };

                let selection_symbol = if is_selected { "✓" } else { "○" };

                let quantity_suffix = format!(" x{}", item.quantity);
                let item_line = Line::from(vec![
                    Span::raw("  "),
                    Span::raw(selection_symbol).fg(if is_selected {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                    Span::raw(" "),
                    Span::raw(truncate_row_text(
                        &item.name,
                        "  ✓ ",
                        &quantity_suffix,
                        list_area.width as usize,
                    ))
                    .style(item_style),
                    Span::raw(" x"),
                    Span::raw(item.quantity.to_string()).bold(),
                ]);

                let item_para =
                    Paragraph::new(vec![item_line]).block(Block::default().borders(Borders::NONE));

                if row_index >= scroll && row_index - scroll < viewport_height {
                    let item_area = Rect {
                        x: list_area.x,
                        y: list_area.y + (row_index - scroll) as u16,
                        width: list_area.width,
                        height: 1,
                    };
                    f.render_widget(item_para, item_area);
                }
                row_index += 1;
            }
        }
    }

    // Scrollbar when the content overflows the viewport
    if total_rows > viewport_height {
        let mut scrollbar_state = ScrollbarState::new(total_rows)
            .position(scroll)
            .viewport_content_length(viewport_height);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            list_area,
            &mut scrollbar_state,
        );
    }

    // Status line: the most recent error/info message, if any
    if let Some(status) = &state.status {
        let line = Line::from(vec![
            Span::raw(" "),
            Span::raw(status.message.clone()).style(match status.kind {
                StatusKind::Error => Style::new().fg(Color::Red).bold(),
            }),
        ]);
        f.render_widget(Paragraph::new(line), status_area);
    }

    // Navigation hints: fixed footer, always visible
    let hints = Paragraph::new(hint_lines).wrap(Wrap { trim: true });
    f.render_widget(hints, hints_area);
}

// Whether every item of a category is packed; an empty category counts
// as all-packed (vacuous truth). Used for the compiled list's category
// header indicator.
pub(crate) fn category_all_packed(packed_states: &[bool]) -> bool {
    packed_states.iter().all(|&p| p)
}

/// Truncate a row's text so the whole row fits the panel width,
/// with a visible `…` indicator when it does not (spec 006, US1/AC3
/// — nothing truncates silently). `prefix` and `suffix` are the
/// fixed-width parts around the text (e.g. "  ✓ " and " x6").
pub(crate) fn truncate_row_text(text: &str, prefix: &str, suffix: &str, width: usize) -> String {
    let prefix_len = prefix.chars().count();
    let suffix_len = suffix.chars().count();
    let available = width.saturating_sub(prefix_len + suffix_len);
    let text_len = text.chars().count();
    if text_len <= available {
        text.to_string()
    } else if available == 0 {
        String::new()
    } else {
        // Keep available-1 chars plus the ellipsis
        let kept: String = text.chars().take(available.saturating_sub(1)).collect();
        format!("{kept}…")
    }
}

fn draw_selected_items_panel(
    f: &mut ratatui::prelude::Frame,
    area: Rect,
    state: &mut AppDrawState,
) {
    // The hint footer's height adapts to the panel width: entries are
    // packed into as few rows as fit, so every binding stays visible.
    // When the packed rows would squeeze the list below its minimum
    // (spec 006, US5), the hints collapse to the single `? Help` line.
    let entries = hint_entries(HintSurface::RightPanel);
    let hint_lines = if hint_rows_fit_budget(entries, area.width as usize, 3) {
        hint_rows(entries, area.width as usize)
    } else {
        vec![collapsed_hint_line()]
    };
    let hints_height = hint_lines.len() as u16;
    let [title_area, list_area, hints_area, stats_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(hints_height),
            Constraint::Length(5),
        ])
        .split(area)
    else {
        return;
    };

    // Title with side indicator
    let title = if state.focus_side == FocusSide::Right {
        Paragraph::new("Selected Items [RIGHT]")
            .style(Style::new().bold().fg(Color::Cyan))
            .block(Block::default().borders(Borders::BOTTOM))
    } else {
        Paragraph::new("Selected Items [RIGHT]")
            .style(Style::new().bold().fg(Color::White))
            .block(Block::default().borders(Borders::BOTTOM))
    };
    f.render_widget(title, title_area);

    // Get only selected items
    let selected_items: Vec<(usize, &Item)> = state
        .packing_list
        .items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.selected)
        .collect();

    // Cursor position within the panel's item rows (only when the right side is focused)
    let current_row = if state.focus_side == FocusSide::Right {
        Some(state.selected_index)
    } else {
        None
    };

    // Group selected items by category
    let categories_map = group_items_by_category(selected_items.iter().copied(), |item| {
        vec![item.primary_category().to_string()]
    });

    // Drawn rows: one header per category plus its items
    let mut drawn_rows: Vec<PanelRow> = Vec::new();
    for (category, items) in &categories_map {
        drawn_rows.push(PanelRow::Category(category.clone()));
        for (idx, _) in items {
            drawn_rows.push(PanelRow::Item(*idx));
        }
    }
    let total_rows = drawn_rows.len();

    // Map the cursor (an item-only index) to its position among the drawn rows
    let cursor_drawn_row = current_row.and_then(|cursor| {
        let item_order = state.right_item_order();
        let cursor_item = item_order.get(cursor).copied();
        cursor_item.and_then(|item| {
            drawn_rows
                .iter()
                .position(|r| matches!(r, PanelRow::Item(i) if *i == item))
        })
    });

    // Visible window: keep the cursor row in view, otherwise keep the current offset
    let viewport_height = list_area.height as usize;
    let mut scroll = state
        .right_scroll
        .min(total_rows.saturating_sub(viewport_height));
    if let Some(cursor) = cursor_drawn_row {
        if cursor < scroll {
            scroll = cursor;
        } else if cursor >= scroll + viewport_height {
            scroll = cursor + 1 - viewport_height;
        }
    }
    scroll = scroll.min(total_rows.saturating_sub(viewport_height));
    state.right_scroll = scroll;

    for (row_index, row) in drawn_rows.iter().enumerate() {
        let in_view = row_index >= scroll && row_index - scroll < viewport_height;

        match row {
            PanelRow::Category(category) => {
                // Packing indicator on the compiled list's category header:
                // all-packed (✓ green) vs. partially packed (○ red)
                let packed_states: Vec<bool> = state
                    .packing_list
                    .items
                    .iter()
                    .filter(|item| item.selected && item.in_category(category))
                    .map(|item| item.packed)
                    .collect();
                let all_packed = category_all_packed(&packed_states);
                let (marker, marker_color) = if all_packed {
                    ("✓ ", Color::Green)
                } else {
                    ("○ ", Color::Red)
                };

                let category_line = Line::from(vec![
                    Span::raw(marker).fg(marker_color),
                    Span::raw("▼ "),
                    Span::raw(truncate_row_text(
                        category,
                        &format!("{marker}▼ "),
                        "",
                        list_area.width as usize,
                    ))
                    .bold()
                    .fg(Color::Cyan),
                ]);

                let category_para = Paragraph::new(vec![category_line])
                    .block(Block::default().borders(Borders::NONE));

                if in_view {
                    let category_area = Rect {
                        x: list_area.x,
                        y: list_area.y + (row_index - scroll) as u16,
                        width: list_area.width,
                        height: 1,
                    };
                    f.render_widget(category_para, category_area);
                }
            }
            PanelRow::Item(item_index) => {
                let item = &state.packing_list.items[*item_index];
                let is_current = cursor_drawn_row == Some(row_index);

                let item_style = if is_current {
                    Style::new().fg(Color::Yellow).bg(Color::DarkGray)
                } else {
                    Style::new()
                };

                let item_symbol = if item.packed { "✓ " } else { "○ " };

                let quantity_suffix = format!(" x{}", item.quantity);
                let item_line = Line::from(vec![
                    Span::raw("  "),
                    Span::raw(item_symbol).fg(if item.packed {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                    Span::raw(" "),
                    Span::raw(truncate_row_text(
                        &item.name,
                        "  ✓ ",
                        &quantity_suffix,
                        list_area.width as usize,
                    ))
                    .style(item_style),
                    Span::raw(" x"),
                    Span::raw(item.quantity.to_string()).bold(),
                ]);

                let item_para =
                    Paragraph::new(vec![item_line]).block(Block::default().borders(Borders::NONE));

                if in_view {
                    let item_area = Rect {
                        x: list_area.x,
                        y: list_area.y + (row_index - scroll) as u16,
                        width: list_area.width,
                        height: 1,
                    };
                    f.render_widget(item_para, item_area);
                }
            }
        }
    }

    // Scrollbar when the content overflows the viewport
    if total_rows > viewport_height {
        let mut scrollbar_state = ScrollbarState::new(total_rows)
            .position(scroll)
            .viewport_content_length(viewport_height);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            list_area,
            &mut scrollbar_state,
        );
    }

    // Stats
    let total_selected = selected_items.len();
    let total_packing = state.packing_list.items.len();
    let packed = selected_items
        .iter()
        .filter(|(_, item)| item.packed)
        .count();

    let stats = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Total: ").bold(),
            Span::raw(total_selected.to_string()).fg(Color::White),
            Span::raw(" / "),
            Span::raw(total_packing.to_string()).fg(Color::Gray),
        ]),
        Line::from(vec![
            Span::raw("Packed: ").bold().fg(Color::Green),
            Span::raw(packed.to_string()).fg(Color::Green),
            Span::raw(" | "),
            Span::raw("Unpacked: ").bold().fg(Color::Red),
            Span::raw((total_selected - packed).to_string()).fg(Color::Red),
        ]),
        Line::from(vec![
            Span::raw("Progress: "),
            Span::raw(format!(
                "{:.0}%",
                if total_selected > 0 {
                    packed as f32 / total_selected as f32 * 100.0
                } else {
                    0.0
                }
            ))
            .bold(),
        ]),
    ])
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(stats, stats_area);

    // Navigation hints: fixed footer row, always visible
    let hints = Paragraph::new(hint_lines).wrap(Wrap { trim: true });
    f.render_widget(hints, hints_area);
}
