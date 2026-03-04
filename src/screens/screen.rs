use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Margin, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::Block;

const BG_HEX: u32 = 0x2D2D2A;
const FG_HEX: u32 = 0x3F5E5A;
const HL_HEX: u32 = 0x20FC8F;

pub const STANDARD_COLOR: Style = Style::new()
    .bg(Color::from_u32(BG_HEX))
    .fg(Color::from_u32(FG_HEX));
pub const HIGHLIGHT_COLOR: Style = Style::new()
    .bg(Color::from_u32(BG_HEX))
    .fg(Color::from_u32(HL_HEX));

pub fn draw_screen_border(f: &mut Frame, title: &'static str) -> Rect {
    f.render_widget(
        Block::bordered().title_top(title).style(HIGHLIGHT_COLOR),
        f.area(),
    );
    f.area().inner(Margin::new(1, 1))
}

pub trait Screen {
    fn handle_input(&mut self, key: (KeyCode, KeyModifiers)) -> Option<Box<dyn Screen>>;
    fn render(&mut self, f: &mut Frame);
}

#[derive(Clone, Copy, Default)]
pub enum ScreenType {
    #[default]
    Register,
    Login,
}
