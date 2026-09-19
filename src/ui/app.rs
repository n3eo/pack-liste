use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Terminal,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, stdout};
use std::path::PathBuf;

use crate::config::modifiers::ModifiersConfig;
use crate::config::trip_file::{TripFile, TripItem};
use crate::models::item::Item;
use crate::models::packing_list::PackingList;
use crate::models::weekdays::Weekdays;
use crate::quantity::calculation::QuantityCalculation;
use crate::quantity::model::{ItemQuantitySpec, QuantityCalculationInput};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    Welcome,
    Main,
    Help,
    Config,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    Weekdays,
    TravelStyle,
    Weather,
    Activities,
}

impl Focus {
    pub fn next(&self) -> Self {
        match self {
            Focus::Weekdays => Focus::TravelStyle,
            Focus::TravelStyle => Focus::Weather,
            Focus::Weather => Focus::Activities,
            Focus::Activities => Focus::Weekdays,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Focus::Weekdays => Focus::Activities,
            Focus::TravelStyle => Focus::Weekdays,
            Focus::Weather => Focus::TravelStyle,
            Focus::Activities => Focus::Weather,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputField {
    Name,
    Category,
    Quantity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    AddingItem { field: InputField, name: String, category: String, quantity: String, category_list: Vec<String>, category_index: usize },
    EditingItem { index: usize, field: InputField, name: String, category: String, quantity: String, category_list: Vec<String>, category_index: usize },
    ConfirmDelete { index: usize },
}

pub enum AppState {
    Running,
    Exit,
}

// Config structure for saving to file
#[derive(Debug, Serialize, Deserialize)]
struct AppConfig {
    categories: Vec<String>,
    items: Vec<SavedItem>,
    #[serde(default)]
    weekdays: u32,
    #[serde(default = "default_travel_style_name")]
    travel_style: String,
    #[serde(default = "default_weather_name")]
    weather: String,
    #[serde(default)]
    selected_activities: Vec<String>,
    #[serde(default)]
    modifiers: ModifiersConfig,
    /// Name of the trip file to continue by default
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_trip_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SavedItem {
    name: String,
    category: String,
    /// Only present for fixed-quantity items (auto_quantity: false);
    /// auto items are recalculated on every load
    #[serde(default, skip_serializing_if = "Option::is_none")]
    quantity: Option<u32>,
    /// When false, the quantity is fixed and never recalculated
    #[serde(default = "default_true")]
    auto_quantity: bool,
}

fn default_true() -> bool {
    true
}

fn default_travel_style_name() -> String {
    "standard".to_string()
}

fn default_weather_name() -> String {
    "mixed".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_config_without_trip_params_loads() {
        // Config files written before the modifiers feature only had
        // categories and items; they must keep loading with defaults
        let yaml = "categories:\n- Clothing\nitems:\n- name: T-Shirt\n  category: Clothing\n  quantity: 5\n";
        let config: AppConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.items.len(), 1);
        assert_eq!(config.items[0].name, "T-Shirt");
        // Old quantity is ignored for auto items; the item recalculates
        assert!(config.items[0].auto_quantity);
        assert_eq!(config.items[0].quantity, Some(5)); // kept on load, unused for auto
        assert_eq!(config.weekdays, 0); // serde default, clamped to Weekdays default on load
        assert_eq!(config.travel_style, "standard");
        assert_eq!(config.weather, "mixed");
        assert!(config.selected_activities.is_empty());
        assert!(!config.modifiers.travel_styles.is_empty());
        assert!(!config.modifiers.weather.is_empty());
        assert!(!config.modifiers.activities.is_empty());
    }

    #[test]
    fn full_config_round_trips() {
        let config = AppConfig {
            categories: vec!["Clothing".to_string()],
            items: vec![
                SavedItem {
                    name: "T-Shirt".to_string(),
                    category: "Clothing".to_string(),
                    quantity: None,
                    auto_quantity: true,
                },
                SavedItem {
                    name: "Toothbrush".to_string(),
                    category: "Toiletries".to_string(),
                    quantity: Some(1),
                    auto_quantity: false,
                },
            ],
            weekdays: 10,
            travel_style: "luxury".to_string(),
            weather: "snowy".to_string(),
            selected_activities: vec!["skiing".to_string()],
            modifiers: ModifiersConfig::default(),
            last_trip_file: Some("my-trip.yaml".to_string()),
        };
        let yaml = serde_yaml::to_string(&config).unwrap();
        let parsed: AppConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.weekdays, 10);
        assert_eq!(parsed.travel_style, "luxury");
        assert_eq!(parsed.weather, "snowy");
        assert_eq!(parsed.selected_activities, vec!["skiing".to_string()]);
        assert_eq!(parsed.last_trip_file.as_deref(), Some("my-trip.yaml"));
        // Auto items do not persist a quantity
        assert_eq!(parsed.items[0].quantity, None);
        assert!(parsed.items[0].auto_quantity);
        // Fixed items keep their quantity
        assert_eq!(parsed.items[1].quantity, Some(1));
        assert!(!parsed.items[1].auto_quantity);
        assert_eq!(
            parsed.modifiers.travel_style_multiplier("luxury"),
            Some(1.5)
        );
    }

    #[test]
    fn auto_quantities_are_not_persisted() {
        // The serialized config must not persist a quantity value for auto
        // items (only the auto_quantity flag may appear)
        let config = AppConfig {
            categories: vec![],
            items: vec![SavedItem {
                name: "T-Shirt".to_string(),
                category: "Clothing".to_string(),
                quantity: None,
                auto_quantity: true,
            }],
            weekdays: 7,
            travel_style: "standard".to_string(),
            weather: "mixed".to_string(),
            selected_activities: vec![],
            modifiers: ModifiersConfig::default(),
            last_trip_file: None,
        };
        let yaml = serde_yaml::to_string(&config).unwrap();
        // No "quantity: <n>" line for the auto item
        assert!(!yaml.lines().any(|l| l.trim_start().starts_with("quantity:")));
        assert!(yaml.contains("auto_quantity: true"));
    }

    #[test]
    fn trip_file_round_trips_and_reconciles() {
        use crate::config::trip_file::{TripFile, TripItem};

        let mut trip = TripFile {
            weekdays: 7,
            travel_style: "standard".to_string(),
            weather: "cold".to_string(),
            selected_activities: vec!["hiking".to_string()],
            items: vec![
                TripItem { name: "T-Shirt".to_string(), category: "Clothing".to_string(), packed: true },
                TripItem { name: "Old Item".to_string(), category: "Gone".to_string(), packed: false },
            ],
        };

        // Reconcile against a config where "Old Item" no longer exists
        let config_items = vec![
            ("T-Shirt".to_string(), "Clothing".to_string()),
            ("Socks".to_string(), "Clothing".to_string()),
        ];
        trip.reconcile(&config_items);

        // Dropped item that is gone from the config, kept the rest
        assert_eq!(trip.items.len(), 1);
        assert_eq!(trip.items[0].name, "T-Shirt");
        assert!(trip.items[0].packed);

        // Round trip through YAML
        let yaml = serde_yaml::to_string(&trip).unwrap();
        let parsed: TripFile = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.weekdays, 7);
        assert_eq!(parsed.items.len(), 1);
        assert!(parsed.items[0].packed);
    }
}

// State for drawing (cloneable)
#[derive(Clone)]
struct AppDrawState {
    screen: Screen,
    packing_list: PackingList,
    weekdays: Weekdays,
    travel_style: String,
    weather: String,
    selected_activities: Vec<String>,
    modifiers: ModifiersConfig,
    focus: Focus,
    collapsed_categories: Vec<String>,
    selected_index: usize,
    focus_side: FocusSide,
    input_mode: InputMode,
    left_scroll: usize,
    right_scroll: usize,
    activity_cursor: usize,
    /// File name of the packing list to persist (without directory)
    trip_file_name: String,
    /// Text input for the trip file name on the welcome screen
    trip_file_input: String,
    /// Whether the trip file name is being edited
    editing_trip_file: bool,
    /// Config screen state
    config_section: ConfigSection,
    config_cursor: usize,
    config_scroll: usize,
    /// Editor for adding/editing a modifier on the config screen
    modifier_editor: Option<ModifierEditor>,
}

/// Sections of the config screen
#[derive(Debug, Clone, Copy, PartialEq)]
enum ConfigSection {
    TravelStyles,
    Weather,
    Activities,
}

impl ConfigSection {
    fn title(&self) -> &'static str {
        match self {
            ConfigSection::TravelStyles => "Travel Styles",
            ConfigSection::Weather => "Weather",
            ConfigSection::Activities => "Activities",
        }
    }

    fn next(&self) -> Self {
        match self {
            ConfigSection::TravelStyles => ConfigSection::Weather,
            ConfigSection::Weather => ConfigSection::Activities,
            ConfigSection::Activities => ConfigSection::TravelStyles,
        }
    }

    fn prev(&self) -> Self {
        match self {
            ConfigSection::TravelStyles => ConfigSection::Activities,
            ConfigSection::Weather => ConfigSection::TravelStyles,
            ConfigSection::Activities => ConfigSection::Weather,
        }
    }
}

