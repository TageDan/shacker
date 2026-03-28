use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Constraint, Layout},
    style::Style,
    widgets::{Block, Paragraph},
};

use crate::{
    conf::Conf,
    database,
    screens::{
        flags::BrowseScreen,
        home::HomeScreen,
        screen::{Screen, draw_screen_border},
    },
};

pub struct RegisterScreen {
    username: String,
    key: russh::keys::PublicKey,
    error: Option<String>,
    conf: Conf,
}

impl Screen for RegisterScreen {
    fn handle_input(
        &mut self,
        key: (KeyCode, KeyModifiers),
    ) -> Option<Box<dyn Screen + Send + Sync>> {
        // Remove error on input
        self.error = None;
        match key {
            (KeyCode::Enter, _) => return self.submit(),
            (KeyCode::Char(c), _) => self.write_char(c),
            (KeyCode::Esc, _) => {
                return Some(Box::new(HomeScreen::new(
                    self.conf.clone(),
                    self.key.clone(),
                )));
            }
            (KeyCode::Backspace, _) => self.delete(),
            _ => (),
        };
        None
    }
    fn render(&mut self, f: &mut ratatui::Frame) {
        let area = draw_screen_border(
            f,
            vec!["REGISTER"],
            0,
            "QUIT<CTRL+Q> SUBMIT<ENTER>",
            self.error.as_deref(),
            None,
            &self.conf,
        );

        let [_, col, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(2),
            Constraint::Fill(1),
        ])
        .areas(area);
        let [_, user, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .areas(col);

        let color = Style::new()
            .fg(self.conf.theme.base08)
            .bg(self.conf.theme.base00);

        f.render_widget(
            Paragraph::new(self.username.as_str())
                .block(Block::bordered().title_top("USERNAME"))
                .style(color),
            user,
        );
    }
}

impl RegisterScreen {
    pub fn new(conf: Conf, key: russh::keys::PublicKey) -> Self {
        Self {
            username: String::new(),
            key,
            error: None,
            conf,
        }
    }

    fn submit(&mut self) -> Option<Box<dyn Screen + Send + Sync>> {
        if self.username.is_empty() {
            self.error = Some("username can not be empty".to_string());
            return None;
        }
        match database::User::register_user(&self.username, self.key.clone()) {
            Err(e) => {
                self.error = Some(e.to_string());
                return None;
            }
            Ok(u) => {
                return Some(Box::new(BrowseScreen::new(u, self.conf.clone())));
            }
        }
    }

    fn write_char(&mut self, c: char) {
        self.username.push(c);
    }

    fn delete(&mut self) {
        self.username.pop();
    }
}
