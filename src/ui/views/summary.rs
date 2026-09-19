use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct SummaryView;

impl SummaryView {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for SummaryView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title("Summary View")
            .borders(Borders::ALL);
        let paragraph = Paragraph::new("Summary view with packing progress")
            .block(block);
        paragraph.render(area, buf);
    }
}
