use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Constraint, Layout},
    widgets::{Block, Paragraph},
};

use crate::screens::{
    login::LoginScreen,
    register::RegisterScreen,
    screen::{HIGHLIGHT_COLOR, STANDARD_COLOR, Screen, draw_screen_border},
};

#[derive(Default)]
pub struct HomeScreen {
    selected: u8,
}

impl Screen for HomeScreen {
    fn handle_input(
        &mut self,
        key: (KeyCode, KeyModifiers),
    ) -> Option<Box<dyn Screen + Sync + Send>> {
        match key {
            (KeyCode::Enter, _) => return self.submit(),
            (KeyCode::Tab, _) | (KeyCode::Down, _) => self.focus_next(),
            (KeyCode::BackTab, KeyModifiers::SHIFT) | (KeyCode::Up, _) => self.focus_prev(),
            _ => (),
        };
        None
    }
    fn render(&mut self, f: &mut ratatui::Frame) {
        let area = draw_screen_border(
            f,
            "HOME",
            "QUIT: <CTRL+Q> - NAVIGATE: <UP|DOWN|TAB> - SELECT: <ENTER>",
            None,
            None,
        );
        let [_, col, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(2),
            Constraint::Fill(1),
        ])
        .areas(area);
        let [_, login, register, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .areas(col);

        let color = match self.selected {
            0 => HIGHLIGHT_COLOR,
            _ => STANDARD_COLOR,
        };
        f.render_widget(
            Paragraph::new("LOGIN")
                .block(Block::bordered())
                .style(color),
            login,
        );
        let color = match self.selected {
            1 => HIGHLIGHT_COLOR,
            _ => STANDARD_COLOR,
        };
        f.render_widget(
            Paragraph::new("REGISTER")
                .block(Block::bordered())
                .style(color),
            register,
        );
    }
}

impl HomeScreen {
    fn submit(&mut self) -> Option<Box<dyn Screen + Sync + Send>> {
        match self.selected {
            0 => Some(Box::new(LoginScreen::default())),
            1 => Some(Box::new(RegisterScreen::default())),
            _ => None,
        }
    }

    fn focus_next(&mut self) {
        self.selected += 1;
        self.selected = 1.min(self.selected);
    }

    fn focus_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }
}
