use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use std::io::{self, stdout};

use crate::models::activity::Activity;
use crate::models::item::Item;
use crate::models::packing_list::PackingList;
use crate::models::travel_style::TravelStyle;
use crate::models::weather::WeatherInput;
use crate::models::weekdays::Weekdays;
use crate::services::quantity_service::QuantityService;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    Welcome,
    Main,
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
    Categories,
    Items,
}

impl Focus {
    pub fn next(&self) -> Self {
        match self {
            Focus::Weekdays => Focus::TravelStyle,
            Focus::TravelStyle => Focus::Weather,
            Focus::Weather => Focus::Activities,
            Focus::Activities => Focus::Categories,
            Focus::Categories => Focus::Items,
            Focus::Items => Focus::Weekdays,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Focus::Weekdays => Focus::Items,
            Focus::TravelStyle => Focus::Weekdays,
            Focus::Weather => Focus::TravelStyle,
            Focus::Activities => Focus::Weather,
            Focus::Categories => Focus::Activities,
            Focus::Items => Focus::Categories,
        }
    }
}

pub enum AppState {
    Running,
    Exit,
}

// State for drawing (cloneable)
#[derive(Clone)]
struct AppDrawState {
    screen: Screen,
    packing_list: PackingList,
    weekdays: Weekdays,
    travel_style: TravelStyle,
    weather: WeatherInput,
    activities: Vec<Activity>,
    focus: Focus,
    focus_side: FocusSide,
    collapsed_categories: Vec<String>,
}

pub struct App {
    state: AppState,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    draw_state: AppDrawState,
    quantity_service: QuantityService,
}

impl App {
    pub fn new() -> Result<Self> {
        // Initialize terminal
        enable_raw_mode()
            .context("Failed to enable raw mode")?;
        stdout()
            .execute(EnterAlternateScreen)
            .context("Failed to enter alternate screen")?;

        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend)
            .context("Failed to create terminal")?;

        // Create a default packing list with categorized items
        let mut packing_list = PackingList::new("My Trip".to_string());
        
        // Add items by category
        packing_list.add_item(Item::new(
            "tshirt_1".to_string(),
            "T-Shirts".to_string(),
            "Clothing".to_string(),
            5,
        ));
        packing_list.add_item(Item::new(
            "pants_1".to_string(),
            "Pants".to_string(),
            "Clothing".to_string(),
            3,
        ));
        packing_list.add_item(Item::new(
            "socks_1".to_string(),
            "Socks".to_string(),
            "Clothing".to_string(),
            7,
        ));
        packing_list.add_item(Item::new(
            "underwear_1".to_string(),
            "Underwear".to_string(),
            "Clothing".to_string(),
            7,
        ));
        packing_list.add_item(Item::new(
            "toothbrush_1".to_string(),
            "Toothbrush".to_string(),
            "Toiletries".to_string(),
            1,
        ));
        packing_list.add_item(Item::new(
            "toothpaste_1".to_string(),
            "Toothpaste".to_string(),
            "Toiletries".to_string(),
            1,
        ));
        packing_list.add_item(Item::new(
            "shampoo_1".to_string(),
            "Shampoo".to_string(),
            "Toiletries".to_string(),
            1,
        ));
        packing_list.add_item(Item::new(
            "passport_1".to_string(),
            "Passport".to_string(),
            "Documents".to_string(),
            1,
        ));
        packing_list.add_item(Item::new(
            "ticket_1".to_string(),
            "Tickets".to_string(),
            "Documents".to_string(),
            1,
        ));

        let draw_state = AppDrawState {
            screen: Screen::Welcome,
            packing_list: packing_list.clone(),
            weekdays: Weekdays::default(),
            travel_style: TravelStyle::default(),
            weather: WeatherInput::default(),
            activities: Vec::new(),
            focus: Focus::Weekdays,
            focus_side: FocusSide::Left,
            collapsed_categories: Vec::new(),
        };

        let mut app = Self {
            state: AppState::Running,
            terminal,
            draw_state,
            quantity_service: QuantityService::new(),
        };

        // Initialize quantities
        app.update_quantities()?;

