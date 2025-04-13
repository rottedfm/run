use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Paragraph, Widget},
};

use crate::app::App;

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let outer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(42), // unused / top
                Constraint::Length(3),      // input box
                Constraint::Percentage(50), // middle: results
                Constraint::Length(3),      // status line
            ])
            .split(area);

        // Input box
        let input_paragraph = Paragraph::new(self.user_input.as_str())
            .block(
                ratatui::widgets::Block::default()
                    .title("捜索")
                    .title_alignment(Alignment::Left)
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded),
            )
            .alignment(Alignment::Left);
        input_paragraph.render(outer_layout[1], buf);

        // Results (center column, left-aligned, no block)
        let results_text = self
            .results
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");

        let results_paragraph = Paragraph::new(results_text).alignment(Alignment::Center); // no block, just plain text
        results_paragraph.render(outer_layout[2], buf); // middle column

        // Status message
        let status_paragraph = Paragraph::new(self.status_message.as_str())
            .block(
                ratatui::widgets::Block::default()
                    .title("Status")
                    .title_alignment(Alignment::Left)
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded),
            )
            .alignment(Alignment::Left)
            .style(Style::default().dim()); // subtle look
        status_paragraph.render(outer_layout[3], buf);
    }
}
