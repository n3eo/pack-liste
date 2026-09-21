use anyhow::Result;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::models::packing_list::PackingList;
use crate::ui::hints::{HintSurface, hint_entries, hint_line, hint_rows};
use crate::ui::screens::config::draw_config_screen;
use crate::ui::screens::editor::draw_config_editor_screen;
use crate::ui::screens::help::draw_help_screen;
use crate::ui::screens::main::{draw_main_screen, truncate_row_text};
use crate::ui::screens::welcome::draw_welcome_screen;
use crate::ui::state::{
    InputField, InputMode, PendingEdit, PopupFields, QuantityBufferKind, Screen,
};

use crate::ui::input::App;

impl App {
    /// Render the current screen and persist the scroll offsets
    /// computed during rendering.
    pub(crate) fn draw(&mut self) -> Result<()> {
        // The per-frame clone is an accepted trade-off (spec 022 FR-012):
        // terminal.draw needs &mut on the closure state while App holds
        // the authoritative state, so rendering works on a clone and
        // writes the computed scroll offsets back afterwards. At
        // realistic list sizes (hundreds of items) the clone cost is
        // negligible against the 100 ms poll loop.
        let mut state = self.draw_state.clone();

        self.terminal.draw(|f| {
            let size = f.area();

            match state.screen {
                Screen::Welcome => {
                    draw_welcome_screen(f, size, &state);
                }
                Screen::Main => match state.input_mode {
                    InputMode::Normal => {
                        draw_main_screen(f, size, &mut state);
                    }
                    InputMode::ConfirmDelete { index } => {
                        draw_confirm_delete_popup(f, size, index, &state.packing_list);
                    }
                    InputMode::ConfirmEdit { ref pending } => {
                        draw_confirm_edit_popup(f, size, pending, &state.packing_list);
                    }
                    InputMode::AddingItem { .. } | InputMode::EditingItem { .. } => {
                        // The shared dialog surface (spec 021, US3):
                        // one renderer for both dialogs
                        if let Some(fields) = state.input_mode.popup_fields() {
                            let title = if matches!(state.input_mode, InputMode::EditingItem { .. })
                            {
                                "Edit Item"
                            } else {
                                "Add New Item"
                            };
                            draw_item_dialog(f, size, title, &fields);
                        }
                    }
                },
                Screen::Help => {
                    draw_help_screen(f, size);
                }
                Screen::Config => {
                    draw_config_screen(f, size, &state);
                }
                Screen::ConfigEditor => {
                    draw_config_editor_screen(f, size, &mut state);
                }
            }
        })?;

        // Persist the scroll offsets computed during rendering
        self.draw_state.left_scroll = state.left_scroll;
        self.draw_state.right_scroll = state.right_scroll;
        self.draw_state.config_editor_scroll = state.config_editor_scroll;
        Ok(())
    }
}