/// In-progress modifier being added or edited on the config screen
#[derive(Debug, Clone)]
struct ModifierEditor {
    /// Index into the section's list; None when adding a new modifier
    index: Option<usize>,
    section: ConfigSection,
    name: String,
    multiplier: String,
    field: ModifierField,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ModifierField {
    Name,
    Multiplier,
}

// A navigable row of a panel: a category header or an item
#[derive(Clone, Debug)]
enum PanelRow {
    Category(String),
    Item(usize), // index into packing_list.items
}

impl AppDrawState {
    fn left_rows(&self) -> Vec<PanelRow> {
        // Rows of the left panel in display order: each category header,
        // followed by its items unless the category is collapsed
        let mut categories_map: std::collections::BTreeMap<String, Vec<usize>> = std::collections::BTreeMap::new();
        for (idx, item) in self.packing_list.items.iter().enumerate() {
            categories_map.entry(item.category.clone()).or_default().push(idx);
        }

        let mut rows = Vec::new();
        for (category, indices) in &categories_map {
            rows.push(PanelRow::Category(category.clone()));
            if !self.collapsed_categories.contains(category) {
                for idx in indices {
                    rows.push(PanelRow::Item(*idx));
                }
            }
        }
        rows
    }

    fn right_item_order(&self) -> Vec<usize> {
        // Selected items in the order they are displayed on the right
        // panel (grouped by category)
        let mut categories_map: std::collections::BTreeMap<String, Vec<usize>> = std::collections::BTreeMap::new();
        for (idx, item) in self.packing_list.items.iter().enumerate() {
            if item.editable {
                categories_map.entry(item.category.clone()).or_default().push(idx);
            }
        }

        let mut order = Vec::new();
        for (_, indices) in &categories_map {
            order.extend(indices);
        }
        order
    }

    // Rows the cursor moves over on the focused side
    fn active_rows(&self) -> Vec<PanelRow> {
        match self.focus_side {
            FocusSide::Left => self.left_rows(),
            FocusSide::Right => self.right_item_order().into_iter().map(PanelRow::Item).collect(),
        }
    }
}

pub struct App {
    state: AppState,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    draw_state: AppDrawState,
    quantity_calculation: QuantityCalculation,
    config_path: PathBuf,
    /// Directory trip files are persisted to
    trips_dir: PathBuf,
}

impl App {
    pub fn new(trip_file_arg: Option<String>) -> Result<Self> {
        // Initialize terminal
        enable_raw_mode()
            .context("Failed to enable raw mode")?;
        stdout()
            .execute(EnterAlternateScreen)
            .context("Failed to enter alternate screen")?;

        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend)
            .context("Failed to create terminal")?;

        // Determine config path
        let config_path = dirs::config_dir()
            .context("Could not get config directory")?
            .join("pack-liste");

        let config_file = config_path.join("config.yaml");
        let trips_dir = config_path.join("trips");

        // Load or create default config
        let (mut packing_list, trip_params) = if config_file.exists() {
            load_config(&config_file)?
        } else {
            create_default_config(&config_file)?
        };

        // Resolve the trip file: CLI arg wins, else the name saved in the config
        let trip_file_name = trip_file_arg
            .or_else(|| trip_params.last_trip_file.clone())
            .unwrap_or_default();

        // Trip parameters, possibly overridden by a loaded trip file
        let mut weekdays = trip_params.weekdays;
        let mut travel_style = trip_params.travel_style;
        let mut weather = trip_params.weather;
        let mut selected_activities = trip_params.selected_activities;

        // Load and reconcile the trip file if it exists
        if !trip_file_name.is_empty() {
            let trip_path = trips_dir.join(&trip_file_name);
            if let Some(mut trip) = TripFile::load(&trip_path)? {
                // Reconcile selected items against the current input config
                let config_items: Vec<(String, String)> = packing_list.items.iter()
                    .map(|item| (item.name.clone(), item.category.clone()))
                    .collect();
                trip.reconcile(&config_items);

                // Restore trip parameters from the trip file
                weekdays = Weekdays::new(trip.weekdays).unwrap_or(weekdays);
                travel_style = trip.travel_style;
                weather = trip.weather;
                selected_activities = trip.selected_activities;

                // Restore selection and packed state
                for item in &mut packing_list.items {
                    if let Some(t) = trip.items.iter()
                        .find(|t| t.name.eq_ignore_ascii_case(&item.name))
                    {
                        item.editable = true;
                        item.packed = t.packed;
                    }
                }
            }
        }

        let draw_state = AppDrawState {
            screen: Screen::Welcome,
            packing_list,
            weekdays,
            travel_style,
            weather,
            selected_activities,
            modifiers: trip_params.modifiers,
            focus: Focus::Weekdays,
            collapsed_categories: Vec::new(),
            selected_index: 0,
            focus_side: FocusSide::Left,
            input_mode: InputMode::Normal,
            left_scroll: 0,
            right_scroll: 0,
            activity_cursor: 0,
            trip_file_name,
            trip_file_input: String::new(),
            editing_trip_file: false,
            config_section: ConfigSection::TravelStyles,
            config_cursor: 0,
            config_scroll: 0,
            modifier_editor: None,
        };

        let mut app = Self {
            state: AppState::Running,
            terminal,
            draw_state,
            quantity_calculation: QuantityCalculation::new(),
            config_path: config_file,
            trips_dir,
        };

        // Initialize quantities
        app.update_quantities()?;

        Ok(app)
    }

    fn save_config(&self) -> Result<()> {
        let config = AppConfig {
            categories: self.get_unique_categories(),
            items: self.draw_state.packing_list.items.iter()
                .map(|item| {
                    let auto = item.auto_quantity;
                    SavedItem {
                        name: item.name.clone(),
                        category: item.category.clone(),
                        quantity: if auto { None } else { Some(item.quantity) },
                        auto_quantity: auto,
                    }
                })
                .collect(),
            weekdays: self.draw_state.weekdays.get(),
            travel_style: self.draw_state.travel_style.clone(),
            weather: self.draw_state.weather.clone(),
            selected_activities: self.draw_state.selected_activities.clone(),
            modifiers: self.draw_state.modifiers.clone(),
            last_trip_file: if self.draw_state.trip_file_name.is_empty() {
                None
            } else {
                Some(self.draw_state.trip_file_name.clone())
            },
        };

        let yaml = serde_yaml::to_string(&config)
            .context("Failed to serialize config")?;

        // Ensure config directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        fs::write(&self.config_path, yaml)
            .context("Failed to write config file")?;

        Ok(())
    }

    /// Persist the compiled packing list to the trip file, in place.
    /// Quantities are not persisted; they are recalculated on load.
    fn save_trip_file(&self) -> Result<()> {
        if self.draw_state.trip_file_name.is_empty() {
            return Ok(());
        }

        let trip = TripFile {
            weekdays: self.draw_state.weekdays.get(),
            travel_style: self.draw_state.travel_style.clone(),
            weather: self.draw_state.weather.clone(),
            selected_activities: self.draw_state.selected_activities.clone(),
            items: self.draw_state.packing_list.items.iter()
                .filter(|item| item.editable)
                .map(|item| TripItem {
                    name: item.name.clone(),
                    category: item.category.clone(),
                    packed: item.packed,
                })
                .collect(),
        };

        trip.save(&self.trips_dir.join(&self.draw_state.trip_file_name))
    }

