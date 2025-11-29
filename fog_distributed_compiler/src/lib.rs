use ratatui::{layout::Rect, style::Style, text::Text, widgets::Widget};

#[derive(Debug, Clone, PartialEq)]
pub enum UiState
{
    Main,
    ConnectionEstablisher,
    CurrentConnection,
}

#[derive(Debug, Clone)]
pub struct TextField {
    pub placeholder: String,
    pub inner_text: String,
    pub current_style: Style,
    pub original_style: Style,
    pub highlighted_style: Style,
}

impl TextField {
    pub fn new(placeholder: &str, original_style: Style, highlighted_style: Style) -> Self {
        Self { placeholder: placeholder.to_string(), inner_text: String::new(), original_style, highlighted_style, current_style: original_style }
    }

    pub fn should_highlight(&mut self, should_highlight: bool) {
        self.current_style = if should_highlight {
            self.highlighted_style
        }
        else {
            self.original_style
        };
    }
}

impl Widget for TextField {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized {
        let text = if self.inner_text.is_empty() {
            Text::raw(self.placeholder)
        }
        else {
            Text::raw(self.inner_text)
        }.style(self.current_style);

        text.clone().render(Rect::new(area.x, area.y, 30, 1), buf);
    }
}