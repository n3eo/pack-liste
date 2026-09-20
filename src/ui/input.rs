use anyhow::{Context, Result};
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyEvent, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::fs;
use std::io::{self, stdout};
use std::path::PathBuf;

use crate::config::trip_file::{TripFile, TripItem};
use crate::models::item::QuantityBuffer;
use crate::models::weekdays::Weekdays;
use crate::quantity::calculation::QuantityCalculation;
use crate::ui::config_io::{
    AppConfig, SavedItem, create_default_config, load_config, resolve_trip_path, select_all_items,
};
use crate::ui::handlers;
use crate::ui::state::{
    AppDrawState, AppState, ConfigSection, Focus, FocusSide, InputMode, RecalculateScope, Screen,
    StatusKind,
};

/// The application: owns the terminal, the draw state, the quantity
/// calculation, and the config path; runs the draw/input loop.
pub struct App {
    pub(crate) state: AppState,
    pub(crate) terminal: Terminal<CrosstermBackend<io::Stdout>>,
    pub(crate) draw_state: AppDrawState,
    pub(crate) quantity_calculation: QuantityCalculation,
    pub(crate) config_path: PathBuf,
}

impl App {
    /// Initialize the terminal, load (or create) the config, load the
    /// trip file when a path is given, and calculate the quantities.
    pub fn new(trip_file_arg: Option<String>) -> Result<Self> {
        // Initialize terminal
        enable_raw_mode().context("Failed to enable raw mode")?;
        stdout()
            .execute(EnterAlternateScreen)
            .context("Failed to enter alternate screen")?;

        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend).context("Failed to create terminal")?;

        // Determine config path
        let config_path = dirs::config_dir()
            .context("Could not get config directory")?
            .join("pack-liste");

        let config_file = config_path.join("config.yaml");

        // Load or create default config. A missing file means first
        // run (created fresh, EC-001 of spec 013); an existing file
        // that fails to load exits with the error — never silently
        // substituted (EC-003).
        let (mut packing_list, trip_params) = if config_file.exists() {
            load_config(&config_file)?
        } else {
            create_default_config(&config_file)?
        };

        // A trip file passed on the command line is loaded directly; the
        // remembered path only prompts the user to resume on the welcome screen
        let cli_trip = trip_file_arg.filter(|s| !s.trim().is_empty());
        let remembered_trip = trip_params
            .last_trip_file
            .clone()
            .filter(|s| !s.trim().is_empty());

        let trip_file_path = cli_trip
            .as_deref()
            .map(resolve_trip_path)
            .unwrap_or_default();
        let mut resume_prompt = false;
        let mut remembered_trip_file: Option<PathBuf> = None;

        // Trip parameters, possibly overridden by a loaded trip file
        let mut weekdays = trip_params.weekdays;
        let mut travel_style = trip_params.travel_style;
        let mut weather = trip_params.weather;

        // Load and reconcile the trip file if a path is set
        let mut resumed = false;
        if !trip_file_path.as_os_str().is_empty() {
            if let Some(mut trip) = TripFile::load(&trip_file_path)? {
                resumed = true;
                // Reconcile selected items against the current input config
                let config_items: Vec<(String, Vec<String>)> = packing_list
                    .items
                    .iter()
                    .map(|item| (item.name.clone(), item.categories.clone()))
                    .collect();
                trip.reconcile(&config_items);

                // Restore trip parameters from the trip file
                weekdays = Weekdays::new(trip.weekdays).unwrap_or(weekdays);
                travel_style = trip.travel_style;
                weather = trip.weather;

                // Reset the default selection, then restore from the
                // file — including the saved quantities (spec 012):
                // the file is authoritative for its own trip
                for item in &mut packing_list.items {
                    item.selected = false;
                    item.packed = false;
                    if let Some(t) = trip
                        .items
                        .iter()
                        .find(|t| t.name.eq_ignore_ascii_case(&item.name))
                    {
                        item.selected = true;
                        item.packed = t.packed;
                        item.quantity = t.quantity;
                    }
                }
            } else {
                // A new trip file starts with every item selected
                select_all_items(&mut packing_list);
            }
        } else {
            // Offer to resume the remembered trip file on the welcome screen
            if let Some(remembered) = remembered_trip {
                let path = resolve_trip_path(&remembered);
                if path.exists() {
                    remembered_trip_file = Some(path);
                    resume_prompt = true;
                }
            }
            // A fresh list starts with every item selected
            select_all_items(&mut packing_list);
        }

