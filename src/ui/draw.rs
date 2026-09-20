use anyhow::Result;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::models::packing_list::PackingList;
use crate::ui::hints::{HintSurface, hint_entries, hint_line};
use crate::ui::screens::config::draw_config_screen;
use crate::ui::screens::editor::draw_config_editor_screen;
use crate::ui::screens::help::draw_help_screen;
use crate::ui::screens::main::draw_main_screen;
use crate::ui::screens::welcome::draw_welcome_screen;
use crate::ui::state::{InputField, InputMode, PopupFields, Screen};

use crate::ui::input::App;

impl App {
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
                    InputMode::AddingItem {
                        field,
                        ref name,
                        ref category,
                        ref quantity,
                        ref category_list,
                        category_index,
                    } => {
                        let fields = PopupFields {
                            field,
                            name,
                            category,
                            quantity,
                            auto_quantity: None,
                            category_list,
                            category_index,
                        };
                        draw_item_dialog(f, size, "Add New Item", &fields);
                    }
                    InputMode::EditingItem {
                        field,
                        ref name,
                        ref category,
                        ref quantity,
                        auto_quantity,
                        ref category_list,
                        category_index,
                        ..
                    } => {
                        let fields = PopupFields {
                            field,
                            name,
                            category,
                            quantity,
                            auto_quantity: Some(auto_quantity),
                            category_list,
                            category_index,
                        };
                        draw_item_dialog(f, size, "Edit Item", &fields);
                    }
                    InputMode::ConfirmDelete { index } => {
                        draw_confirm_delete_popup(f, size, index, &state.packing_list);
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
    let popup_height = 14;

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

    // Draw category field
    let category_area = Rect {
        x: content_area.x,
        y: content_area.y + 1,
        width: content_area.width,
        height: 1,
    };
    let category_line = Line::from(vec![Span::raw("Category: ").style(
        if fields.field == InputField::Category {
            field_style
        } else {
            normal_style
        },
    )]);
    let category_para = Paragraph::new(vec![category_line]);
    f.render_widget(category_para, category_area);

    // Draw category dropdown or selected category
    let dropdown_area = Rect {
        x: content_area.x + 10,
        y: content_area.y + 1,
        width: content_area.width - 10,
        height: 1,
    };

    // Draw quantity field
    let quantity_area = Rect {
        x: content_area.x,
        y: content_area.y + 2,
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
            y: content_area.y + 3,
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

    // Draw instructions
    let instructions_area = Rect {
        x: content_area.x,
        y: content_area.y + 4,
        width: content_area.width,
        height: content_area.height - 4,
    };
    let instructions = Paragraph::new(hint_line(hint_entries(HintSurface::AddEditPopup)));
    f.render_widget(instructions, instructions_area);

    // Draw category dropdown or selected category (last, so the open dropdown overlays the fields below)
    if fields.field == InputField::Category {
        // Draw dropdown list
        let category_items: Vec<ListItem> = fields
            .category_list
            .iter()
            .map(|c| ListItem::new(c.as_str()))
            .collect();

        let category_list_widget = List::new(category_items)
            .highlight_style(Style::new().fg(Color::Yellow).bold())
            .highlight_symbol("> ");

        let dropdown_height = 5.min(fields.category_list.len() as u16 + 1);
        let dropdown_list_area = Rect {
            x: dropdown_area.x,
            y: dropdown_area.y,
            width: dropdown_area.width,
            height: dropdown_height,
        };
        f.render_widget(category_list_widget, dropdown_list_area);

        // Highlight selected fields.category
        if fields.category_index < fields.category_list.len() {
            let selected_category = &fields.category_list[fields.category_index];
            let selected_line =
                Line::from(vec![Span::raw(selected_category).fg(Color::Yellow).bold()]);
            let selected_para = Paragraph::new(vec![selected_line]);
            f.render_widget(selected_para, dropdown_area);
        }
    } else {
        // Show selected fields.category text
        let selected_category = if fields.category_index < fields.category_list.len() {
            &fields.category_list[fields.category_index]
        } else {
            fields.category
        };
        let category_text = Paragraph::new(vec![Line::from(selected_category)]);
        f.render_widget(category_text, dropdown_area);
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
            Span::raw(item_name).fg(Color::Red).bold(),
            Span::raw("?"),
        ]),
        Line::from(""),
        hint_line(hint_entries(HintSurface::DeletePopup)),
    ];

    let content = Paragraph::new(lines);
    f.render_widget(content, content_area);
}