/// The add/edit item dialog: one renderer parameterized by the title;
/// the Mode field appears only when a quantity mode is given (edit).
fn draw_item_dialog(
    f: &mut ratatui::prelude::Frame,
    size: Rect,
    title: &str,
    fields: &PopupFields,
) {
    // Create a centered popup with dynamic size
    let popup_width = (size.width * 60 / 100).clamp(40, 60);
    // Seven field rows (Name, Labels, Singleton, Quantity, Mode,
    // Rate, Buffer) plus borders and the hint area
    let popup_height = 17;

    let popup_area = Rect {
        x: (size.width.saturating_sub(popup_width)) / 2,
        y: (size.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height.min(size.height),
    };

    // Draw background overlay
    let background = Block::default()
        .style(Style::new().bg(Color::Black).fg(Color::White))
        .borders(Borders::NONE);
    f.render_widget(background, size);

    // Draw popup
    let popup_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::new().bg(Color::Black).fg(Color::White));
    f.render_widget(popup_block, popup_area);

    // Popup content
    let content_area = Rect {
        x: popup_area.x + 1,
        y: popup_area.y + 1,
        width: popup_area.width - 2,
        height: popup_area.height - 2,
    };

    let field_style = Style::new().fg(Color::Yellow).bold();
    let normal_style = Style::new().fg(Color::White);

    // Draw name field
    let name_area = Rect {
        x: content_area.x,
        y: content_area.y,
        width: content_area.width,
        height: 1,
    };
    let name_line = Line::from(vec![
        Span::raw("Name: ").style(if fields.field == InputField::Name {
            field_style
        } else {
            normal_style
        }),
        Span::raw(fields.name),
    ]);
    let name_para = Paragraph::new(vec![name_line]);
    f.render_widget(name_para, name_area);

    // Draw the labels field (spec 025): the multi-select's set, the
    // highlighted picker entry with its membership marker, and the
    // free-text input. Left/Right cycles the highlight; Space
    // toggles membership; typed text joins on save.
    let labels_area = Rect {
        x: content_area.x,
        y: content_area.y + 1,
        width: content_area.width,
        height: 1,
    };
    let highlighted = if fields.label_index < fields.label_list.len() {
        fields.label_list[fields.label_index].as_str()
    } else {
        ""
    };
    let is_member = fields.selected_labels.iter().any(|l| l == highlighted);
    let labels_line = Line::from(vec![
        Span::raw("Labels: ").style(if fields.field == InputField::Labels {
            field_style
        } else {
            normal_style
        }),
        Span::raw(if is_member { "✓ " } else { "○ " }).fg(if is_member {
            Color::Green
        } else {
            Color::Red
        }),
        Span::raw(highlighted).style(if fields.field == InputField::Labels {
            field_style
        } else {
            normal_style
        }),
        Span::raw(format!("  [{}]{}", fields.selected_labels.join(", "), {
            if fields.label_input.is_empty() {
                String::new()
            } else {
                format!(" +{}", fields.label_input)
            }
        }))
        .dim(),
    ]);
    let labels_para = Paragraph::new(vec![labels_line]);
    f.render_widget(labels_para, labels_area);

    // Draw the singleton field (spec 025, US5): Space toggles
    let singleton_area = Rect {
        x: content_area.x,
        y: content_area.y + 2,
        width: content_area.width,
        height: 1,
    };
    let singleton_line = Line::from(vec![
        Span::raw("Singleton: ").style(if fields.field == InputField::Singleton {
            field_style
        } else {
            normal_style
        }),
        Span::raw(if fields.singleton { "yes" } else { "no" }).style(
            if fields.field == InputField::Singleton {
                field_style
            } else {
                normal_style
            },
        ),
        Span::raw("  (Space toggles; syncs all labels)").dim(),
    ]);
    let singleton_para = Paragraph::new(vec![singleton_line]);
    f.render_widget(singleton_para, singleton_area);

    // Draw quantity field
    let quantity_area = Rect {
        x: content_area.x,
        y: content_area.y + 3,
        width: content_area.width,
        height: 1,
    };
    let quantity_line = Line::from(vec![
        Span::raw("Quantity: ").style(if fields.field == InputField::Quantity {
            field_style
        } else {
            normal_style
        }),
        Span::raw(fields.quantity),
    ]);
    let quantity_para = Paragraph::new(vec![quantity_line]);
    f.render_widget(quantity_para, quantity_area);

    // Draw mode field (edit dialog only): automatic or fixed quantity
    if let Some(auto) = fields.auto_quantity {
        let mode_area = Rect {
            x: content_area.x,
            y: content_area.y + 4,
            width: content_area.width,
            height: 1,
        };
        let mode_line = Line::from(vec![
            Span::raw("Mode: ").style(if fields.field == InputField::Mode {
                field_style
            } else {
                normal_style
            }),
            Span::raw(if auto { "automatic" } else { "fixed" }).style(
                if fields.field == InputField::Mode {
                    field_style
                } else {
                    normal_style
                },
            ),
            Span::raw("  (Space toggles)").dim(),
        ]);
        let mode_para = Paragraph::new(vec![mode_line]);
        f.render_widget(mode_para, mode_area);
    }

    // Draw rate field (spec 010): the item's per-day rate
    let rate_area = Rect {
        x: content_area.x,
        y: content_area.y + 5,
        width: content_area.width,
        height: 1,
    };
    let rate_line = Line::from(vec![
        Span::raw("Rate/day: ").style(if fields.field == InputField::Rate {
            field_style
        } else {
            normal_style
        }),
        Span::raw(fields.rate),
        Span::raw("  (days x rate)").dim(),
    ]);
    let rate_para = Paragraph::new(vec![rate_line]);
    f.render_widget(rate_para, rate_area);

    // Draw buffer field (spec 010): the kind and its value
    let buffer_area = Rect {
        x: content_area.x,
        y: content_area.y + 6,
        width: content_area.width,
        height: 1,
    };
    let buffer_kind_text = match fields.buffer_kind {
        QuantityBufferKind::None => "none",
        QuantityBufferKind::Reserve => "reserve +",
        QuantityBufferKind::Multiplier => "multiplier x",
    };
    let buffer_line = Line::from(vec![
        Span::raw("Buffer: ").style(if fields.field == InputField::Buffer {
            field_style
        } else {
            normal_style
        }),
        Span::raw(buffer_kind_text).style(if fields.field == InputField::Buffer {
            field_style
        } else {
            normal_style
        }),
        Span::raw(fields.buffer_value),
        Span::raw("  (Space cycles kind)").dim(),
    ]);
    let buffer_para = Paragraph::new(vec![buffer_line]);
    f.render_widget(buffer_para, buffer_area);

    // Draw instructions: packed into rows that fit the popup width so
    // no binding is cut off mid-word
    let instructions_area = Rect {
        x: content_area.x,
        y: content_area.y + 7,
        width: content_area.width,
        height: content_area.height - 7,
    };
    let instructions = Paragraph::new(hint_rows(
        hint_entries(HintSurface::AddEditPopup),
        instructions_area.width as usize,
    ));
    f.render_widget(instructions, instructions_area);

    // Text cursor (spec 020, US2): a highlighted cell immediately
    // after the focused free-text field's text (Name, Quantity only);
    // the position is derived from the text length — no new state
    // (the raw editor's cursor-cell pattern)
    let cursor = match fields.field {
        InputField::Name => Some((name_area.y, "Name: ".len() + fields.name.chars().count())),
        InputField::Quantity => Some((
            quantity_area.y,
            "Quantity: ".len() + fields.quantity.chars().count(),
        )),
        _ => None,
    };
    if let Some((y, offset)) = cursor {
        let x = content_area.x + offset as u16;
        if x < content_area.x + content_area.width {
            let cell = Rect {
                x,
                y,
                width: 1,
                height: 1,
            };
            f.render_widget(
                Paragraph::new(" ").style(Style::new().bg(Color::Yellow).fg(Color::Yellow)),
                cell,
            );
        }
    }
}