        let draw_state = AppDrawState {
            screen: Screen::Welcome,
            packing_list,
            weekdays,
            travel_style,
            weather,
            modifiers: trip_params.modifiers,
            focus: Focus::TripFile,
            collapsed_categories: Vec::new(),
            selected_index: 0,
            focus_side: FocusSide::Left,
            fullscreen: false,
            input_mode: InputMode::Normal,
            left_scroll: 0,
            right_scroll: 0,
            trip_file_path,
            trip_file_input: String::new(),
            editing_trip_file: false,
            resume_prompt,
            remembered_trip_file,
            config_section: ConfigSection::TravelStyles,
            config_cursor: 0,
            config_scroll: 0,
            modifier_editor: None,
            previous_screen: Screen::Welcome,
            config_editor_text: Vec::new(),
            config_editor_cursor: (0, 0),
            config_editor_scroll: 0,
            config_editor_error: None,
            status: None,
        };

        let mut app = Self {
            state: AppState::Running,
            terminal,
            draw_state,
            quantity_calculation: QuantityCalculation::new(),
            config_path: config_file,
        };

        // Initialize quantities: a fresh list calculates once; a resumed
        // trip keeps the quantities restored from its file (spec 012 —
        // the file is authoritative for its own trip)
        if !resumed {
            app.update_quantities()?;
        }

