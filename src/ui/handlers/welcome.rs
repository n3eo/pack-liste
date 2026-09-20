use anyhow::Result;
use crossterm::event::KeyEvent;

use crate::ui::input::App;
use crate::ui::state::{Focus, FocusSide, InputMode, Screen};

/// Key handling of the welcome screen (file field editing, value
/// adjustment, focus movement) and the resume prompt.
pub(crate) fn handle_welcome_input(app: &mut App, key: KeyEvent) -> Result<bool> {
    use crossterm::event::KeyCode as K;
    use crossterm::event::KeyModifiers as M;

    // Resume prompt: Y/Enter loads the previous trip file,
    // N/Esc declines and starts a fresh list
    if app.draw_state.resume_prompt {
        return handle_resume_prompt(app, key);
    }

    // Trip file path editing
    if app.draw_state.editing_trip_file {
        return handle_trip_file_editor(app, key);
    }

    // Left/Right adjust the value of the focused field
    if let K::Left | K::Right = key.code {
        let direction = if key.code == K::Right { 1 } else { -1 };
        match app.draw_state.focus {
            Focus::TripFile => {
                // No value to adjust; Enter opens the file name editor
            }
            Focus::Weekdays => {
                app.adjust_weekdays(direction)?;
            }
            Focus::TravelStyle => {
                app.cycle_travel_style(direction);
                app.update_quantities()?;
            }
            Focus::Weather => {
                app.cycle_weather(direction);
                app.update_quantities()?;
            }
        }
        return Ok(false);
    }

    match key.code {
        K::Enter => {
            // Enter on the file field opens its editor;
            // a file path is required to continue
            if app.draw_state.focus == Focus::TripFile
                || app.draw_state.trip_file_path.as_os_str().is_empty()
            {
                app.draw_state.trip_file_input =
                    app.draw_state.trip_file_path.display().to_string();
                app.draw_state.editing_trip_file = true;
            } else {
                // Move to main screen after configuring
                app.draw_state.screen = Screen::Main;
                app.draw_state.focus_side = FocusSide::Left;
                app.draw_state.selected_index = 0;
            }
        }
        K::Tab => {
            app.draw_state.focus = app.draw_state.focus.next();
        }
        K::BackTab => {
            app.draw_state.focus = app.draw_state.focus.prev();
        }
        K::Up => {
            app.draw_state.focus = app.draw_state.focus.prev();
        }
        K::Down => {
            app.draw_state.focus = app.draw_state.focus.next();
        }
        K::Char('f') | K::Char('F') => {
            // Edit the trip file path
            app.draw_state.trip_file_input = app.draw_state.trip_file_path.display().to_string();
            app.draw_state.editing_trip_file = true;
        }
        K::Char('+') => {
            if app.draw_state.focus == Focus::Weekdays {
                app.adjust_weekdays(1)?;
            }
        }
        K::Char('-') => {
            if app.draw_state.focus == Focus::Weekdays {
                app.adjust_weekdays(-1)?;
            }
        }
        K::Char(' ') if key.modifiers.contains(M::CONTROL) => {
            // Ctrl+Space is main-screen only; nothing to do here
        }
        _ => {}
    }
    Ok(false)
}

/// The resume prompt shown on the welcome screen when a remembered
/// trip file exists.
fn handle_resume_prompt(app: &mut App, key: KeyEvent) -> Result<bool> {
    use crossterm::event::KeyCode as K;
    match key.code {
        K::Char('y') | K::Char('Y') | K::Enter => {
            app.draw_state.resume_prompt = false;
            if let Some(path) = app.draw_state.remembered_trip_file.clone() {
                let input = path.display().to_string();
                app.apply_trip_file_path(&input)?;
            }
        }
        K::Char('n') | K::Char('N') | K::Esc => {
            app.draw_state.resume_prompt = false;
            // Fresh list: keep everything selected; the
            // remembered path stays saved for the next run
            app.select_all_items();
            app.update_quantities()?;
        }
        _ => {}
    }
    Ok(false)
}