fn draw_confirm_delete_popup(
    f: &mut ratatui::prelude::Frame,
    size: Rect,
    index: usize,
    packing_list: &PackingList,
) {
    // Create a centered popup with dynamic size
    let popup_width = (size.width * 40 / 100).clamp(30, 40);
    let popup_height = 8;

    let popup_area = Rect {
        x: (size.width.saturating_sub(popup_width)) / 2,
        y: (size.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height.min(size.height),
    };

    // Draw background overlay
    let background = Block::default()
        .style(Style::new().bg(Color::Black).fg(Color::White))
        .borders(Borders::NONE);
    f.render_widget(background, size);

    // Draw popup
    let popup_block = Block::default()
        .title("Confirm Delete")
        .borders(Borders::ALL)
        .style(Style::new().bg(Color::Black).fg(Color::White));
    f.render_widget(popup_block, popup_area);

    // Popup content
    let content_area = Rect {
        x: popup_area.x + 1,
        y: popup_area.y + 1,
        width: popup_area.width - 2,
        height: popup_area.height - 2,
    };

    let item_name = if index < packing_list.items.len() {
        &packing_list.items[index].name
    } else {
        "Unknown"
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("Delete ").bold(),
            Span::raw(truncate_row_text(
                item_name,
                "Delete ",
                "?",
                content_area.width as usize,
            ))
            .fg(Color::Red)
            .bold(),
            Span::raw("?"),
        ]),
        Line::from(""),
        hint_line(hint_entries(HintSurface::DeletePopup)),
    ];

    let content = Paragraph::new(lines);
    f.render_widget(content, content_area);
}

