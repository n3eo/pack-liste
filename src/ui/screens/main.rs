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
use crate::ui::state::{AppDrawState, FocusSide, PanelRow, StatusKind, sticky_rows};

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
    // stacked, or the too-small notice. One shared hint footer spans
    // the full width below the panels (spec 023, US2 — every binding
    // stated once for the whole screen).
    let footer_height = {
        let entries = hint_entries(HintSurface::MainScreen);
        let hint_lines = if hint_rows_fit_budget(entries, size.width as usize, 3) {
            hint_rows(entries, size.width as usize)
        } else {
            vec![collapsed_hint_line()]
        };
        hint_lines.len() as u16
    };
    let [panels_area, footer_area] = *Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(footer_height)])
        .split(size)
    else {
        return;
    };

    match layout_mode(panels_area.width, panels_area.height) {
        LayoutMode::TooSmall => {
            draw_too_small_notice(f, size);
            return;
        }
        LayoutMode::SideBySide => {
            // Split screen: left (all items), right (selected items only)
            let [left_area, right_area] = *Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(panels_area)
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
                .split(panels_area)
            else {
                return;
            };
            draw_all_items_panel(f, top_area, state);
            draw_selected_items_panel(f, bottom_area, state);
        }
    }

    // The shared footer: the main screen's complete binding set,
    // stated once (spec 023, US2)
    let entries = hint_entries(HintSurface::MainScreen);
    let hint_lines = if hint_rows_fit_budget(entries, footer_area.width as usize, 3) {
        hint_rows(entries, footer_area.width as usize)
    } else {
        vec![collapsed_hint_line()]
    };
    let hints = Paragraph::new(hint_lines).wrap(Wrap { trim: true });
    f.render_widget(hints, footer_area);
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
    // One extra row for the status line when a message is pending
    let status_height = u16::from(state.status.is_some());
    let [title_area, list_area, status_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(status_height),
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

    // The visible slice with a sticky category header: no item is
    // ever orphaned from its group when scrolled mid-category. The
    // pin displaces the window's last row — when the cursor is on
    // the displaced row, scroll one further so the cursor stays
    // visible (repeated until it is, or the scroll maxes out).
    let max_scroll = total_rows.saturating_sub(viewport_height);
    let mut visible_rows = sticky_rows(&rows, scroll, viewport_height);
    if let Some(cursor) = current_row {
        while scroll > 0
            && scroll < max_scroll
            && rows.get(cursor).is_some()
            && cursor >= scroll
            && !visible_rows.iter().any(|(pos, _)| *pos == cursor)
        {
            scroll += 1;
            visible_rows = sticky_rows(&rows, scroll, viewport_height);
        }
    }
    state.left_scroll = scroll;

    for (visible_index, (row_pos, row)) in visible_rows.iter().enumerate() {
        let row_area = Rect {
            x: list_area.x,
            y: list_area.y + visible_index as u16,
            width: list_area.width,
            height: 1,
        };
        match row {
            PanelRow::Label(label) => {
                // The flat label header (spec 025): the collapse
                // symbol and name
                let is_collapsed = state.collapsed_labels.contains(label);
                let label_symbol = if is_collapsed { "▶ " } else { "▼ " };
                let label_line = Line::from(vec![
                    Span::raw(label_symbol).fg(Color::Yellow),
                    Span::raw(truncate_row_text(
                        label,
                        label_symbol,
                        "",
                        list_area.width as usize,
                    ))
                    .bold()
                    .fg(Color::Cyan),
                ]);
                let label_para =
                    Paragraph::new(vec![label_line]).block(Block::default().borders(Borders::NONE));
                f.render_widget(label_para, row_area);
                // Highlight the label header the cursor is on
                if current_row == Some(*row_pos) {
                    let highlight = Block::default().style(Style::new().bg(Color::DarkGray));
                    f.render_widget(highlight, row_area);
                }
            }
            PanelRow::Item(item_index) => {
                let item = &state.packing_list.items[*item_index];
                let is_current = current_row == Some(*row_pos);

                let item_style = if is_current {
                    Style::new().fg(Color::Yellow).bg(Color::DarkGray)
                } else {
                    Style::new()
                };

                // The occurrence's effective selection under the
                // current label (spec 025, FR-012): the override if
                // present, else the item's own state
                let current_label = rows
                    .get(*row_pos)
                    .and_then(|_| {
                        rows[..*row_pos].iter().rev().find_map(|r| match r {
                            PanelRow::Label(l) => Some(l.clone()),
                            _ => None,
                        })
                    })
                    .unwrap_or_default();
                let occurrence_selected = state.effective_selected(*item_index, &current_label);
                let selection_symbol = if occurrence_selected { "✓" } else { "○" };

                let quantity_suffix = format!(" x{}", item.quantity);
                // The chips render between the name and the quantity
                // (SC-009): their width joins the truncation budget so
                // the quantity never leaves the row
                let chips =
                    label_chip_spans(state, *item_index, &current_label, occurrence_selected);
                let chips_width: usize = chips.iter().map(|s| s.content.chars().count()).sum();
                // The budget: everything after the name — the chips
                // plus " x<quantity>"
                let suffix_budget = " ".repeat(chips_width + quantity_suffix.chars().count());
                let mut spans = vec![
                    Span::raw("  "),
                    Span::raw(selection_symbol).fg(if occurrence_selected {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                    Span::raw(" "),
                    Span::raw(truncate_row_text(
                        &item.name,
                        "  ✓ ",
                        &suffix_budget,
                        list_area.width as usize,
                    ))
                    .style(item_style),
                ];
                spans.extend(chips);
                spans.push(Span::raw(" x"));
                spans.push(Span::raw(item.quantity.to_string()).bold());
                let item_line = Line::from(spans);

                let item_para =
                    Paragraph::new(vec![item_line]).block(Block::default().borders(Borders::NONE));
                f.render_widget(item_para, row_area);
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
}

/// The label chips for an item row (spec 025, US5): one span per
/// label, ` [Label]` — dark grey when the occurrence's effective
/// state matches the item's own on both dimensions (selection and
/// packed), light grey when either differs (only possible for a
/// non-singleton with an override). A singleton's overrides are
/// always clear, so its chips are uniformly dark.
pub(crate) fn label_chip_spans(
    state: &AppDrawState,
    item_index: usize,
    current_label: &str,
    item_state: bool,
) -> Vec<Span<'static>> {
    let item = &state.packing_list.items[item_index];
    item.labels
        .iter()
        .map(|label| {
            // The occurrence's effective states: the row's displayed
            // state under the current label, the override (or the
            // item's own state) under the others
            let (occurrence_selected, occurrence_packed) = if label == current_label {
                (item_state, item.packed)
            } else {
                (
                    state.effective_selected(item_index, label),
                    state.effective_packed(item_index, label),
                )
            };
            // Dark when the occurrence matches the item's own state
            // on both dimensions
            let matches = occurrence_selected == item.selected && occurrence_packed == item.packed;
            let color = if matches {
                Color::DarkGray
            } else {
                Color::Gray
            };
            Span::raw(format!(" [{label}]")).fg(color)
        })
        .collect()
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
    let [title_area, list_area, stats_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(10),
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

    // Cursor position within the panel's rows (only when the right
    // side is focused): the same tree the panel renders, so the
    // cursor walks headers and items alike (spec 024)
    let current_row = if state.focus_side == FocusSide::Right {
        Some(state.selected_index)
    } else {
        None
    };

    // The two-layer tree over the selected items (spec 024): the
    // shared row model — the cursor and the renderer walk the same
    // rows, so every header is reachable by navigation
    let drawn_rows = state.right_rows();
    let total_rows = drawn_rows.len();

    // The cursor's position among the drawn rows (a direct row index)
    let cursor_drawn_row = current_row.filter(|cursor| *cursor < total_rows);

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

    // The visible slice with a sticky category header: no item is
    // ever orphaned from its group when scrolled mid-category. The
    // pin displaces the window's last row — when the cursor is on
    // the displaced row, scroll one further so the cursor stays
    // visible (repeated until it is, or the scroll maxes out).
    let max_scroll = total_rows.saturating_sub(viewport_height);
    let mut visible_rows = sticky_rows(&drawn_rows, scroll, viewport_height);
    if let Some(cursor) = cursor_drawn_row {
        while scroll > 0
            && scroll < max_scroll
            && drawn_rows.get(cursor).is_some()
            && cursor >= scroll
            && !visible_rows.iter().any(|(pos, _)| *pos == cursor)
        {
            scroll += 1;
            visible_rows = sticky_rows(&drawn_rows, scroll, viewport_height);
        }
    }
    state.right_scroll = scroll;

    for (visible_index, (row_pos, row)) in visible_rows.iter().enumerate() {
        let row_area = Rect {
            x: list_area.x,
            y: list_area.y + visible_index as u16,
            width: list_area.width,
            height: 1,
        };

        match row {
            PanelRow::Label(label) => {
                // The flat label header (spec 025): the packing
                // indicator computed over every selected item's
                // occurrence under this label (the effective packed
                // state, spec 025 FR-012)
                let packed_states: Vec<bool> = state
                    .packing_list
                    .items
                    .iter()
                    .enumerate()
                    .filter(|(_, item)| item.selected && item.has_label(label))
                    .map(|(idx, _)| state.effective_packed(idx, label))
                    .collect();
                let all_packed = category_all_packed(&packed_states);
                let (marker, marker_color) = if all_packed {
                    ("✓ ", Color::Green)
                } else {
                    ("○ ", Color::Red)
                };
                let label_line = Line::from(vec![
                    Span::raw(marker).fg(marker_color),
                    Span::raw("▼ "),
                    Span::raw(truncate_row_text(
                        label,
                        &format!("{marker}▼ "),
                        "",
                        list_area.width as usize,
                    ))
                    .bold()
                    .fg(Color::Cyan),
                ]);
                let label_para =
                    Paragraph::new(vec![label_line]).block(Block::default().borders(Borders::NONE));
                f.render_widget(label_para, row_area);
                // Highlight the label header the cursor is on
                if cursor_drawn_row == Some(*row_pos) {
                    let highlight = Block::default().style(Style::new().bg(Color::DarkGray));
                    f.render_widget(highlight, row_area);
                }
            }
            PanelRow::Item(item_index) => {
                let item = &state.packing_list.items[*item_index];
                let is_current = cursor_drawn_row == Some(*row_pos);

                let item_style = if is_current {
                    Style::new().fg(Color::Yellow).bg(Color::DarkGray)
                } else {
                    Style::new()
                };

                // The occurrence's effective packed state under the
                // current label (spec 025, FR-012): the override if
                // present, else the item's own state
                let current_label = drawn_rows
                    .get(*row_pos)
                    .and_then(|_| {
                        drawn_rows[..*row_pos].iter().rev().find_map(|r| match r {
                            PanelRow::Label(l) => Some(l.clone()),
                            _ => None,
                        })
                    })
                    .unwrap_or_default();
                let occurrence_packed = state.effective_packed(*item_index, &current_label);
                let item_symbol = if occurrence_packed { "✓ " } else { "○ " };

                let quantity_suffix = format!(" x{}", item.quantity);
                // The chips render between the name and the quantity
                // (SC-009): their width joins the truncation budget so
                // the quantity never leaves the row
                let chips = label_chip_spans(state, *item_index, &current_label, item.selected);
                let chips_width: usize = chips.iter().map(|s| s.content.chars().count()).sum();
                // The budget: everything after the name — the chips
                // plus " x<quantity>"
                let suffix_budget = " ".repeat(chips_width + quantity_suffix.chars().count());
                let mut spans = vec![
                    Span::raw("  "),
                    Span::raw(item_symbol).fg(if occurrence_packed {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                    Span::raw(" "),
                    Span::raw(truncate_row_text(
                        &item.name,
                        "  ✓ ",
                        &suffix_budget,
                        list_area.width as usize,
                    ))
                    .style(item_style),
                ];
                spans.extend(chips);
                spans.push(Span::raw(" x"));
                spans.push(Span::raw(item.quantity.to_string()).bold());
                let item_line = Line::from(spans);

                let item_para =
                    Paragraph::new(vec![item_line]).block(Block::default().borders(Borders::NONE));
                f.render_widget(item_para, row_area);
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
}