        Ok(app)
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
                AppState::Exit => break,
            }
        }

        Ok(())
    }

    fn update_quantities(&mut self) -> Result<()> {
        // Calculate base quantities based on weekdays
        let base_quantities = self.quantity_service.calculate_base_quantities(self.draw_state.weekdays)?;
        
        // Apply travel style
        let style_quantities = self.quantity_service.calculate_travel_style(base_quantities, self.draw_state.travel_style)?;
        
        // Apply weather
        let weather_quantities = self.quantity_service.calculate_weather(style_quantities, self.draw_state.weather)?;
        
        // Apply activities
        let final_quantities = self.quantity_service.calculate_activities(weather_quantities, &self.draw_state.activities)?;
        
        // Update item quantities
        for (i, quantity) in final_quantities.iter().enumerate() {
            if i < self.draw_state.packing_list.items.len() {
                self.draw_state.packing_list.items[i].quantity = *quantity;
            }
        }
        
        Ok(())
    }

    fn draw(&mut self) -> Result<()> {
        let state = self.draw_state.clone();
        
        self.terminal.draw(|f| {
            let size = f.area();
            
            match state.screen {
                Screen::Welcome => {
                    draw_welcome_screen(f, size, &state);
                }
                Screen::Main => {
                    draw_main_screen(f, size, &state);
                }
            }
        })?;
        Ok(())
    }

    fn handle_input(&mut self) -> Result<bool> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            self.state = AppState::Exit;
                            return Ok(true);
                        }
                        KeyCode::Enter => {
                            match self.draw_state.screen {
                                Screen::Welcome => {
                                    // Move to main screen after configuring
                                    self.draw_state.screen = Screen::Main;
                                    self.draw_state.focus = Focus::Categories;
                                    self.draw_state.focus_side = FocusSide::Left;
                                }
                                Screen::Main => {
                                    // Toggle category/item selection
                                    self.handle_select()?;
                                }
                            }
                        }
                        KeyCode::Tab => {
                            match self.draw_state.screen {
                                Screen::Welcome => {
                                    self.draw_state.focus = self.draw_state.focus.next();
                                }
                                Screen::Main => {
                                    // Switch between left and right panels
                                    self.draw_state.focus_side = match self.draw_state.focus_side {
                                        FocusSide::Left => FocusSide::Right,
                                        FocusSide::Right => FocusSide::Left,
                                    };
                                    // Reset focus based on side
                                    if self.draw_state.focus_side == FocusSide::Left {
                                        self.draw_state.focus = Focus::Categories;
                                    } else {
                                        self.draw_state.focus = Focus::Categories; // Will be adjusted in draw
                                    }
                                }
                            }
                        }
                        KeyCode::BackTab => {
                            match self.draw_state.screen {
                                Screen::Welcome => {
                                    self.draw_state.focus = self.draw_state.focus.prev();
                                }
                                Screen::Main => {
                                    self.draw_state.focus_side = match self.draw_state.focus_side {
                                        FocusSide::Left => FocusSide::Right,
                                        FocusSide::Right => FocusSide::Left,
                                    };
                                }
                            }
                        }
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            if self.draw_state.screen == Screen::Main {
                                // Toggle collapse of current category
                                self.toggle_category_collapse();
                            }
                        }
                        KeyCode::Char(' ') => {
                            if self.draw_state.screen == Screen::Main {
                                // Mark item/category as done
                                self.toggle_done()?;
                            }
                        }
                        KeyCode::Up => {
                            self.handle_up()?;
                        }
                        KeyCode::Down => {
                            self.handle_down()?;
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
                        _ => {
                            self.handle_other_input(key.code)?;
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
                    Focus::TravelStyle => {
                        self.draw_state.travel_style = match self.draw_state.travel_style {
                            TravelStyle::Minimalist => TravelStyle::Backpacker,
                            TravelStyle::Standard => TravelStyle::Minimalist,
                            TravelStyle::Luxury => TravelStyle::Standard,
                            TravelStyle::Backpacker => TravelStyle::Luxury,
                        };
                        self.update_quantities()?;
                    }
                    Focus::Weather => {
                        self.draw_state.weather = match self.draw_state.weather {
                            WeatherInput::Hot => WeatherInput::Mixed,
                            WeatherInput::Cold => WeatherInput::Hot,
                            WeatherInput::Rainy => WeatherInput::Cold,
                            WeatherInput::Snowy => WeatherInput::Rainy,
                            WeatherInput::Mixed => WeatherInput::Snowy,
                        };
                        self.update_quantities()?;
                    }
                    Focus::Activities => {
                        if !self.draw_state.activities.is_empty() {
                            self.draw_state.activities.clear();
                        } else {
                            self.draw_state.activities.push(Activity::Hiking);
                        }
                        self.update_quantities()?;
                    }
                    _ => {}
                }
            }
            Screen::Main => {
                // Navigate up in categories/items
                if self.draw_state.focus_side == FocusSide::Left {
                    // Move up in categories
                    // For now, just cycle through categories
                }
            }
        }
        Ok(())
    }

    fn handle_down(&mut self) -> Result<()> {
        match self.draw_state.screen {
            Screen::Welcome => {
                match self.draw_state.focus {
                    Focus::TravelStyle => {
                        self.draw_state.travel_style = match self.draw_state.travel_style {
                            TravelStyle::Minimalist => TravelStyle::Standard,
                            TravelStyle::Standard => TravelStyle::Luxury,
                            TravelStyle::Luxury => TravelStyle::Backpacker,
                            TravelStyle::Backpacker => TravelStyle::Minimalist,
                        };
                        self.update_quantities()?;
                    }
                    Focus::Weather => {
                        self.draw_state.weather = match self.draw_state.weather {
                            WeatherInput::Hot => WeatherInput::Cold,
                            WeatherInput::Cold => WeatherInput::Rainy,
                            WeatherInput::Rainy => WeatherInput::Snowy,
                            WeatherInput::Snowy => WeatherInput::Mixed,
                            WeatherInput::Mixed => WeatherInput::Hot,
                        };
                        self.update_quantities()?;
                    }
                    Focus::Activities => {
                        if !self.draw_state.activities.is_empty() {
                            self.draw_state.activities.clear();
                        } else {
                            self.draw_state.activities.push(Activity::Hiking);
                        }
                        self.update_quantities()?;
                    }
                    _ => {}
                }
            }
            Screen::Main => {
                // Navigate down in categories/items
            }
        }
        Ok(())
    }

    fn handle_other_input(&mut self, key: KeyCode) -> Result<()> {
        // Handle other key inputs
        Ok(())
    }

    fn handle_select(&mut self) -> Result<()> {
        // Handle selection in main screen
        Ok(())
    }

    fn toggle_category_collapse(&mut self) {
        // Toggle collapse state of current category
        // For now, just a placeholder
    }

    fn toggle_done(&mut self) -> Result<()> {
        // Toggle done status of current item/category
        // For now, mark all items in first category as done/undone
        if !self.draw_state.packing_list.items.is_empty() {
            let first_category = self.draw_state.packing_list.items[0].category.clone();
            for item in &mut self.draw_state.packing_list.items {
                if item.category == first_category {
                    item.packed = !item.packed;
                }
            }
        }
        Ok(())
    }
}