/// The trip-file-path text editor on the welcome screen.
fn handle_trip_file_editor(app: &mut App, key: KeyEvent) -> Result<bool> {
    use crossterm::event::KeyCode as K;
    match key.code {
        K::Enter => {
            let input = app.draw_state.trip_file_input.trim().to_string();
            // The file path is required: empty input keeps the editor open
            if !input.is_empty() {
                app.apply_trip_file_path(&input)?;
            }
        }
        K::Esc => {
            app.draw_state.editing_trip_file = false;
        }
        K::Backspace => {
            app.draw_state.trip_file_input.pop();
        }
        K::Char(c) => {
            app.draw_state.trip_file_input.push(c);
        }
        _ => {}
    }
    Ok(false)
}

/// Key handling of the main screen, normal mode: navigation, selection,
/// packed toggling, collapse, and the add/edit/delete/recalculate
/// entry points.
pub(crate) fn handle_main_input(app: &mut App, key: KeyEvent) -> Result<bool> {
    use crossterm::event::KeyCode as K;
    use crossterm::event::KeyModifiers as M;

    // Ctrl+Space toggles the category's selection (left panel)
    if key.modifiers.contains(M::CONTROL) && key.code == K::Char(' ') {
        if app.draw_state.focus_side == FocusSide::Left {
            app.toggle_category_selection();
        }
        return Ok(false);
    }

    match key.code {
        K::Tab => {
            // Switch between left and right panels
            app.switch_focus_side();
        }
        K::BackTab => {
            app.switch_focus_side();
        }
        K::Up => {
            app.handle_up()?;
        }
        K::Down => {
            app.handle_down()?;
        }
        K::Enter | K::Char(' ') => {
            // Toggle selection (left) / packed (right)
            if app.draw_state.focus_side == FocusSide::Left {
                app.toggle_item_selection();
            } else {
                app.toggle_item_packed_right_side();
            }
        }
        K::Char('c') | K::Char('C') => {
            if app.draw_state.focus_side == FocusSide::Left {
                app.toggle_category_collapse();
            }
        }
        K::Char('a') | K::Char('A') => {
            if app.draw_state.focus_side == FocusSide::Left {
                app.start_adding_item();
            }
        }
        K::Char('e') | K::Char('E') => {
            if app.draw_state.focus_side == FocusSide::Left {
                app.start_editing_item();
            }
        }
        K::Char('d') | K::Char('D') => {
            if app.draw_state.focus_side == FocusSide::Left {
                // Ask for confirmation before deleting
                if let Some(item_index) = app.current_item_index() {
                    app.draw_state.input_mode = InputMode::ConfirmDelete { index: item_index };
                }
            }
        }
        K::Char('r') | K::Char('R') => {
            // Recalculate the current list's quantities (spec 018):
            // works from both list views, unbound everywhere else
            app.update_quantities()?;
        }
        _ => {}
    }
    Ok(false)
}

/// Key handling of the add/edit item dialogs and the delete
/// confirmation popup.
pub(crate) fn handle_dialog_input(app: &mut App, key: KeyEvent) -> Result<bool> {
    use crossterm::event::KeyCode as K;
    match &mut app.draw_state.input_mode {
        InputMode::AddingItem { .. } | InputMode::EditingItem { .. } => {
            match key.code {
                K::Enter => {
                    app.save_open_dialog();
                    app.draw_state.input_mode = InputMode::Normal;
                }
                K::Tab => {
                    app.cycle_dialog_field();
                }
                K::Up | K::Down => {
                    // Move the category dropdown highlight (category field)
                    if key.code == K::Up {
                        app.draw_state.move_cursor_up();
                    } else {
                        app.draw_state.move_cursor_down();
                    }
                }
                K::Left | K::Right => {
                    // Cycle the category selection (category field)
                    let direction = if key.code == K::Right { 1 } else { -1 };
                    app.draw_state.cycle_category_selection(direction);
                }
                K::Char(' ') => {
                    app.handle_dialog_space();
                }
                K::Char(c) => {
                    app.handle_dialog_char(c);
                }
                K::Backspace => {
                    app.handle_dialog_backspace();
                }
                _ => {}
            }
            Ok(false)
        }
        InputMode::ConfirmDelete { index } => {
            if key.code == K::Enter {
                let index = *index;
                app.remove_item_confirmed(index);
                app.draw_state.input_mode = InputMode::Normal;
                // Save config after deletion
                app.save_config()?;
            }
            Ok(false)
        }
        InputMode::Normal => Ok(false),
    }
}