        Ok(app)
    }

    /// Serialize the current state to the config file.
    pub(crate) fn save_config(&self) -> Result<()> {
        let config = AppConfig {
            categories: self.get_all_categories(),
            items: self
                .draw_state
                .packing_list
                .items
                .iter()
                .map(|item| {
                    let auto = item.auto_quantity;
                    SavedItem {
                        name: item.name.clone(),
                        categories: item.categories.clone(),
                        quantity: if auto { None } else { Some(item.quantity) },
                        auto_quantity: auto,
                        // Per-item modifiers (spec 010): the rate is
                        // written when it differs from the default; the
                        // buffer writes exactly one kind (SC-007)
                        rate: (item.rate != 1.0).then_some(item.rate),
                        reserve: match item.buffer {
                            QuantityBuffer::Reserve(n) => Some(n),
                            _ => None,
                        },
                        buffer_multiplier: match item.buffer {
                            QuantityBuffer::Multiplier(m) => Some(m),
                            _ => None,
                        },
                    }
                })
                .collect(),
            weekdays: self.draw_state.weekdays.get(),
            travel_style: self.draw_state.travel_style.clone(),
            weather: self.draw_state.weather.clone(),
            modifiers: self.draw_state.modifiers.clone(),
            last_trip_file: if self.draw_state.trip_file_path.as_os_str().is_empty() {
                // No new path chosen: keep remembering the previous trip file
                self.draw_state
                    .remembered_trip_file
                    .as_ref()
                    .map(|p| p.display().to_string())
            } else {
                Some(self.draw_state.trip_file_path.display().to_string())
            },
        };

        let yaml = serde_yaml::to_string(&config).context("Failed to serialize config")?;

        // Ensure config directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        fs::write(&self.config_path, yaml).context("Failed to write config file")?;

        Ok(())
    }

    /// Persist the compiled packing list to the trip file, in place.
    /// Quantities are not persisted; they are recalculated on load.
    fn save_trip_file(&self) -> Result<()> {
        if self.draw_state.trip_file_path.as_os_str().is_empty() {
            return Ok(());
        }

        let trip = TripFile {
            weekdays: self.draw_state.weekdays.get(),
            travel_style: self.draw_state.travel_style.clone(),
            weather: self.draw_state.weather.clone(),
            items: self
                .draw_state
                .packing_list
                .items
                .iter()
                .filter(|item| item.selected)
                .map(|item| TripItem {
                    name: item.name.clone(),
                    category: item.primary_category().to_string(),
                    packed: item.packed,
                    // The quantity snapshot (spec 012): whatever the
                    // user sees — calculated, fixed, or manually set
                    quantity: item.quantity,
                })
                .collect(),
        };

        trip.save(&self.draw_state.trip_file_path)
    }

    /// Set the trip file path from user input and load that trip file if it
    /// exists, restoring its trip parameters and packed state.
    pub(crate) fn apply_trip_file_path(&mut self, input: &str) -> Result<()> {
        let path = resolve_trip_path(input);
        self.draw_state.trip_file_path = path.clone();
        self.draw_state.editing_trip_file = false;

        // Load the trip file if it exists; a missing file means a new
        // trip: select everything and calculate once (FR-008's fresh
        // list). An existing trip restores its saved quantities below
        // without recalculating (spec 012/018).
        let Some(mut trip) = TripFile::load(&path)? else {
            select_all_items(&mut self.draw_state.packing_list);
            return self.update_quantities();
        };

        // Reconcile selected items against the current input config
        let config_items: Vec<(String, Vec<String>)> = self
            .draw_state
            .packing_list
            .items
            .iter()
            .map(|item| (item.name.clone(), item.categories.clone()))
            .collect();
        trip.reconcile(&config_items);

        // Restore trip parameters from the trip file
        self.draw_state.weekdays = Weekdays::new(trip.weekdays).unwrap_or(self.draw_state.weekdays);
        self.draw_state.travel_style = trip.travel_style;
        self.draw_state.weather = trip.weather;

        // Reset the previous trip's selection, then restore from the
        // file — including the saved quantities (spec 012); no
        // recalculation on resume: the file is authoritative
        for item in &mut self.draw_state.packing_list.items {
            item.selected = false;
            item.packed = false;
            if let Some(t) = trip
                .items
                .iter()
                .find(|t| t.name.eq_ignore_ascii_case(&item.name))
            {
                item.selected = true;
                item.packed = t.packed;
                item.quantity = t.quantity;
            }
        }

        Ok(())
    }

    /// All distinct item categories, sorted.
    pub(crate) fn get_all_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self
            .draw_state
            .packing_list
            .items
            .iter()
            .flat_map(|item| item.categories.iter().cloned())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }

    // Cycle through the configured travel styles (direction: 1 = forward, -1 = backward)
    pub(crate) fn cycle_travel_style(&mut self, direction: i32) {
        let styles = &self.draw_state.modifiers.travel_styles;
        let next = cycle_value(
            &styles.iter().map(|m| m.name.as_str()).collect::<Vec<_>>(),
            &self.draw_state.travel_style,
            direction,
        );
        if let Some(name) = next {
            self.draw_state.travel_style = name.to_string();
        }
    }

    // Cycle through the configured weather options (direction: 1 = forward, -1 = backward)
    pub(crate) fn cycle_weather(&mut self, direction: i32) {
        let weather = &self.draw_state.modifiers.weather;
        let next = cycle_value(
            &weather.iter().map(|m| m.name.as_str()).collect::<Vec<_>>(),
            &self.draw_state.weather,
            direction,
        );
        if let Some(name) = next {
            self.draw_state.weather = name.to_string();
        }
    }

    /// Recalculate every automatic item's quantity from the current
    /// trip parameters.
    pub(crate) fn update_quantities(&mut self) -> Result<()> {
        self.draw_state
            .recalculate_quantities(&self.quantity_calculation)
    }

    /// Save the config; on failure show the error in the status line
    /// instead of discarding it (the session continues).
    pub(crate) fn save_config_or_report(&mut self) {
        if let Err(e) = self.save_config() {
            self.draw_state
                .set_status(StatusKind::Error, format!("Save failed: {e}"));
        } else {
            self.draw_state.clear_status();
        }
    }

    /// Run an explicit recalculate action in the given scope (spec
    /// 018): resolve the scope against the cursor when needed, then
    /// recalculate; failures surface in the status line.
    pub(crate) fn recalculate_current_scope(&mut self, scope: RecalculateScope) -> Result<()> {
        // On a category header, the item scope means the header's
        // category (EC-001: R and Ctrl+R are equivalent there)
        let scope = match scope {
            RecalculateScope::Item(idx) => match self.draw_state.packing_list.items.get(idx) {
                Some(item) => {
                    if self.draw_state.cursor_item_index().is_some() {
                        RecalculateScope::Item(idx)
                    } else {
                        RecalculateScope::Category(item.primary_category().to_string())
                    }
                }
                None => return Ok(()),
            },
            other => other,
        };
        self.draw_state
            .recalculate_quantities_scoped(&self.quantity_calculation, scope)
    }

    /// Adjust the trip length by `direction` days, clamped to the valid
    /// range, and recalculate quantities.
    pub(crate) fn adjust_weekdays(&mut self, direction: i32) -> Result<()> {
        let target = self.draw_state.weekdays.get() as i32 + direction;
        if target >= Weekdays::MIN as i32
            && target <= Weekdays::MAX as i32
            && let Ok(new_weekdays) = Weekdays::new(target as u32)
        {
            // Parameters change; quantities recalculate only on the
            // explicit recalculate keys (spec 018, amended)
            self.draw_state.weekdays = new_weekdays;
        }
        Ok(())
    }

    /// Select every item of a fresh or empty packing list.
    pub(crate) fn select_all_items(&mut self) {
        select_all_items(&mut self.draw_state.packing_list);
    }

    /// Switch the focused panel and mirror the right cursor's category
    /// on the left panel.
    pub(crate) fn switch_focus_side(&mut self) {
        self.draw_state.focus_side = match self.draw_state.focus_side {
            FocusSide::Left => FocusSide::Right,
            FocusSide::Right => FocusSide::Left,
        };
        // Reset selected index for the new side
        self.draw_state.selected_index = 0;
        // Mirror the right cursor's category on the left panel
        if self.draw_state.focus_side == FocusSide::Right {
            self.sync_left_view_to_right_cursor();
        }
    }

    /// The main loop: draw, handle input, repeat until exit; saves the
    /// config and trip file on every exit path.
    pub fn run(&mut self) -> Result<()> {
        while let AppState::Running = self.state {
            self.draw()?;
            if self.handle_input()? {
                break;
            }
        }

        // Save config and trip file on every exit path (q, Esc, Ctrl+C)
        self.save_config()?;
        self.save_trip_file()?;

        Ok(())
    }

    /// The key dispatcher: global keys first, then routing to the
    /// handler of the current screen and input mode.
    fn handle_input(&mut self) -> Result<bool> {
        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            return self.dispatch_key(key);
        }
        Ok(false)
    }

    /// Global keys, then screen/mode routing.
    fn dispatch_key(&mut self, key: KeyEvent) -> Result<bool> {
        use crossterm::event::KeyCode as K;
        use crossterm::event::KeyModifiers as M;

        // Terminate on Ctrl+C from any screen or input mode
        if key.modifiers.contains(M::CONTROL) && key.code == K::Char('c') {
            self.state = AppState::Exit;
            return Ok(true);
        }

        // Ctrl+E opens the raw config file editor from any screen
        if key.modifiers.contains(M::CONTROL) && key.code == K::Char('e') {
            self.open_config_editor()?;
            return Ok(false);
        }

        // q quits from any screen (the config screen and raw editor
        // handle their own keys below and never see it)
        if key.code == K::Char('q')
            && !matches!(
                self.draw_state.screen,
                Screen::Config | Screen::ConfigEditor
            )
        {
            self.state = AppState::Exit;
            return Ok(true);
        }

        // ? opens help from Main and Welcome in normal mode
        if key.code == K::Char('?')
            && matches!(self.draw_state.screen, Screen::Main | Screen::Welcome)
            && matches!(self.draw_state.input_mode, InputMode::Normal)
        {
            self.draw_state.previous_screen = self.draw_state.screen;
            self.draw_state.screen = Screen::Help;
            return Ok(false);
        }

        // O opens the config screen from any screen except itself
        if matches!(key.code, K::Char('o') | K::Char('O'))
            && !matches!(
                self.draw_state.screen,
                Screen::Config | Screen::ConfigEditor
            )
        {
            self.draw_state.previous_screen = self.draw_state.screen;
            self.draw_state.screen = Screen::Config;
            return Ok(false);
        }

        // The recalculate chords (spec 018): Ctrl+R = the current
        // item's category, Ctrl+Shift+R = everything; matched before
        // the plain R arm in the main-screen handler. Main screen,
        // normal mode only (EC-006).
        if self.draw_state.screen == Screen::Main
            && matches!(self.draw_state.input_mode, InputMode::Normal)
            && key.modifiers.contains(M::CONTROL)
            && matches!(key.code, K::Char('r') | K::Char('R'))
        {
            let scope = if key.modifiers.contains(M::SHIFT) {
                RecalculateScope::All
            } else {
                // The cursor's item's category; on a header, the
                // header's own category (EC-001)
                match self.current_item_index() {
                    Some(idx) => RecalculateScope::Category(
                        self.draw_state.packing_list.items[idx]
                            .primary_category()
                            .to_string(),
                    ),
                    None => match self.current_category() {
                        Some(c) => RecalculateScope::Category(c),
                        None => return Ok(false),
                    },
                }
            };
            self.recalculate_current_scope(scope)?;
            return Ok(false);
        }

        // Esc: leave the current screen/mode or quit
        if key.code == K::Esc {
            return self.handle_escape();
        }

        // Screen routing
        match self.draw_state.screen {
            Screen::Welcome => handlers::welcome::handle_welcome_input(self, key),
            Screen::Main => match self.draw_state.input_mode {
                InputMode::Normal => handlers::welcome::handle_main_input(self, key),
                InputMode::AddingItem { .. }
                | InputMode::EditingItem { .. }
                | InputMode::ConfirmDelete { .. }
                | InputMode::ConfirmEdit { .. } => {
                    handlers::welcome::handle_dialog_input(self, key)
                }
            },
            Screen::Help => {
                if key.code == K::Enter {
                    self.draw_state.screen = self.draw_state.previous_screen;
                }
                Ok(false)
            }
            Screen::Config => self.handle_config_screen_input(key),
            Screen::ConfigEditor => self.handle_config_editor_input(key),
        }
    }

    /// Esc semantics per screen and mode.
    fn handle_escape(&mut self) -> Result<bool> {
        // A status message does not persist across a transition (V3)
        self.draw_state.clear_status();
        match self.draw_state.screen {
            Screen::Welcome => {
                self.state = AppState::Exit;
                Ok(true)
            }
            Screen::Help | Screen::ConfigEditor => {
                self.draw_state.screen = self.draw_state.previous_screen;
                Ok(false)
            }
            Screen::Config => {
                // Cancel the modifier editor, else leave the config screen
                if self.draw_state.modifier_editor.is_some() {
                    self.draw_state.modifier_editor = None;
                } else {
                    self.draw_state.screen = self.draw_state.previous_screen;
                }
                Ok(false)
            }
            Screen::Main => {
                match self.draw_state.input_mode {
                    InputMode::Normal => {
                        self.state = AppState::Exit;
                        Ok(true)
                    }
                    InputMode::AddingItem { .. }
                    | InputMode::EditingItem { .. }
                    | InputMode::ConfirmDelete { .. }
                    | InputMode::ConfirmEdit { .. } => {
                        // Cancel adding/editing/confirming
                        self.draw_state.input_mode = InputMode::Normal;
                        Ok(false)
                    }
                }
            }
        }
    }

    /// Move the cursor up one row and keep the left panel in sync
    /// when navigating the final list.
    pub(crate) fn handle_up(&mut self) -> Result<()> {
        self.draw_state.move_cursor_up();
        // Keep the left panel in sync when navigating the final list
        if self.draw_state.screen == Screen::Main
            && matches!(self.draw_state.input_mode, InputMode::Normal)
            && self.draw_state.focus_side == FocusSide::Right
        {
            self.sync_left_view_to_right_cursor();
        }
        Ok(())
    }

    /// Move the cursor down one row and keep the left panel in sync
    /// when navigating the final list.
    pub(crate) fn handle_down(&mut self) -> Result<()> {
        self.draw_state.move_cursor_down();
        // Keep the left panel in sync when navigating the final list
        if self.draw_state.screen == Screen::Main
            && matches!(self.draw_state.input_mode, InputMode::Normal)
            && self.draw_state.focus_side == FocusSide::Right
        {
            self.sync_left_view_to_right_cursor();
        }
        Ok(())
    }

    // The item currently selected on the focused side, if the cursor is on an item row
    pub(crate) fn current_item_index(&self) -> Option<usize> {
        self.draw_state.cursor_item_index()
    }

    // The category the cursor is on (header row or one of its items)
    pub(crate) fn current_category(&self) -> Option<String> {
        self.draw_state.cursor_category()
    }

    /// Toggle collapse of the category under the cursor.
    pub(crate) fn toggle_category_collapse(&mut self) {
        self.draw_state.toggle_category_collapse();
    }

    /// Toggle selection of the item under the cursor.
    pub(crate) fn toggle_item_selection(&mut self) {
        self.draw_state.toggle_item_selection();
    }

    /// Toggle selection of the cursor's whole category.
    pub(crate) fn toggle_category_selection(&mut self) {
        self.draw_state.toggle_category_selection();
    }

    /// Toggle the packed state of the item under the cursor (right
    /// panel only).
    pub(crate) fn toggle_item_packed_right_side(&mut self) {
        self.draw_state.toggle_item_packed();
    }

    /// After navigating on the right panel: collapse all categories on the
    /// left and expand the one the cursor's item belongs to, so the left
    /// panel mirrors the right panel's position.
    pub(crate) fn sync_left_view_to_right_cursor(&mut self) {
        let Some(item_index) = self.current_item_index() else {
            return;
        };
        let category = self.draw_state.packing_list.items[item_index]
            .primary_category()
            .to_string();

        // Collapse everything, then expand only the cursor's category
        let all_categories: Vec<String> = self
            .draw_state
            .packing_list
            .items
            .iter()
            .flat_map(|item| item.categories.iter().cloned())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        self.draw_state.collapsed_categories = all_categories;
        self.draw_state
            .collapsed_categories
            .retain(|c| c != &category);
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = stdout().execute(LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

/// The next value when cycling through `values` from `current` by
/// `direction` (+1/-1), wrapping around; None when the list is empty.
/// An unknown current value restarts at the first entry.
fn cycle_value<'a>(values: &[&'a str], current: &str, direction: i32) -> Option<&'a str> {
    if values.is_empty() {
        return None;
    }
    let pos = values.iter().position(|v| *v == current);
    let next = match pos {
        Some(i) => (i as i32 + direction).rem_euclid(values.len() as i32) as usize,
        None => 0,
    };
    values.get(next).copied()
}