fn draw_welcome_screen(f: &mut ratatui::prelude::Frame, size: Rect, state: &AppDrawState) {
    let [title_area, content_area, instructions_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(15),
            Constraint::Length(3),
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
        Line::from("  Use +/- to adjust".dim()),
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
        Line::from(format!("  Current: {:?}", state.travel_style)),
        Line::from(""),
        Line::from("  Use ↑/↓ to cycle through options".dim()),
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
        Line::from(format!("  Current: {:?}", state.weather)),
        Line::from(""),
        Line::from("  Use ↑/↓ to cycle through options".dim()),
    ];
    let weather_para = Paragraph::new(weather_text);
    f.render_widget(weather_block, params_layout[2]);
    f.render_widget(weather_para, params_layout[2]);

    // Activities
    let activities_block = Block::default()
        .title("Activities")
        .borders(Borders::ALL)
        .border_style(if state.focus == Focus::Activities {
            Style::new().fg(Color::Yellow)
        } else {
            Style::new()
        });
    
    let activities_text = vec![
        Line::from(""),
        Line::from(if state.activities.is_empty() {
            "  Current: None".to_string()
        } else {
            format!("  Current: {:?}", state.activities)
        }),
        Line::from(""),
        Line::from("  Use ↑/↓ to toggle activities".dim()),
    ];
    let activities_para = Paragraph::new(activities_text);
    f.render_widget(activities_block, params_layout[3]);
    f.render_widget(activities_para, params_layout[3]);

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

fn draw_main_screen(f: &mut ratatui::prelude::Frame, size: Rect, state: &AppDrawState) {
    // Split screen: left (categories/items), right (compiled packing list)
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

    // Draw left panel - categories and items
    draw_categories_panel(f, left_area, state);
    
    // Draw right panel - compiled packing list
    draw_packing_list_panel(f, right_area, state);
}

fn draw_categories_panel(f: &mut ratatui::prelude::Frame, area: Rect, state: &AppDrawState) {
    let [title_area, list_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(20),
        ])
        .split(area) else {
        return;
    };

    // Title
    let title = Paragraph::new("Categories & Items")
        .style(Style::new().bold().fg(Color::Green))
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, title_area);

    // Get unique categories
    let mut categories: Vec<&str> = state.packing_list.items.iter()
        .map(|item| item.category.as_str())
        .collect();
    categories.sort();
    categories.dedup();

    // Draw categories and their items
    let mut current_line = 0;
    for category in categories {
        let is_collapsed = state.collapsed_categories.contains(&category.to_string());
        
        // Category header
        let category_symbol = if is_collapsed { "▶ " } else { "▼ " };
        let category_line = Line::from(vec![
            Span::raw(category_symbol).fg(Color::Yellow),
            Span::raw(category).bold().fg(Color::Cyan),
        ]);
        
        let category_para = Paragraph::new(vec![category_line])
            .block(Block::default().borders(Borders::NONE));
        
        let category_area = Rect {
            x: list_area.x,
            y: list_area.y + current_line,
            width: list_area.width,
            height: 1,
        };
        f.render_widget(category_para, category_area);
        current_line += 1;

        // Draw items if category is not collapsed
        if !is_collapsed {
            let category_items: Vec<&Item> = state.packing_list.items.iter()
                .filter(|item| item.category == category)
                .collect();
            
            for item in category_items {
                let item_symbol = if item.packed { "✓ " } else { "○ " };
                let item_line = Line::from(vec![
                    Span::raw("  "),
                    Span::raw(item_symbol).fg(if item.packed { Color::Green } else { Color::Red }),
                    Span::raw(" "),
                    Span::raw(&item.name),
                    Span::raw(" x"),
                    Span::raw(item.quantity.to_string()).bold(),
                ]);
                
                let item_para = Paragraph::new(vec![item_line])
                    .block(Block::default().borders(Borders::NONE));
                
                let item_area = Rect {
                    x: list_area.x,
                    y: list_area.y + current_line,
                    width: list_area.width,
                    height: 1,
                };
                f.render_widget(item_para, item_area);
                current_line += 1;
            }
        }
    }
}