    fn get_unique_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self.draw_state.packing_list.items.iter()
            .map(|item| item.category.clone())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }

    fn get_all_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self.draw_state.packing_list.items.iter()
            .map(|item| item.category.clone())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }

    // Cycle through the configured travel styles (direction: 1 = forward, -1 = backward)
    fn cycle_travel_style(&mut self, direction: i32) {
        let styles = &self.draw_state.modifiers.travel_styles;
        if styles.is_empty() {
            return;
        }
        let current = styles.iter().position(|m| m.name == self.draw_state.travel_style);
        let next = match current {
            Some(i) => (i as i32 + direction).rem_euclid(styles.len() as i32) as usize,
            None => 0,
        };
        self.draw_state.travel_style = styles[next].name.clone();
    }

    // Cycle through the configured weather options (direction: 1 = forward, -1 = backward)
    fn cycle_weather(&mut self, direction: i32) {
        let weather = &self.draw_state.modifiers.weather;
        if weather.is_empty() {
            return;
        }
        let current = weather.iter().position(|m| m.name == self.draw_state.weather);
        let next = match current {
            Some(i) => (i as i32 + direction).rem_euclid(weather.len() as i32) as usize,
            None => 0,
        };
        self.draw_state.weather = weather[next].name.clone();
    }

    // Toggle the activity under the cursor on the welcome screen
    fn toggle_activity(&mut self) {
        let activities = &self.draw_state.modifiers.activities;
        let Some(activity) = activities.get(self.draw_state.activity_cursor) else {
            return;
        };
        let name = activity.name.clone();
        if self.draw_state.selected_activities.contains(&name) {
            self.draw_state.selected_activities.retain(|a| a != &name);
        } else {
            self.draw_state.selected_activities.push(name);
        }
        self.update_quantities().ok();
    }

    fn update_quantities(&mut self) -> Result<()> {
        // Resolve multipliers from the config; fall back to 1.0 for unknown names
        let style_multiplier = self.draw_state.modifiers
            .travel_style_multiplier(&self.draw_state.travel_style)
            .unwrap_or(1.0);
        let weather_multiplier = self.draw_state.modifiers
            .weather_multiplier(&self.draw_state.weather)
            .unwrap_or(1.0);
        let activity_multipliers: Vec<f32> = self.draw_state.selected_activities.iter()
            .filter_map(|name| self.draw_state.modifiers.activity_multiplier(name))
            .collect();

        let item_specs: Vec<ItemQuantitySpec> = self.draw_state.packing_list.items.iter()
            .map(|item| {
                if item.auto_quantity {
                    ItemQuantitySpec::Auto
                } else {
                    ItemQuantitySpec::Fixed(item.quantity)
                }
            })
            .collect();

        let input = QuantityCalculationInput {
            weekdays: self.draw_state.weekdays,
            item_specs,
            travel_style: self.draw_state.travel_style.clone(),
            travel_style_multiplier: style_multiplier,
            weather: self.draw_state.weather.clone(),
            weather_multiplier,
            activities: self.draw_state.selected_activities.clone(),
            activity_multipliers,
        };

        let output = self.quantity_calculation.calculate(&input)?;

        // Update item quantities
        for (i, quantity) in output.item_quantities.iter().enumerate() {
            if i < self.draw_state.packing_list.items.len() {
                self.draw_state.packing_list.items[i].quantity = *quantity;
            }
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            match self.state {
                AppState::Running => {
                    self.draw()?;
                    if self.handle_input()? {
                        break;
                    }
                }
                AppState::Exit => {
                    // Save config and trip file before exiting
                    self.save_config()?;
                    self.save_trip_file()?;
                    break;
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self) -> Result<()> {
        let mut state = self.draw_state.clone();

        self.terminal.draw(|f| {
            let size = f.area();

            match state.screen {
                Screen::Welcome => {
                    draw_welcome_screen(f, size, &state);
                }
                Screen::Main => {
                    match state.input_mode {
                        InputMode::Normal => {
                            draw_main_screen(f, size, &mut state);
                        }
                        InputMode::AddingItem { field, ref name, ref category, ref quantity, ref category_list, category_index } => {
                            draw_add_item_popup(f, size, field, name, category, quantity, category_list, category_index);
                        }
                        InputMode::EditingItem { field, ref name, ref category, ref quantity, ref category_list, category_index, .. } => {
                            draw_edit_item_popup(f, size, field, name, category, quantity, category_list, category_index);
                        }
                        InputMode::ConfirmDelete { index } => {
                            draw_confirm_delete_popup(f, size, index, &state.packing_list);
                        }
                    }
                }
                Screen::Help => {
                    draw_help_screen(f, size);
                }
                Screen::Config => {
                    draw_config_screen(f, size, &state);
                }
            }
        })?;

        // Persist the scroll offsets computed during rendering
        self.draw_state.left_scroll = state.left_scroll;
        self.draw_state.right_scroll = state.right_scroll;
        Ok(())
    }

    fn handle_input(&mut self) -> Result<bool> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Terminate on Ctrl+C from any screen or input mode
                    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                        self.state = AppState::Exit;
                        return Ok(true);
                    }

                    // Check for Ctrl+Space first
                    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char(' ') {
                        if self.draw_state.screen == Screen::Main && matches!(self.draw_state.input_mode, InputMode::Normal) {
                            if self.draw_state.focus_side == FocusSide::Left {
                                self.toggle_category_selection();
                            }
                        }
                        return Ok(false);
                    }

                    // Left/Right collapse/expand the group under the cursor
                    if self.draw_state.screen == Screen::Main
                        && matches!(self.draw_state.input_mode, InputMode::Normal)
                        && self.draw_state.focus_side == FocusSide::Left
                        && matches!(key.code, KeyCode::Left | KeyCode::Right)
                    {
                        let collapse = key.code == KeyCode::Left;
                        self.set_category_collapsed(collapse);
                        return Ok(false);
                    }
                    
                    // Route text input to the add/edit popups
                    if self.draw_state.screen == Screen::Main
                        && matches!(self.draw_state.input_mode, InputMode::AddingItem { .. } | InputMode::EditingItem { .. })
                        && matches!(key.code, KeyCode::Char(_) | KeyCode::Backspace)
                    {
                        self.handle_add_edit_input(key);
                        return Ok(false);
                    }

                    // The config screen handles all its own keys
                    if self.draw_state.screen == Screen::Config {
                        return self.handle_config_screen_input(key);
                    }

                    // Trip file name editing on the welcome screen
                    if self.draw_state.screen == Screen::Welcome && self.draw_state.editing_trip_file {
                        match key.code {
                            KeyCode::Enter => {
                                let name = self.draw_state.trip_file_input.trim().to_string();
                                if !name.is_empty() {
                                    self.draw_state.trip_file_name = name;
                                }
                                self.draw_state.editing_trip_file = false;
                            }
                            KeyCode::Esc => {
                                self.draw_state.editing_trip_file = false;
                            }
                            KeyCode::Backspace => {
                                self.draw_state.trip_file_input.pop();
                            }
                            KeyCode::Char(c) => {
                                self.draw_state.trip_file_input.push(c);
                            }
                            _ => {}
                        }
                        return Ok(false);
                    }

                    match key.code {
                        KeyCode::Esc => {
                            match self.draw_state.screen {
                                Screen::Welcome => {
                                    self.state = AppState::Exit;
                                    return Ok(true);
                                }
                                Screen::Help => {
                                    self.draw_state.screen = Screen::Main;
                                }
                                Screen::Config => {
                                    // Cancel the modifier editor, else leave the config screen
                                    if self.draw_state.modifier_editor.is_some() {
                                        self.draw_state.modifier_editor = None;
                                    } else {
                                        self.draw_state.screen = Screen::Welcome;
                                    }
                                }
                                Screen::Main => {
                                    match self.draw_state.input_mode {
                                        InputMode::Normal => {
                                            self.state = AppState::Exit;
                                            return Ok(true);
                                        }
                                        InputMode::AddingItem { .. } | InputMode::EditingItem { .. } | InputMode::ConfirmDelete { .. } => {
                                            // Cancel adding/editing/confirming
                                            self.draw_state.input_mode = InputMode::Normal;
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Char('q') => {
                            self.state = AppState::Exit;
                            return Ok(true);
                        }
                        KeyCode::Char('?') => {
                            if self.draw_state.screen == Screen::Main && matches!(self.draw_state.input_mode, InputMode::Normal) {
                                self.draw_state.screen = Screen::Help;
                            }
                        }
                        KeyCode::Enter => {
                            match self.draw_state.screen {
                                Screen::Welcome => {
                                    // Move to main screen after configuring
                                    self.draw_state.screen = Screen::Main;
                                    self.draw_state.focus_side = FocusSide::Left;
                                    self.draw_state.selected_index = 0;
                                }
                                Screen::Help => {
                                    self.draw_state.screen = Screen::Main;
                                }
                                Screen::Config => {}
                                Screen::Main => {
                                    match &self.draw_state.input_mode {
                                        InputMode::Normal => {
                                            if self.draw_state.focus_side == FocusSide::Left {
                                                // Toggle selection on left side
                                                self.toggle_item_selection();
                                            } else {
                                                // Toggle packed status on right side
                                                self.toggle_item_packed_right_side();
                                            }
                                        }
                                        InputMode::AddingItem { name, category, quantity, category_index, .. } => {
                                            // Get the selected category from the list
                                            let categories = self.get_all_categories();
                                            let selected_category = if *category_index < categories.len() {
                                                categories[*category_index].clone()
                                            } else if !categories.is_empty() {
                                                categories[0].clone()
                                            } else {
                                                category.clone()
                                            };
                                            // Save the new item
                                            self.save_new_item(name.clone(), selected_category, quantity.clone());
                                            self.draw_state.input_mode = InputMode::Normal;
                                        }
                                        InputMode::EditingItem { index, name, category, quantity, category_index, .. } => {
                                            // Get the selected category from the list
                                            let categories = self.get_all_categories();
                                            let selected_category = if *category_index < categories.len() {
                                                categories[*category_index].clone()
                                            } else if !categories.is_empty() {
                                                categories[0].clone()
                                            } else {
                                                category.clone()
                                            };
                                            // Save the edited item
                                            self.save_edited_item(*index, name.clone(), selected_category, quantity.clone());
                                            self.draw_state.input_mode = InputMode::Normal;
                                        }
                                        InputMode::ConfirmDelete { index } => {
                                            // Confirm deletion
                                            self.remove_item_confirmed(*index);
                                            self.draw_state.input_mode = InputMode::Normal;
                                            // Save config after deletion
                                            self.save_config()?;
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Tab => {
                            if self.draw_state.screen == Screen::Main {
                                match &mut self.draw_state.input_mode {
                                    InputMode::Normal => {
                                        // Switch between left and right panels
                                        self.draw_state.focus_side = match self.draw_state.focus_side {
                                            FocusSide::Left => FocusSide::Right,
                                            FocusSide::Right => FocusSide::Left,
                                        };
                                        // Reset selected index for the new side
                                        self.draw_state.selected_index = 0;
                                    }
                                    InputMode::AddingItem { field, .. } => {
                                        // Cycle through fields
                                        *field = match field {
                                            InputField::Name => InputField::Category,
                                            InputField::Category => InputField::Quantity,
                                            InputField::Quantity => InputField::Name,
                                        };
                                    }
                                    InputMode::EditingItem { field, .. } => {
                                        // Cycle through fields
                                        *field = match field {
                                            InputField::Name => InputField::Category,
                                            InputField::Category => InputField::Quantity,
                                            InputField::Quantity => InputField::Name,
                                        };
                                    }
                                    _ => {}
                                }
                            } else if self.draw_state.screen == Screen::Welcome {
                                // In welcome screen, Tab cycles through fields
                                self.draw_state.focus = self.draw_state.focus.next();
                            }
                        }
                        KeyCode::BackTab => {
                            if self.draw_state.screen == Screen::Main && matches!(self.draw_state.input_mode, InputMode::Normal) {
                                self.draw_state.focus_side = match self.draw_state.focus_side {
                                    FocusSide::Left => FocusSide::Right,
                                    FocusSide::Right => FocusSide::Left,
                                };
                                self.draw_state.selected_index = 0;
                            } else if self.draw_state.screen == Screen::Welcome {
                                self.draw_state.focus = self.draw_state.focus.prev();
                            }
                        }
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            if self.draw_state.screen == Screen::Main && matches!(self.draw_state.input_mode, InputMode::Normal) {
                                if self.draw_state.focus_side == FocusSide::Left {
                                    self.toggle_category_collapse();
                                }
                            }
                        }
                        KeyCode::Char(' ') => {
                            if self.draw_state.screen == Screen::Main && matches!(self.draw_state.input_mode, InputMode::Normal) {
                                if self.draw_state.focus_side == FocusSide::Left {
                                    // Toggle selection on left side
                                    self.toggle_item_selection();
                                } else {
                                    // Toggle packed status on right side
                                    self.toggle_item_packed_right_side();
                                }
                            } else if self.draw_state.screen == Screen::Welcome {
                                // Toggle the activity under the cursor
                                if self.draw_state.focus == Focus::Activities {
                                    self.toggle_activity();
                                }
                            } else if self.draw_state.screen == Screen::Main {
                                // Handle space in edit mode for category selection
                                if let InputMode::EditingItem { field, .. } = &mut self.draw_state.input_mode {
                                    if *field == InputField::Category {
                                        // Space toggles category selection in edit mode
                                        // This is handled by the field cycling
                                    }
                                }
                            }
                        }
                        KeyCode::Char('a') | KeyCode::Char('A') => {
                            if self.draw_state.screen == Screen::Main && matches!(self.draw_state.input_mode, InputMode::Normal) {
                                if self.draw_state.focus_side == FocusSide::Left {
                                    // Start adding new item with category dropdown
                                    let categories = self.get_all_categories();
                                    self.draw_state.input_mode = InputMode::AddingItem {
                                        field: InputField::Name,
                                        name: String::new(),
                                        category: String::new(),
                                        quantity: String::new(),
                                        category_list: categories,
                                        category_index: 0,
                                    };
                                }
                            }
                        }
                        KeyCode::Char('e') | KeyCode::Char('E') => {
                            if self.draw_state.screen == Screen::Main && matches!(self.draw_state.input_mode, InputMode::Normal) {
                                if self.draw_state.focus_side == FocusSide::Left {
                                    // Start editing selected item
                                    self.start_editing_item();
                                }
                            }
                        }
                        KeyCode::Char('d') | KeyCode::Char('D') => {
                            if self.draw_state.screen == Screen::Main && matches!(self.draw_state.input_mode, InputMode::Normal) {
                                if self.draw_state.focus_side == FocusSide::Left {
                                    // Ask for confirmation before deleting
                                    if let Some(item_index) = self.current_item_index() {
                                        self.draw_state.input_mode = InputMode::ConfirmDelete {
                                            index: item_index,
                                        };
                                    }
                                }
                            }
                        }
                        KeyCode::Up => {
                            self.handle_up()?;
                        }
                        KeyCode::Down => {
                            self.handle_down()?;
                        }
                        KeyCode::Char('f') | KeyCode::Char('F') => {
                            if self.draw_state.screen == Screen::Welcome {
                                // Edit the trip file name
                                self.draw_state.trip_file_input = self.draw_state.trip_file_name.clone();
                                self.draw_state.editing_trip_file = true;
                            }
                        }
                        KeyCode::Char('o') | KeyCode::Char('O') => {
                            if self.draw_state.screen == Screen::Welcome {
                                // Open the config screen
                                self.draw_state.screen = Screen::Config;
                            }
                        }
                        KeyCode::Char('+') => {
                            if self.draw_state.screen == Screen::Welcome && self.draw_state.focus == Focus::Weekdays {
                                if let Ok(new_weekdays) = Weekdays::new(self.draw_state.weekdays.get() + 1) {
                                    self.draw_state.weekdays = new_weekdays;
                                    self.update_quantities()?;
                                }
                            }
                        }
                        KeyCode::Char('-') => {
                            if self.draw_state.screen == Screen::Welcome && self.draw_state.focus == Focus::Weekdays {
                                if self.draw_state.weekdays.get() > Weekdays::MIN {
                                    if let Ok(new_weekdays) = Weekdays::new(self.draw_state.weekdays.get() - 1) {
                                        self.draw_state.weekdays = new_weekdays;
                                        self.update_quantities()?;
                                    }
                                }
                            }
                        }
                        // Handle input mode for adding/editing item
                        _ => {
                            if self.draw_state.screen == Screen::Main {
                                self.handle_add_edit_input(key);
                            }
                        }
                    }
                }
            }
        }
        Ok(false)
    }

    fn handle_up(&mut self) -> Result<()> {
        match self.draw_state.screen {
            Screen::Welcome => {
                match self.draw_state.focus {
                    Focus::Weekdays => {
                        if let Ok(new_weekdays) = Weekdays::new(self.draw_state.weekdays.get() + 1) {
                            self.draw_state.weekdays = new_weekdays;
                            self.update_quantities()?;
                        }
                    }
                    Focus::TravelStyle => {
                        self.cycle_travel_style(-1);
                        self.update_quantities()?;
                    }
                    Focus::Weather => {
                        self.cycle_weather(-1);
                        self.update_quantities()?;
                    }
                    Focus::Activities => {
                        if self.draw_state.activity_cursor > 0 {
                            self.draw_state.activity_cursor -= 1;
                        }
                    }
                }
            }
            Screen::Config => {}
            Screen::Main => {
                if matches!(self.draw_state.input_mode, InputMode::Normal) {
                    // Navigate up through the visible rows of the focused side
                    if self.draw_state.selected_index > 0 {
                        self.draw_state.selected_index -= 1;
                    }
                } else if let InputMode::AddingItem { field, category_index, .. } = &mut self.draw_state.input_mode {
                    // Only navigate category dropdown when category field is selected
                    if *field == InputField::Category && *category_index > 0 {
                        *category_index -= 1;
                    }
                } else if let InputMode::EditingItem { field, category_index, .. } = &mut self.draw_state.input_mode {
                    // Only navigate category dropdown when category field is selected
                    if *field == InputField::Category {
                        if *category_index > 0 {
                            *category_index -= 1;
                        }
                    }
                }
            }
            Screen::Help => {}
        }
        Ok(())
    }

    fn handle_down(&mut self) -> Result<()> {
        match self.draw_state.screen {
            Screen::Welcome => {
                match self.draw_state.focus {
                    Focus::Weekdays => {
                        if self.draw_state.weekdays.get() > Weekdays::MIN {
                            if let Ok(new_weekdays) = Weekdays::new(self.draw_state.weekdays.get() - 1) {
                                self.draw_state.weekdays = new_weekdays;
                                self.update_quantities()?;
                            }
                        }
                    }
                    Focus::TravelStyle => {
                        self.cycle_travel_style(1);
                        self.update_quantities()?;
                    }
                    Focus::Weather => {
                        self.cycle_weather(1);
                        self.update_quantities()?;
                    }
                    Focus::Activities => {
                        let count = self.draw_state.modifiers.activities.len();
                        if self.draw_state.activity_cursor + 1 < count {
                            self.draw_state.activity_cursor += 1;
                        }
                    }
                }
            }
            Screen::Config => {}
            Screen::Main => {
                if matches!(self.draw_state.input_mode, InputMode::Normal) {
                    // Navigate down through the visible rows of the focused side
                    let rows = self.draw_state.active_rows();
                    if self.draw_state.selected_index < rows.len().saturating_sub(1) {
                        self.draw_state.selected_index += 1;
                    }
                } else if let InputMode::AddingItem { field, category_index, category_list, .. } = &mut self.draw_state.input_mode {
                    // Only navigate category dropdown when category field is selected
                    if *field == InputField::Category && *category_index < category_list.len().saturating_sub(1) {
                        *category_index += 1;
                    }
                } else if let InputMode::EditingItem { field, category_index, category_list, .. } = &mut self.draw_state.input_mode {
                    // Only navigate category dropdown when category field is selected
                    if *field == InputField::Category {
                        if *category_index < category_list.len().saturating_sub(1) {
                            *category_index += 1;
                        }
                    }
                }
            }
            Screen::Help => {}
        }
        Ok(())
    }

    // The modifiers of the focused config section
    fn config_section_modifiers(&self, section: ConfigSection) -> &Vec<crate::config::modifiers::ModifierConfig> {
        match section {
            ConfigSection::TravelStyles => &self.draw_state.modifiers.travel_styles,
            ConfigSection::Weather => &self.draw_state.modifiers.weather,
            ConfigSection::Activities => &self.draw_state.modifiers.activities,
        }
    }

    fn config_section_modifiers_mut(&mut self, section: ConfigSection) -> &mut Vec<crate::config::modifiers::ModifierConfig> {
        match section {
            ConfigSection::TravelStyles => &mut self.draw_state.modifiers.travel_styles,
            ConfigSection::Weather => &mut self.draw_state.modifiers.weather,
            ConfigSection::Activities => &mut self.draw_state.modifiers.activities,
        }
    }

    fn handle_config_screen_input(&mut self, key: event::KeyEvent) -> Result<bool> {
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
                    // Save the modifier
                    let editor = self.draw_state.modifier_editor.take().unwrap();
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
                        self.save_config().ok();
                        self.update_quantities()?;
                    }
                }
                KeyCode::Backspace => {
                    match editor.field {
                        ModifierField::Name => { editor.name.pop(); }
                        ModifierField::Multiplier => { editor.multiplier.pop(); }
                    }
                }
                KeyCode::Char(c) => {
                    match editor.field {
                        ModifierField::Name => { editor.name.push(c); }
                        ModifierField::Multiplier => { editor.multiplier.push(c); }
                    }
                }
                _ => {}
            }
            return Ok(false);
        }

        match key.code {
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
                let count = self.config_section_modifiers(self.draw_state.config_section).len();
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
                    self.save_config().ok();
                    self.update_quantities()?;
                }
            }
            _ => {}
        }
        Ok(false)
    }

    /// Keep the current travel style / weather / activity selections valid
    /// after modifiers change
    fn reconcile_selections(&mut self) {
        if self.draw_state.modifiers.travel_style_multiplier(&self.draw_state.travel_style).is_none() {
            self.draw_state.travel_style = self.draw_state.modifiers.travel_styles.first()
                .map(|m| m.name.clone())
                .unwrap_or_default();
        }
        if self.draw_state.modifiers.weather_multiplier(&self.draw_state.weather).is_none() {
            self.draw_state.weather = self.draw_state.modifiers.weather.first()
                .map(|m| m.name.clone())
                .unwrap_or_default();
        }
        self.draw_state.selected_activities.retain(|name| {
            self.draw_state.modifiers.activity_multiplier(name).is_some()
        });
    }

    fn handle_add_edit_input(&mut self, key: event::KeyEvent) {
        match &mut self.draw_state.input_mode {
            InputMode::AddingItem { field, name, category, quantity, .. } => {
                match key.code {
                    KeyCode::Char(c) => {
                        match field {
                            InputField::Name => name.push(c),
                            InputField::Category => category.push(c),
                            InputField::Quantity => quantity.push(c),
                        }
                    }
                    KeyCode::Backspace => {
                        match field {
                            InputField::Name => { name.pop(); }
                            InputField::Category => { category.pop(); }
                            InputField::Quantity => { quantity.pop(); }
                        }
                    }
                    _ => {}
                }
            }
            InputMode::EditingItem { field, name, category, quantity, .. } => {
                match key.code {
                    KeyCode::Char(c) => {
                        match field {
                            InputField::Name => name.push(c),
                            InputField::Category => category.push(c),
                            InputField::Quantity => quantity.push(c),
                        }
                    }
                    KeyCode::Backspace => {
                        match field {
                            InputField::Name => { name.pop(); }
                            InputField::Category => { category.pop(); }
                            InputField::Quantity => { quantity.pop(); }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    // The item currently selected on the focused side, if the cursor is on an item row
    fn current_item_index(&self) -> Option<usize> {
        let rows = self.draw_state.active_rows();
        match rows.get(self.draw_state.selected_index) {
            Some(PanelRow::Item(index)) => Some(*index),
            _ => None,
        }
    }

    // The category the cursor is on (header row or one of its items)
    fn current_category(&self) -> Option<String> {
        let rows = self.draw_state.active_rows();
        match rows.get(self.draw_state.selected_index) {
            Some(PanelRow::Category(c)) => Some(c.clone()),
            Some(PanelRow::Item(index)) => Some(self.draw_state.packing_list.items[*index].category.clone()),
            None => None,
        }
    }

    // Collapse (true) or expand (false) the category under the cursor.
    // When collapsing, the cursor moves to the category's header row so it
    // stays visible.
    fn set_category_collapsed(&mut self, collapsed: bool) {
        let Some(category) = self.current_category() else {
            return;
        };

        if collapsed {
            if !self.draw_state.collapsed_categories.contains(&category) {
                self.draw_state.collapsed_categories.push(category);
            }
            // Move the cursor to the category's header row
            let rows = self.draw_state.active_rows();
            if let Some(row) = rows.iter().position(|r| matches!(r, PanelRow::Category(c) if *c == category)) {
                self.draw_state.selected_index = row;
            }
        } else {
            self.draw_state.collapsed_categories.retain(|c| c != &category);
        }
    }

    fn toggle_category_collapse(&mut self) {
        // Toggle collapse of the category the cursor is on
        let collapsed = self.current_category()
            .map(|c| self.draw_state.collapsed_categories.contains(&c))
            .unwrap_or(false);
        self.set_category_collapsed(!collapsed);
    }

    fn toggle_item_selection(&mut self) {
        // Toggle selection of the currently selected item
        if let Some(item_index) = self.current_item_index() {
            let item = &mut self.draw_state.packing_list.items[item_index];
            item.editable = !item.editable;
        }
    }

    fn toggle_category_selection(&mut self) {
        // Toggle selection of all items in the current category, then
        // collapse the group when items were added to the trip and expand
        // it when they were removed
        let Some(category) = self.current_category() else {
            return;
        };

        let was_selected = self.draw_state.packing_list.items.iter()
            .any(|item| item.category == category && item.editable);

        for item in &mut self.draw_state.packing_list.items {
            if item.category == category {
                item.editable = !item.editable;
            }
        }

        // Follow the selection state: added -> collapse, removed -> expand
        let now_selected = !was_selected;
        self.set_category_collapsed(!now_selected);
    }

    fn toggle_item_packed_right_side(&mut self) {
        // Only toggle packed status when on right side
        if self.draw_state.focus_side == FocusSide::Right {
            if let Some(item_index) = self.current_item_index() {
                self.draw_state.packing_list.items[item_index].packed =
                    !self.draw_state.packing_list.items[item_index].packed;
            }
        }
    }

    fn start_editing_item(&mut self) {
        if let Some(item_index) = self.current_item_index() {
            let item = &self.draw_state.packing_list.items[item_index];
            let categories = self.get_all_categories();
            let category_index = categories.iter().position(|c| c == &item.category).unwrap_or(0);
            
            self.draw_state.input_mode = InputMode::EditingItem {
                index: item_index,
                field: InputField::Name,
                name: item.name.clone(),
                category: item.category.clone(),
                quantity: item.quantity.to_string(),
                category_list: categories,
                category_index,
            };
        }
    }

    fn save_new_item(&mut self, name: String, category: String, quantity: String) {
        // Parse quantity
        let qty = quantity.parse::<u32>().unwrap_or(1);
        
        // Create new item
        let new_id = format!("item_{}", self.draw_state.packing_list.items.len() + 1);
        let new_name = if name.is_empty() { "New Item".to_string() } else { name };
        let new_category = if category.is_empty() { 
            // Use first category or default
            if !self.draw_state.packing_list.items.is_empty() {
                self.draw_state.packing_list.items[0].category.clone()
            } else {
                "Other".to_string()
            }
        } else { category };
        
        self.draw_state.packing_list.add_item(Item::new(
            new_id,
            new_name,
            new_category,
            qty,
        ));
        
        // Update quantities for the new item
        self.update_quantities().ok();
        
        // Save config after adding
        self.save_config().ok();
        
        // Reset selection to the new item, clamped to the visible rows
        let rows = self.draw_state.active_rows();
        self.draw_state.selected_index = (self.draw_state.packing_list.items.len() - 1).min(rows.len().saturating_sub(1));
    }

    fn save_edited_item(&mut self, index: usize, name: String, category: String, quantity: String) {
        if index < self.draw_state.packing_list.items.len() {
            let item = &mut self.draw_state.packing_list.items[index];
            
            // Update fields if they're not empty
            if !name.is_empty() {
                item.name = name;
            }
            if !category.is_empty() {
                item.category = category;
            }
            if let Ok(qty) = quantity.parse::<u32>() {
                item.quantity = qty;
            }
            
            // Save config after editing
            self.save_config().ok();
        }
    }

    fn remove_item_confirmed(&mut self, index: usize) {
        if index < self.draw_state.packing_list.items.len() {
            self.draw_state.packing_list.items.remove(index);

            // Adjust selection index to stay within the visible rows
            let rows = self.draw_state.active_rows();
            if self.draw_state.selected_index >= rows.len() {
                self.draw_state.selected_index = rows.len().saturating_sub(1);
            }
        }
    }
}

/// Trip parameters loaded from (or defaulted into) the config file
struct TripParams {
    weekdays: Weekdays,
    travel_style: String,
    weather: String,
    selected_activities: Vec<String>,
    modifiers: ModifiersConfig,
    last_trip_file: Option<String>,
}

fn load_config(config_file: &PathBuf) -> Result<(PackingList, TripParams)> {
    let yaml_content = fs::read_to_string(config_file)
        .context("Failed to read config file")?;

    let config: AppConfig = serde_yaml::from_str(&yaml_content)
        .context("Failed to parse config file")?;

    // Create packing list from config
    let mut packing_list = PackingList::new("My Trip".to_string());

    for saved_item in config.items {
        let new_id = format!("item_{}", packing_list.items.len() + 1);
        let auto = saved_item.auto_quantity;
        let fixed_quantity = if auto { 1 } else { saved_item.quantity.unwrap_or(1) };
        let mut item = Item::new(
            new_id,
            saved_item.name,
            saved_item.category,
            fixed_quantity,
        );
        item.auto_quantity = auto;
        packing_list.add_item(item);
    }

    let weekdays = Weekdays::new(config.weekdays).unwrap_or_default();
    let trip_params = TripParams {
        weekdays,
        travel_style: config.travel_style,
        weather: config.weather,
        selected_activities: config.selected_activities,
        modifiers: config.modifiers,
        last_trip_file: config.last_trip_file,
    };

    Ok((packing_list, trip_params))
}

fn create_default_config(config_file: &PathBuf) -> Result<(PackingList, TripParams)> {
    // Create default packing list
    let mut packing_list = PackingList::new("My Trip".to_string());

    // Add default items
    let default_items = vec![
        ("T-Shirt", "Clothing", 5),
        ("Pants", "Clothing", 3),
        ("Socks", "Clothing", 7),
        ("Underwear", "Clothing", 7),
        ("Jacket", "Clothing", 1),
        ("Toothbrush", "Toiletries", 1),
        ("Toothpaste", "Toiletries", 1),
        ("Shampoo", "Toiletries", 1),
        ("Passport", "Documents", 1),
        ("Tickets", "Documents", 1),
        ("Phone", "Electronics", 1),
        ("Charger", "Electronics", 1),
        ("Shoes", "Footwear", 1),
        ("Hat", "Accessories", 1),
        ("Sunglasses", "Accessories", 1),
    ];

    for (name, category, quantity) in default_items {
        let new_id = format!("item_{}", packing_list.items.len() + 1);
        packing_list.add_item(Item::new(
            new_id,
            name.to_string(),
            category.to_string(),
            quantity,
        ));
    }

    // Default trip parameters
    let trip_params = TripParams {
        weekdays: Weekdays::default(),
        travel_style: default_travel_style_name(),
        weather: default_weather_name(),
        selected_activities: Vec::new(),
        modifiers: ModifiersConfig::default(),
        last_trip_file: None,
    };

    // Save config
    let config = AppConfig {
        categories: vec![
            "Clothing".to_string(),
            "Toiletries".to_string(),
            "Documents".to_string(),
            "Electronics".to_string(),
            "Footwear".to_string(),
            "Accessories".to_string(),
            "Miscellaneous".to_string(),
        ],
        items: packing_list.items.iter()
            .map(|item| {
                let auto = item.auto_quantity;
                SavedItem {
                    name: item.name.clone(),
                    category: item.category.clone(),
                    quantity: if auto { None } else { Some(item.quantity) },
                    auto_quantity: auto,
                }
            })
            .collect(),
        weekdays: trip_params.weekdays.get(),
        travel_style: trip_params.travel_style.clone(),
        weather: trip_params.weather.clone(),
        selected_activities: trip_params.selected_activities.clone(),
        modifiers: trip_params.modifiers.clone(),
        last_trip_file: None,
    };

    let yaml = serde_yaml::to_string(&config)
        .context("Failed to serialize default config")?;

    if let Some(parent) = config_file.parent() {
        fs::create_dir_all(parent)
            .context("Failed to create config directory")?;
    }

    fs::write(config_file, yaml)
        .context("Failed to write default config file")?;

    Ok((packing_list, trip_params))
}

fn draw_welcome_screen(f: &mut ratatui::prelude::Frame, size: Rect, state: &AppDrawState) {
    let [title_area, content_area, instructions_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(12),
            Constraint::Length(4),
        ])
        .split(size) else {
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
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Min(6),
        ])
        .split(content_area);

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
        Line::from(""),
        Line::from(format!("  Current: {} days", state.weekdays.get())),
        Line::from(""),
        Line::from("  Use Up/Down or +/- to adjust".dim()),
    ];
    let weekdays_para = Paragraph::new(weekdays_text);
    f.render_widget(weekdays_block, params_layout[0]);
    f.render_widget(weekdays_para, params_layout[0]);

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
        Line::from(""),
        Line::from(format!("  Current: {}", state.travel_style)),
        Line::from(""),
        Line::from("  Use Up/Down to cycle through options".dim()),
    ];
    let style_para = Paragraph::new(style_text);
    f.render_widget(style_block, params_layout[1]);
    f.render_widget(style_para, params_layout[1]);

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
        Line::from(""),
        Line::from(format!("  Current: {}", state.weather)),
        Line::from(""),
        Line::from("  Use Up/Down to cycle through options".dim()),
    ];
    let weather_para = Paragraph::new(weather_text);
    f.render_widget(weather_block, params_layout[2]);
    f.render_widget(weather_para, params_layout[2]);

    // Trip file name (persist target for the compiled packing list)
    let file_block = Block::default()
        .title("Packing List File")
        .borders(Borders::ALL)
        .border_style(if state.editing_trip_file {
            Style::new().fg(Color::Yellow)
        } else {
            Style::new()
        });

    let file_text = if state.editing_trip_file {
        vec![
            Line::from(""),
            Line::from(format!("  Name: {}_", state.trip_file_input)),
            Line::from(""),
            Line::from("  Enter to confirm | Esc to cancel".dim()),
        ]
    } else {
        let shown = if state.trip_file_name.is_empty() {
            "(none - set a name to persist your packing list)".dim()
        } else {
            state.trip_file_name.clone().into()
        };
        vec![
            Line::from(""),
            Line::from(vec![Span::raw("  File: "), shown]),
            Line::from(""),
            Line::from("  Press F to edit (saved under ~/.config/pack-liste/trips/)".dim()),
        ]
    };
    let file_para = Paragraph::new(file_text);
    f.render_widget(file_block, params_layout[3]);
    f.render_widget(file_para, params_layout[3]);

    // Activities (multi-select, scrollable)
    let activities_block = Block::default()
        .title("Activities")
        .borders(Borders::ALL)
        .border_style(if state.focus == Focus::Activities {
            Style::new().fg(Color::Yellow)
        } else {
            Style::new()
        });

    let activities_area = params_layout[4];
    let activities_inner = Rect {
        x: activities_area.x + 1,
        y: activities_area.y + 1,
        width: activities_area.width.saturating_sub(2),
        height: activities_area.height.saturating_sub(2),
    };

    // Window the activity list so the cursor stays visible
    let viewport_height = activities_inner.height as usize;
    let activity_scroll = if state.activity_cursor >= viewport_height {
        state.activity_cursor + 1 - viewport_height
    } else {
        0
    };

    let activity_lines: Vec<Line> = state.modifiers.activities.iter()
        .enumerate()
        .skip(activity_scroll)
        .take(viewport_height)
        .map(|(i, activity)| {
            let is_selected = state.selected_activities.contains(&activity.name);
            let is_cursor = state.focus == Focus::Activities && i == state.activity_cursor;
            let symbol = if is_selected { "[x] " } else { "[ ] " };
            let mut line = Line::from(vec![
                Span::raw("  "),
                Span::raw(symbol),
                Span::raw(activity.name.clone()),
            ]);
            if is_cursor {
                line = line.style(Style::new().fg(Color::Yellow).bg(Color::DarkGray));
            } else if is_selected {
                line = line.style(Style::new().fg(Color::Green));
            }
            line
        })
        .collect();

    let activities_para = Paragraph::new(activity_lines);
    f.render_widget(activities_block, activities_area);
    f.render_widget(activities_para, activities_inner);

    // Instructions
    let instructions = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Tab").bold().fg(Color::Yellow),
            Span::raw(" to navigate between fields"),
        ]),
        Line::from(vec![
            Span::raw("Enter").bold().fg(Color::Yellow),
            Span::raw(" to continue to packing list"),
        ]),
        Line::from(vec![
            Span::raw("q").bold().fg(Color::Red),
            Span::raw(" to quit"),
        ]),
    ])
    .wrap(Wrap { trim: true });
    f.render_widget(instructions, instructions_area);
}

