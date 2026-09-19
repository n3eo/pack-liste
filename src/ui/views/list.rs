use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct ListView;

impl ListView {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for ListView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("Packing List View")
            .borders(Borders::ALL);
        let paragraph = Paragraph::new("Packing list view with items and quantities")
            .block(block);
        paragraph.render(area, buf);
    }
}