/// The edit-confirmation popup (spec 011): the pending change is
/// shown before it is applied — the item's name and the new values
/// (name, categories, quantity, mode; old values where space
/// allows). The delete popup's pattern: centered, min()-sized for
/// small terminals.
fn draw_confirm_edit_popup(
    f: &mut ratatui::prelude::Frame,
    size: Rect,
    pending: &PendingEdit,
    packing_list: &PackingList,
) {
    // Create a centered popup with dynamic size
    let popup_width = (size.width * 50 / 100).clamp(40, 60);
    let popup_height = 10;

    let popup_area = Rect {
        x: (size.width.saturating_sub(popup_width)) / 2,
        y: (size.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height.min(size.height),
    };

    // Draw background overlay
    let background = Block::default()
        .style(Style::new().bg(Color::Black).fg(Color::White))
        .borders(Borders::NONE);
    f.render_widget(background, size);

    // Draw popup
    let popup_block = Block::default()
        .title("Confirm Edit")
        .borders(Borders::ALL)
        .style(Style::new().bg(Color::Black).fg(Color::White));
    f.render_widget(popup_block, popup_area);

    // Popup content
    let content_area = Rect {
        x: popup_area.x + 1,
        y: popup_area.y + 1,
        width: popup_area.width - 2,
        height: popup_area.height - 2,
    };

    // The item's current values, shown where space allows (US2)
    let old = packing_list.items.get(pending.index);
    let old_name = old.map(|i| i.name.as_str()).unwrap_or("Unknown");
    let old_quantity = old.map(|i| i.quantity.to_string()).unwrap_or_default();

    let mode = if pending.auto_quantity {
        "automatic"
    } else {
        "fixed"
    };
    let new_quantity = pending
        .quantity
        .parse::<u32>()
        .map(|q| q.to_string())
        .unwrap_or_else(|_| pending.quantity.clone());

    let lines = vec![
        Line::from(vec![
            Span::raw("Save changes to ").bold(),
            Span::raw(truncate_row_text(
                old_name,
                "Save changes to ",
                "?",
                content_area.width as usize,
            ))
            .fg(Color::Yellow)
            .bold(),
            Span::raw("?"),
        ]),
        Line::from(""),
        Line::from(vec![Span::raw("Name: ").dim(), Span::raw(&pending.name)]),
        Line::from(vec![
            Span::raw("Labels: ").dim(),
            Span::raw(pending.labels.join(", ")),
        ]),
        Line::from(vec![
            Span::raw("Singleton: ").dim(),
            Span::raw(if pending.singleton { "yes" } else { "no" }),
        ]),
        Line::from(vec![
            Span::raw("Quantity: ").dim(),
            Span::raw(&new_quantity),
            Span::raw(" (was ").dim(),
            Span::raw(old_quantity).dim(),
            Span::raw(")").dim(),
        ]),
        Line::from(vec![Span::raw("Mode: ").dim(), Span::raw(mode)]),
        Line::from(""),
        hint_line(hint_entries(HintSurface::ConfirmEditPopup)),
    ];

    let content = Paragraph::new(lines);
    f.render_widget(content, content_area);
}
