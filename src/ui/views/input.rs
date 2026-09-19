use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Widget, List, ListItem},
};

use crate::models::activity::Activity;
use crate::models::travel_style::TravelStyle;
use crate::models::weather::WeatherInput;
use crate::models::weekdays::Weekdays;

pub struct InputView {
    weekdays: Weekdays,
    travel_style: TravelStyle,
    weather: WeatherInput,
    activities: Vec<Activity>,
}

impl InputView {
    pub fn new() -> Self {
        Self {
            weekdays: Weekdays::default(),
            travel_style: TravelStyle::default(),
            weather: WeatherInput::default(),
            activities: Vec::new(),
        }
    }

    pub fn with_weekdays(mut self, weekdays: Weekdays) -> Self {
        self.weekdays = weekdays;
        self
    }

    pub fn with_travel_style(mut self, style: TravelStyle) -> Self {
        self.travel_style = style;
        self
    }

    pub fn with_weather(mut self, weather: WeatherInput) -> Self {
        self.weather = weather;
        self
    }

    pub fn with_activities(mut self, activities: Vec<Activity>) -> Self {
        self.activities = activities;
        self
    }
}

impl Default for InputView {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for InputView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("Input View")
            .borders(Borders::ALL);
        
        let items = vec![
            ListItem::new(format!("Weekdays: {}", self.weekdays.get())),
            ListItem::new(format!("Travel Style: {:?}", self.travel_style)),
            ListItem::new(format!("Weather: {:?}", self.weather)),
            ListItem::new(format!("Activities: {:?}", self.activities)),
        ];
        
        let list = List::new(items)
            .block(block);
        
        ratatui::widgets::StatefulWidget::render(list, area, buf, &mut ratatui::widgets::ListState::default());
    }
}