fn draw_main_screen(f: &mut ratatui::prelude::Frame, size: Rect, state: &mut AppDrawState) {
    // Split screen: left (all items), right (selected items only)
    let [left_area, right_area] = *Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(size) else {
        return;
    };

    // Draw left panel - all items with selection toggle
    draw_all_items_panel(f, left_area, state);

    // Draw right panel - only selected items
    draw_selected_items_panel(f, right_area, state);
}

fn draw_help_screen(f: &mut ratatui::prelude::Frame, size: Rect) {
    let block = Block::default()
        .title("Help - Navigation & Controls")
        .borders(Borders::ALL)
        .style(Style::new().bg(Color::Black).fg(Color::White));
    f.render_widget(block, size);

    let inner = Rect {
        x: size.x + 1,
        y: size.y + 1,
        width: size.width - 2,
        height: size.height - 2,
    };

    let help_text = vec![
        Line::from(vec![
            Span::raw("WELCOME SCREEN:").bold().fg(Color::Yellow),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Up/Down").bold().fg(Color::Yellow),
            Span::raw(" - Adjust weekdays / Cycle options / Move activity cursor"),
        ]),
        Line::from(vec![
            Span::raw("+/-").bold().fg(Color::Yellow),
            Span::raw(" - Adjust weekdays"),
        ]),
        Line::from(vec![
            Span::raw("Space").bold().fg(Color::Yellow),
            Span::raw(" - Toggle activity (in Activities section)"),
        ]),
        Line::from(vec![
            Span::raw("F").bold().fg(Color::Yellow),
            Span::raw(" - Edit packing list file name"),
        ]),
        Line::from(vec![
            Span::raw("O").bold().fg(Color::Yellow),
            Span::raw(" - Open configuration (modifiers)"),
        ]),
        Line::from(vec![
            Span::raw("Tab").bold().fg(Color::Yellow),
            Span::raw(" - Navigate between fields"),
        ]),
        Line::from(vec![
            Span::raw("Enter").bold().fg(Color::Yellow),
            Span::raw(" - Continue to main screen"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("CONFIG SCREEN:").bold().fg(Color::Yellow),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Tab").bold().fg(Color::Yellow),
            Span::raw(" - Switch section (styles/weather/activities)"),
        ]),
        Line::from(vec![
            Span::raw("A/E/D").bold().fg(Color::Yellow),
            Span::raw(" - Add / Edit / Delete modifier"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("LEFT PANEL (All Items):").bold().fg(Color::Green),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("↑/↓").bold().fg(Color::Yellow),
            Span::raw(" - Navigate items"),
        ]),
        Line::from(vec![
            Span::raw("Space").bold().fg(Color::Yellow),
            Span::raw(" - Toggle item selection"),
        ]),
        Line::from(vec![
            Span::raw("Ctrl+Space").bold().fg(Color::Yellow),
            Span::raw(" - Toggle all items in category"),
        ]),
        Line::from(vec![
            Span::raw("A").bold().fg(Color::Yellow),
            Span::raw(" - Add new item"),
        ]),
        Line::from(vec![
            Span::raw("E").bold().fg(Color::Yellow),
            Span::raw(" - Edit selected item"),
        ]),
        Line::from(vec![
            Span::raw("D").bold().fg(Color::Yellow),
            Span::raw(" - Delete selected item (with confirmation)"),
        ]),
        Line::from(vec![
            Span::raw("C").bold().fg(Color::Yellow),
            Span::raw(" - Collapse/expand category"),
        ]),
        Line::from(vec![
            Span::raw("Tab").bold().fg(Color::Yellow),
            Span::raw(" - Switch to RIGHT panel"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("RIGHT PANEL (Selected Items):").bold().fg(Color::Cyan),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("↑/↓").bold().fg(Color::Yellow),
            Span::raw(" - Navigate selected items"),
        ]),
        Line::from(vec![
            Span::raw("Space").bold().fg(Color::Yellow),
            Span::raw(" - Toggle packed status"),
        ]),
        Line::from(vec![
            Span::raw("Tab").bold().fg(Color::Yellow),
            Span::raw(" - Switch to LEFT panel"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("ADD/EDIT POPUP:").bold().fg(Color::Yellow),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Tab").bold().fg(Color::Yellow),
            Span::raw(" - Cycle through fields (Name → Category → Quantity)"),
        ]),
        Line::from(vec![
            Span::raw("↑/↓").bold().fg(Color::Yellow),
            Span::raw(" - Select category from dropdown"),
        ]),
        Line::from(vec![
            Span::raw("Enter").bold().fg(Color::Yellow),
            Span::raw(" - Save changes"),
        ]),
        Line::from(vec![
            Span::raw("Esc").bold().fg(Color::Red),
            Span::raw(" - Cancel"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("GENERAL:").bold().fg(Color::Yellow),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("?").bold().fg(Color::Yellow),
            Span::raw(" - Show this help"),
        ]),
        Line::from(vec![
            Span::raw("q").bold().fg(Color::Red),
            Span::raw(" - Quit application"),
        ]),
    ];

    let help_para = Paragraph::new(help_text)
        .wrap(Wrap { trim: true });
    f.render_widget(help_para, inner);
}

fn draw_config_screen(f: &mut ratatui::prelude::Frame, size: Rect, state: &AppDrawState) {
    let [title_area, content_area, instructions_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(size) else {
        return;
    };

    // Title
    let title = Paragraph::new("Configuration - Modifiers")
        .style(Style::new().bold().fg(Color::Green));
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
            .title(if editor.index.is_some() { "Edit Modifier" } else { "Add Modifier" })
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
            Span::raw("Name: ").style(if editor.field == ModifierField::Name { field_style } else { normal_style }),
            Span::raw(&editor.name),
        ]);
        let multiplier_line = Line::from(vec![
            Span::raw("Multiplier: ").style(if editor.field == ModifierField::Multiplier { field_style } else { normal_style }),
            Span::raw(&editor.multiplier),
        ]);
        let hint = Line::from("Tab switch field | Enter save | Esc cancel".dim());

        let para = Paragraph::new(vec![name_line, multiplier_line, hint]);
        f.render_widget(para, inner);
        return;
    }

    // Section tabs
    let tabs = Line::from(vec![
        Span::raw(if state.config_section == ConfigSection::TravelStyles { "[Travel Styles]" } else { " Travel Styles " })
            .style(if state.config_section == ConfigSection::TravelStyles {
                Style::new().fg(Color::Yellow).bold()
            } else {
                Style::new().fg(Color::DarkGray)
            }),
        Span::raw(" | "),
        Span::raw(if state.config_section == ConfigSection::Weather { "[Weather]" } else { " Weather " })
            .style(if state.config_section == ConfigSection::Weather {
                Style::new().fg(Color::Yellow).bold()
            } else {
                Style::new().fg(Color::DarkGray)
            }),
        Span::raw(" | "),
        Span::raw(if state.config_section == ConfigSection::Activities { "[Activities]" } else { " Activities " })
            .style(if state.config_section == ConfigSection::Activities {
                Style::new().fg(Color::Yellow).bold()
            } else {
                Style::new().fg(Color::DarkGray)
            }),
    ]);
    f.render_widget(Paragraph::new(tabs), Rect {
        x: content_area.x,
        y: content_area.y,
        width: content_area.width,
        height: 1,
    });

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
        ConfigSection::Activities => &state.modifiers.activities,
    };

    // Window the list so the cursor stays visible
    let viewport_height = list_area.height as usize;
    let mut scroll = state.config_scroll.min(modifiers.len().saturating_sub(viewport_height));
    if state.config_cursor >= scroll + viewport_height {
        scroll = state.config_cursor + 1 - viewport_height;
    } else if state.config_cursor < scroll {
        scroll = state.config_cursor;
    }

    let lines: Vec<Line> = modifiers.iter()
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
    f.render_widget(Paragraph::new(lines), Rect {
        x: list_area.x + 1,
        y: list_area.y + 1,
        width: list_area.width.saturating_sub(2),
        height: list_area.height.saturating_sub(2),
    });

    // Instructions
    let instructions = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Tab").bold().fg(Color::Yellow),
            Span::raw(" Switch section | "),
            Span::raw("↑/↓").bold().fg(Color::Yellow),
            Span::raw(" Move | "),
            Span::raw("A").bold().fg(Color::Yellow),
            Span::raw(" Add | "),
            Span::raw("E").bold().fg(Color::Yellow),
            Span::raw(" Edit | "),
            Span::raw("D").bold().fg(Color::Yellow),
            Span::raw(" Delete | "),
            Span::raw("Esc").bold().fg(Color::Red),
            Span::raw(" Back"),
        ]),
    ])
    .wrap(Wrap { trim: true });
    f.render_widget(instructions, instructions_area);
}

