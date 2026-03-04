use ratatui::{
    crossterm::{
        event::{KeyCode, KeyModifiers},
        style::Color,
    },
    layout::{Constraint, Layout, Margin},
    style::Style,
    widgets::{Block, Paragraph},
};

use crate::screens::{
    home::HomeScreen,
    screen::{HIGHLIGHT_COLOR, STANDARD_COLOR, Screen, draw_screen_border},
};

#[derive(Default)]
pub struct LoginScreen {
    username: String,
    password: String,
    selected: u8,
}

impl Screen for LoginScreen {
    fn handle_input(&mut self, key: (KeyCode, KeyModifiers)) -> Option<Box<dyn Screen>> {
        match key {
            (KeyCode::Enter, _) => return self.submit(),
            (KeyCode::Char(c), _) => self.write_char(c),

            (KeyCode::Tab, _) | (KeyCode::Down, _) => self.focus_next(),
            (KeyCode::BackTab, KeyModifiers::SHIFT) | (KeyCode::Up, _) => self.focus_prev(),
            (KeyCode::Backspace, _) => self.delete(),
            (KeyCode::Esc, _) => return Some(Box::new(HomeScreen::default())),
            _ => (),
        };
        None
    }
    fn render(&mut self, f: &mut ratatui::Frame) {
        let area = draw_screen_border(f, "LOGIN");
        let [_, col, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(2),
            Constraint::Fill(1),
        ])
        .areas(area);
        let [_, user, pass, _] = Layout::vertical([
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
            Paragraph::new(self.username.as_str())
                .block(Block::bordered().title_top("USERNAME"))
                .style(color),
            user,
        );
        let color = match self.selected {
            1 => HIGHLIGHT_COLOR,
            _ => STANDARD_COLOR,
        };
        f.render_widget(
            Paragraph::new("*".repeat(self.password.len()))
                .block(Block::bordered().title_top("PASSWORD"))
                .style(color),
            pass,
        );
    }
}

impl LoginScreen {
    fn submit(&mut self) -> Option<Box<dyn Screen>> {
        self.focus_next();
        // TODO: check if password and confirm matches and
        // interact with a database in order to add a user/find a collision.
        // Also figure out a good way to show an error message.
        None
    }

    fn write_char(&mut self, c: char) {
        match self.selected {
            0 => self.username.push(c),
            1 => self.password.push(c),
            _ => (),
        }
    }

    fn focus_next(&mut self) {
        self.selected += 1;
        self.selected = 1.min(self.selected);
    }

    fn focus_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn delete(&mut self) {
        match self.selected {
            0 => {
                self.username.pop();
            }
            1 => {
                self.password.pop();
            }
            _ => (),
        }
    }
}
