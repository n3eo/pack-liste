use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::ui::input::App;
use crate::ui::state::{ConfigSection, ModifierEditor, ModifierField};

impl App {
    // The modifiers of the focused config section
    fn config_section_modifiers(
        &self,
        section: ConfigSection,
    ) -> &Vec<crate::config::modifiers::ModifierConfig> {
        match section {
            ConfigSection::TravelStyles => &self.draw_state.modifiers.travel_styles,
            ConfigSection::Weather => &self.draw_state.modifiers.weather,
        }
    }

    fn config_section_modifiers_mut(
        &mut self,
        section: ConfigSection,
    ) -> &mut Vec<crate::config::modifiers::ModifierConfig> {
        match section {
            ConfigSection::TravelStyles => &mut self.draw_state.modifiers.travel_styles,
            ConfigSection::Weather => &mut self.draw_state.modifiers.weather,
        }
    }

    pub(crate) fn handle_config_screen_input(&mut self, key: KeyEvent) -> Result<bool> {
        // Modifier editor is open: route all keys to it
        if let Some(editor) = self.draw_state.modifier_editor.as_mut() {
            match key.code {
                KeyCode::Esc => {
                    self.draw_state.modifier_editor = None;
                }
                KeyCode::Tab | KeyCode::BackTab => {
                    editor.field = match editor.field {
                        ModifierField::Name => ModifierField::Multiplier,
                        ModifierField::Multiplier => ModifierField::Name,
                    };
                }
                KeyCode::Enter => {
                    // Save the modifier; the editor is open (this handler
                    // only runs when it is), but handle its absence
                    // explicitly instead of panicking
                    let Some(editor) = self.draw_state.modifier_editor.take() else {
                        return Ok(false);
                    };
                    let name = editor.name.trim().to_string();
                    let multiplier: f32 = editor.multiplier.trim().parse().unwrap_or(1.0);
                    if !name.is_empty() {
                        let list = self.config_section_modifiers_mut(editor.section);
                        match editor.index {
                            Some(i) if i < list.len() => {
                                list[i].name = name;
                                list[i].multiplier = multiplier;
                            }
                            _ => {
                                list.push(crate::config::modifiers::ModifierConfig {
                                    name,
                                    multiplier,
                                });
                            }
                        }
                        // Keep the current selections valid
                        self.reconcile_selections();
                        self.save_config_or_report();
                        self.update_quantities()?;
                    }
                }
                KeyCode::Backspace => match editor.field {
                    ModifierField::Name => {
                        editor.name.pop();
                    }
                    ModifierField::Multiplier => {
                        editor.multiplier.pop();
                    }
                },
                KeyCode::Char(c) => match editor.field {
                    ModifierField::Name => {
                        editor.name.push(c);
                    }
                    ModifierField::Multiplier => {
                        editor.multiplier.push(c);
                    }
                },
                _ => {}
            }
            return Ok(false);
        }

        match key.code {
            KeyCode::Esc => {
                // Leave the config screen, back to where it was opened from
                self.draw_state.screen = self.draw_state.previous_screen;
            }
            KeyCode::Tab => {
                self.draw_state.config_section = self.draw_state.config_section.next();
                self.draw_state.config_cursor = 0;
                self.draw_state.config_scroll = 0;
            }
            KeyCode::BackTab => {
                self.draw_state.config_section = self.draw_state.config_section.prev();
                self.draw_state.config_cursor = 0;
                self.draw_state.config_scroll = 0;
            }
            KeyCode::Up => {
                if self.draw_state.config_cursor > 0 {
                    self.draw_state.config_cursor -= 1;
                }
            }
            KeyCode::Down => {
                let count = self
                    .config_section_modifiers(self.draw_state.config_section)
                    .len();
                if self.draw_state.config_cursor + 1 < count {
                    self.draw_state.config_cursor += 1;
                }
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                // Add a new modifier to the focused section
                self.draw_state.modifier_editor = Some(ModifierEditor {
                    index: None,
                    section: self.draw_state.config_section,
                    name: String::new(),
                    multiplier: String::from("1.0"),
                    field: ModifierField::Name,
                });
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                // Edit the modifier under the cursor
                let section = self.draw_state.config_section;
                let cursor = self.draw_state.config_cursor;
                if let Some(m) = self.config_section_modifiers(section).get(cursor) {
                    self.draw_state.modifier_editor = Some(ModifierEditor {
                        index: Some(cursor),
                        section,
                        name: m.name.clone(),
                        multiplier: m.multiplier.to_string(),
                        field: ModifierField::Name,
                    });
                }
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                // Delete the modifier under the cursor
                let section = self.draw_state.config_section;
                let cursor = self.draw_state.config_cursor;
                let list = self.config_section_modifiers_mut(section);
                if cursor < list.len() {
                    list.remove(cursor);
                    let count = list.len();
                    if self.draw_state.config_cursor >= count {
                        self.draw_state.config_cursor = count.saturating_sub(1);
                    }
                    self.reconcile_selections();
                    self.save_config_or_report();
                    self.update_quantities()?;
                }
            }
            _ => {}
        }
        Ok(false)
    }

    /// Keep the current travel style / weather / activity selections valid
    /// after modifiers change
    pub(crate) fn reconcile_selections(&mut self) {
        if self
            .draw_state
            .modifiers
            .travel_style_multiplier(&self.draw_state.travel_style)
            .is_none()
        {
            self.draw_state.travel_style = self
                .draw_state
                .modifiers
                .travel_styles
                .first()
                .map(|m| m.name.clone())
                .unwrap_or_default();
        }
        if self
            .draw_state
            .modifiers
            .weather_multiplier(&self.draw_state.weather)
            .is_none()
        {
            self.draw_state.weather = self
                .draw_state
                .modifiers
                .weather
                .first()
                .map(|m| m.name.clone())
                .unwrap_or_default();
        }
    }
}