fn draw_all_items_panel(f: &mut ratatui::prelude::Frame, area: Rect, state: &mut AppDrawState) {
    let [title_area, list_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(10),
        ])
        .split(area) else {
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
    let mut categories_map: std::collections::BTreeMap<String, Vec<(usize, &Item)>> = std::collections::BTreeMap::new();
    for (idx, item) in state.packing_list.items.iter().enumerate() {
        categories_map.entry(item.category.clone()).or_default().push((idx, item));
    }

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
    let mut scroll = state.left_scroll.min(total_rows.saturating_sub(viewport_height));
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
            Span::raw(category).bold().fg(Color::Cyan),
        ]);

        let category_para = Paragraph::new(vec![category_line])
            .block(Block::default().borders(Borders::NONE));

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
                let highlight = Block::default()
                    .style(Style::new().bg(Color::DarkGray));
                f.render_widget(highlight, category_area);
            }
        }

        row_index += 1;

        // Draw items if category is not collapsed
        if !is_collapsed {
            for (_, item) in items {
                let is_selected = item.editable;
                let is_current = current_row == Some(row_index);

                let item_style = if is_current {
                    Style::new().fg(Color::Yellow).bg(Color::DarkGray)
                } else {
                    Style::new()
                };

                let selection_symbol = if is_selected { "✓" } else { "○" };
                let item_symbol = if item.packed { "✓ " } else { "○ " };

                let item_line = Line::from(vec![
                    Span::raw("  "),
                    Span::raw(selection_symbol).fg(if is_selected { Color::Green } else { Color::Red }),
                    Span::raw(" "),
                    Span::raw(item_symbol).fg(if item.packed { Color::Green } else { Color::Red }),
                    Span::raw(" "),
                    Span::raw(&item.name).style(item_style),
                    Span::raw(" x"),
                    Span::raw(item.quantity.to_string()).bold(),
                ]);

                let item_para = Paragraph::new(vec![item_line])
                    .block(Block::default().borders(Borders::NONE));

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

    // Instructions at bottom if space available
    let visible_rows = total_rows.min(viewport_height);
    if visible_rows + 5 < list_area.height as usize {
        let instructions_y = (visible_rows) as u16;
        let instructions_area = Rect {
            x: list_area.x,
            y: list_area.y + instructions_y,
            width: list_area.width,
            height: list_area.height - instructions_y,
        };

        let instructions = Paragraph::new(vec![
            Line::from(vec![
                Span::raw("↑/↓").bold().fg(Color::Yellow),
                Span::raw(" Navigate | "),
                Span::raw("Space").bold().fg(Color::Yellow),
                Span::raw(" Select | "),
                Span::raw("Ctrl+Space").bold().fg(Color::Yellow),
                Span::raw(" Category | "),
                Span::raw("A").bold().fg(Color::Yellow),
                Span::raw(" Add | "),
                Span::raw("E").bold().fg(Color::Yellow),
                Span::raw(" Edit | "),
                Span::raw("D").bold().fg(Color::Yellow),
                Span::raw(" Delete | "),
                Span::raw("C").bold().fg(Color::Yellow),
                Span::raw(" Collapse"),
            ]),
            Line::from(vec![
                Span::raw("Tab").bold().fg(Color::Yellow),
                Span::raw(" Switch | "),
                Span::raw("?").bold().fg(Color::Yellow),
                Span::raw(" Help"),
            ]),
        ])
        .wrap(Wrap { trim: true });
        f.render_widget(instructions, instructions_area);
    }
}

fn draw_selected_items_panel(f: &mut ratatui::prelude::Frame, area: Rect, state: &mut AppDrawState) {
    let [title_area, list_area, stats_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(5),
        ])
        .split(area) else {
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
    let selected_items: Vec<(usize, &Item)> = state.packing_list.items.iter()
        .enumerate()
        .filter(|(_, item)| item.editable)
        .collect();

    // Cursor position within the panel's item rows (only when the right side is focused)
    let current_row = if state.focus_side == FocusSide::Right {
        Some(state.selected_index)
    } else {
        None
    };

    // Group selected items by category
    let mut categories_map: std::collections::BTreeMap<String, Vec<(usize, &Item)>> = std::collections::BTreeMap::new();
    for (idx, item) in &selected_items {
        categories_map.entry(item.category.clone()).or_default().push((*idx, item));
    }

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
            drawn_rows.iter().position(|r| matches!(r, PanelRow::Item(i) if *i == item))
        })
    });

    // Visible window: keep the cursor row in view, otherwise keep the current offset
    let viewport_height = list_area.height as usize;
    let mut scroll = state.right_scroll.min(total_rows.saturating_sub(viewport_height));
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
        let y = list_area.y + (row_index - scroll) as u16;

        match row {
            PanelRow::Category(category) => {
                let category_line = Line::from(vec![
                    Span::raw("▼ "),
                    Span::raw(category).bold().fg(Color::Cyan),
                ]);

                let category_para = Paragraph::new(vec![category_line])
                    .block(Block::default().borders(Borders::NONE));

                if in_view {
                    let category_area = Rect {
                        x: list_area.x,
                        y,
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

                let item_line = Line::from(vec![
                    Span::raw("  "),
                    Span::raw(item_symbol).fg(if item.packed { Color::Green } else { Color::Red }),
                    Span::raw(" "),
                    Span::raw(&item.name).style(item_style),
                    Span::raw(" x"),
                    Span::raw(item.quantity.to_string()).bold(),
                ]);

                let item_para = Paragraph::new(vec![item_line])
                    .block(Block::default().borders(Borders::NONE));

                if in_view {
                    let item_area = Rect {
                        x: list_area.x,
                        y,
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
    let packed = selected_items.iter().filter(|(_, item)| item.packed).count();
    
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
            Span::raw(format!("{:.0}%", if total_selected > 0 { (packed as f32 / total_selected as f32 * 100.0) } else { 0.0 })).bold(),
        ]),
    ])
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(stats, stats_area);
}

fn draw_add_item_popup(f: &mut ratatui::prelude::Frame, size: Rect, field: InputField, name: &str, category: &str, quantity: &str, category_list: &[String], category_index: usize) {
    // Create a centered popup with dynamic size
    let popup_width = (size.width * 60 / 100).min(60).max(40);
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
        .title("Add New Item")
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
        Span::raw("Name: ").style(if field == InputField::Name { field_style } else { normal_style }),
        Span::raw(name),
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
    let category_line = Line::from(vec![
        Span::raw("Category: ").style(if field == InputField::Category { field_style } else { normal_style }),
    ]);
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
        Span::raw("Quantity: ").style(if field == InputField::Quantity { field_style } else { normal_style }),
        Span::raw(quantity),
    ]);
    let quantity_para = Paragraph::new(vec![quantity_line]);
    f.render_widget(quantity_para, quantity_area);

    // Draw instructions
    let instructions_area = Rect {
        x: content_area.x,
        y: content_area.y + 4,
        width: content_area.width,
        height: content_area.height - 4,
    };
    let instructions = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Tab").bold().fg(Color::Yellow),
            Span::raw(" - Cycle fields"),
        ]),
        Line::from(vec![
            Span::raw("↑/↓").bold().fg(Color::Yellow),
            Span::raw(" - Select category"),
        ]),
        Line::from(vec![
            Span::raw("Enter").bold().fg(Color::Yellow),
            Span::raw(" - Save"),
        ]),
        Line::from(vec![
            Span::raw("Esc").bold().fg(Color::Red),
            Span::raw(" - Cancel"),
        ]),
    ]);
    f.render_widget(instructions, instructions_area);

    // Draw category dropdown or selected category (last, so the open dropdown overlays the fields below)
    if field == InputField::Category {
        // Draw dropdown list
        let category_items: Vec<ListItem> = category_list
            .iter()
            .map(|c| ListItem::new(c.as_str()))
            .collect();

        let category_list_widget = List::new(category_items)
            .highlight_style(Style::new().fg(Color::Yellow).bold())
            .highlight_symbol("> ");

        let dropdown_height = 5.min(category_list.len() as u16 + 1);
        let dropdown_list_area = Rect {
            x: dropdown_area.x,
            y: dropdown_area.y,
            width: dropdown_area.width,
            height: dropdown_height,
        };
        f.render_widget(category_list_widget, dropdown_list_area);

        // Highlight selected category
        if category_index < category_list.len() {
            let selected_category = &category_list[category_index];
            let selected_line = Line::from(vec![
                Span::raw(selected_category).fg(Color::Yellow).bold(),
            ]);
            let selected_para = Paragraph::new(vec![selected_line]);
            f.render_widget(selected_para, dropdown_area);
        }
    } else {
        // Show selected category text
        let selected_category = if category_index < category_list.len() {
            &category_list[category_index]
        } else {
            category
        };
        let category_text = Paragraph::new(vec![Line::from(selected_category)]);
        f.render_widget(category_text, dropdown_area);
    }
}

