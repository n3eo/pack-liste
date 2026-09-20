use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fs;

use crate::ui::config_io::{AppConfig, load_config};
use crate::ui::input::App;
use crate::ui::state::Screen;

impl App {
    /// Open the raw config file editor, loading the current file content
    pub(crate) fn open_config_editor(&mut self) -> Result<()> {
        if self.draw_state.screen == Screen::ConfigEditor {
            return Ok(());
        }
        let text = fs::read_to_string(&self.config_path).context("Failed to read config file")?;
        self.draw_state.config_editor_text = text.lines().map(|l| l.to_string()).collect();
        if self.draw_state.config_editor_text.is_empty() {
            self.draw_state.config_editor_text.push(String::new());
        }
        self.draw_state.config_editor_cursor = (0, 0);
        self.draw_state.config_editor_scroll = 0;
        self.draw_state.config_editor_error = None;
        self.draw_state.previous_screen = self.draw_state.screen;
        self.draw_state.screen = Screen::ConfigEditor;
        Ok(())
    }

    /// Key handling of the raw config editor screen
    pub(crate) fn handle_config_editor_input(&mut self, key: KeyEvent) -> Result<bool> {
        let (line, col) = self.draw_state.config_editor_cursor;

        match key.code {
            KeyCode::Esc => {
                // Discard edits and return
                self.draw_state.screen = self.draw_state.previous_screen;
            }
            KeyCode::Enter => {
                // Split the current line at the cursor and insert a new line
                let lines = &mut self.draw_state.config_editor_text;
                if line < lines.len() {
                    let (head, tail) = lines[line].split_at(col.min(lines[line].len()));
                    let head = head.to_string();
                    let tail = tail.to_string();
                    lines[line] = head;
                    lines.insert(line + 1, tail);
                    self.draw_state.config_editor_cursor = (line + 1, 0);
                }
            }
            KeyCode::Backspace => {
                let lines = &mut self.draw_state.config_editor_text;
                if col > 0 {
                    lines[line].remove(col - 1);
                    self.draw_state.config_editor_cursor = (line, col - 1);
                } else if line > 0 {
                    // Join with the previous line
                    let current = lines.remove(line);
                    let prev_len = lines[line - 1].len();
                    lines[line - 1].push_str(&current);
                    self.draw_state.config_editor_cursor = (line - 1, prev_len);
                }
            }
            KeyCode::Up => {
                if line > 0 {
                    let col = col.min(self.draw_state.config_editor_text[line - 1].len());
                    self.draw_state.config_editor_cursor = (line - 1, col);
                }
            }
            KeyCode::Down => {
                if line + 1 < self.draw_state.config_editor_text.len() {
                    let col = col.min(self.draw_state.config_editor_text[line + 1].len());
                    self.draw_state.config_editor_cursor = (line + 1, col);
                }
            }
            KeyCode::Left => {
                if col > 0 {
                    self.draw_state.config_editor_cursor = (line, col - 1);
                } else if line > 0 {
                    let prev_len = self.draw_state.config_editor_text[line - 1].len();
                    self.draw_state.config_editor_cursor = (line - 1, prev_len);
                }
            }
            KeyCode::Right => {
                if col < self.draw_state.config_editor_text[line].len() {
                    self.draw_state.config_editor_cursor = (line, col + 1);
                } else if line + 1 < self.draw_state.config_editor_text.len() {
                    self.draw_state.config_editor_cursor = (line + 1, 0);
                }
            }
            KeyCode::Home => {
                self.draw_state.config_editor_cursor = (line, 0);
            }
            KeyCode::End => {
                let len = self.draw_state.config_editor_text[line].len();
                self.draw_state.config_editor_cursor = (line, len);
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+S saves and applies the edited config
                self.save_config_editor()?;
            }
            KeyCode::Char(c) => {
                let lines = &mut self.draw_state.config_editor_text;
                let col = col.min(lines[line].len());
                lines[line].insert(col, c);
                self.draw_state.config_editor_cursor = (line, col + 1);
            }
            _ => {}
        }

        Ok(false)
    }

    /// Save the edited config text, reload it into the app state, and return
    /// to the previous screen. On a parse error the editor stays open and
    /// shows the error.
    fn save_config_editor(&mut self) -> Result<()> {
        let text = self.draw_state.config_editor_text.join("\n");

        // Validate by parsing before writing anything to disk
        if let Err(e) = serde_yaml::from_str::<AppConfig>(&text) {
            self.draw_state.config_editor_error = Some(format!("YAML error: {}", e));
            return Ok(());
        }

        if let Err(e) = fs::write(&self.config_path, &text) {
            self.draw_state.config_editor_error = Some(format!("Write error: {}", e));
            return Ok(());
        }

        // Snapshot the trip state (selection/packed) so reloading the input
        // config does not reset the current trip
        let trip_state: Vec<(String, bool, bool)> = self
            .draw_state
            .packing_list
            .items
            .iter()
            .map(|item| (item.name.clone(), item.selected, item.packed))
            .collect();

        // Reload the saved config into the app state
        let (packing_list, trip_params) = load_config(&self.config_path)?;
        self.draw_state.packing_list = packing_list;
        self.draw_state.weekdays = trip_params.weekdays;
        self.draw_state.travel_style = trip_params.travel_style;
        self.draw_state.weather = trip_params.weather;
        self.draw_state.modifiers = trip_params.modifiers;

        // Restore the trip state for items that still exist (by name)
        for item in &mut self.draw_state.packing_list.items {
            if let Some((_, selected, packed)) = trip_state
                .iter()
                .find(|(name, _, _)| name.eq_ignore_ascii_case(&item.name))
            {
                item.selected = *selected;
                item.packed = *packed;
            }
        }

        self.reconcile_selections();
        self.update_quantities()?;
        self.draw_state.config_editor_error = None;
        self.draw_state.screen = self.draw_state.previous_screen;
        Ok(())
    }
}