fn draw_packing_list_panel(f: &mut ratatui::prelude::Frame, area: Rect, state: &AppDrawState) {
    let [title_area, list_area, stats_area] = *Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(20),
            Constraint::Length(5),
        ])
        .split(area) else {
        return;
    };

    // Title
    let title = Paragraph::new("Compiled Packing List")
        .style(Style::new().bold().fg(Color::Cyan))
        .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(title, title_area);

    // Group items by category
    let mut categories: Vec<&str> = state.packing_list.items.iter()
        .map(|item| item.category.as_str())
        .collect();
    categories.sort();
    categories.dedup();

    let mut current_line = 0;
    for category in categories {
        let is_collapsed = state.collapsed_categories.contains(&category.to_string());
        
        // Category header
        let category_symbol = if is_collapsed { "▶ " } else { "▼ " };
        let category_line = Line::from(vec![
            Span::raw(category_symbol).fg(Color::Yellow),
            Span::raw(category).bold().fg(Color::Cyan),
        ]);
        
        let category_para = Paragraph::new(vec![category_line])
            .block(Block::default().borders(Borders::NONE));
        
        let category_area = Rect {
            x: list_area.x,
            y: list_area.y + current_line,
            width: list_area.width,
            height: 1,
        };
        f.render_widget(category_para, category_area);
        current_line += 1;

        // Draw items if category is not collapsed
        if !is_collapsed {
            let category_items: Vec<&Item> = state.packing_list.items.iter()
                .filter(|item| item.category == category)
                .collect();
            
            for item in category_items {
                let item_symbol = if item.packed { "✓ " } else { "○ " };
                let item_line = Line::from(vec![
                    Span::raw("  "),
                    Span::raw(item_symbol).fg(if item.packed { Color::Green } else { Color::Red }),
                    Span::raw(" "),
                    Span::raw(&item.name),
                    Span::raw(" x"),
                    Span::raw(item.quantity.to_string()).bold(),
                ]);
                
                let item_para = Paragraph::new(vec![item_line])
                    .block(Block::default().borders(Borders::NONE));
                
                let item_area = Rect {
                    x: list_area.x,
                    y: list_area.y + current_line,
                    width: list_area.width,
                    height: 1,
                };
                f.render_widget(item_para, item_area);
                current_line += 1;
            }
        }
    }

    // Stats
    let total = state.packing_list.total_items();
    let packed = state.packing_list.packed_items();
    let unpacked = state.packing_list.unpacked_items();
    
    let stats = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Total Items: ").bold(),
            Span::raw(total.to_string()).fg(Color::White),
        ]),
        Line::from(vec![
            Span::raw("Packed: ").bold().fg(Color::Green),
            Span::raw(packed.to_string()).fg(Color::Green),
            Span::raw(" | "),
            Span::raw("Unpacked: ").bold().fg(Color::Red),
            Span::raw(unpacked.to_string()).fg(Color::Red),
        ]),
        Line::from(vec![
            Span::raw("Progress: "),
            Span::raw(format!("{:.0}%", if total > 0 { (packed as f32 / total as f32 * 100.0) } else { 0.0 })).bold(),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Tab").bold().fg(Color::Yellow),
            Span::raw(" to switch panels"),
        ]),
        Line::from(vec![
            Span::raw("C").bold().fg(Color::Yellow),
            Span::raw(" to collapse/expand category"),
        ]),
        Line::from(vec![
            Span::raw("Space").bold().fg(Color::Yellow),
            Span::raw(" to mark as done"),
        ]),
    ])
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(stats, stats_area);
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = stdout().execute(LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}