fn draw_edit_item_popup(f: &mut ratatui::prelude::Frame, size: Rect, field: InputField, name: &str, category: &str, quantity: &str, category_list: &[String], category_index: usize) {
    // Create a centered popup with dynamic size
    let popup_width = (size.width * 60 / 100).min(60).max(40);
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
        .title("Edit Item")
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
        Span::raw("Name: ").style(if field == InputField::Name { field_style } else { normal_style }),
        Span::raw(name),
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
    let category_line = Line::from(vec![
        Span::raw("Category: ").style(if field == InputField::Category { field_style } else { normal_style }),
    ]);
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
        Span::raw("Quantity: ").style(if field == InputField::Quantity { field_style } else { normal_style }),
        Span::raw(quantity),
    ]);
    let quantity_para = Paragraph::new(vec![quantity_line]);
    f.render_widget(quantity_para, quantity_area);

    // Draw instructions
    let instructions_area = Rect {
        x: content_area.x,
        y: content_area.y + 4,
        width: content_area.width,
        height: content_area.height - 4,
    };
    let instructions = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Tab").bold().fg(Color::Yellow),
            Span::raw(" - Cycle fields"),
        ]),
        Line::from(vec![
            Span::raw("↑/↓").bold().fg(Color::Yellow),
            Span::raw(" - Select category"),
        ]),
        Line::from(vec![
            Span::raw("Enter").bold().fg(Color::Yellow),
            Span::raw(" - Save"),
        ]),
        Line::from(vec![
            Span::raw("Esc").bold().fg(Color::Red),
            Span::raw(" - Cancel"),
        ]),
    ]);
    f.render_widget(instructions, instructions_area);

    // Draw category dropdown or selected category (last, so the open dropdown overlays the fields below)
    if field == InputField::Category {
        // Draw dropdown list
        let category_items: Vec<ListItem> = category_list
            .iter()
            .map(|c| ListItem::new(c.as_str()))
            .collect();

        let category_list_widget = List::new(category_items)
            .highlight_style(Style::new().fg(Color::Yellow).bold())
            .highlight_symbol("> ");

        let dropdown_height = 5.min(category_list.len() as u16 + 1);
        let dropdown_list_area = Rect {
            x: dropdown_area.x,
            y: dropdown_area.y,
            width: dropdown_area.width,
            height: dropdown_height,
        };
        f.render_widget(category_list_widget, dropdown_list_area);

        // Highlight selected category
        if category_index < category_list.len() {
            let selected_category = &category_list[category_index];
            let selected_line = Line::from(vec![
                Span::raw(selected_category).fg(Color::Yellow).bold(),
            ]);
            let selected_para = Paragraph::new(vec![selected_line]);
            f.render_widget(selected_para, dropdown_area);
        }
    } else {
        // Show selected category text
        let selected_category = if category_index < category_list.len() {
            &category_list[category_index]
        } else {
            category
        };
        let category_text = Paragraph::new(vec![Line::from(selected_category)]);
        f.render_widget(category_text, dropdown_area);
    }
}

fn draw_confirm_delete_popup(f: &mut ratatui::prelude::Frame, size: Rect, index: usize, packing_list: &PackingList) {
    // Create a centered popup with dynamic size
    let popup_width = (size.width * 40 / 100).min(40).max(30);
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
        Line::from(vec![
            Span::raw("Enter").bold().fg(Color::Yellow),
            Span::raw(" - Confirm"),
        ]),
        Line::from(vec![
            Span::raw("Esc").bold().fg(Color::Red),
            Span::raw(" - Cancel"),
        ]),
    ];
    
    let content = Paragraph::new(lines);
    f.render_widget(content, content_area);
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = stdout().execute(LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}
