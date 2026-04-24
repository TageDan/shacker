use crate::{
    app::screens::{flags::BrowseScreen, home::HomeScreen},
    conf::Conf,
    database,
    screen::{Screen, draw_screen_border},
};
use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Constraint, Layout},
    style::Style,
    widgets::{Block, Paragraph},
};

pub struct RegisterScreen {
    username: String,
    password: String,
    selected: u8,
    key: russh::keys::PublicKey,
    error: Option<String>,
    conf: Conf,
}

impl Screen for RegisterScreen {
    fn handle_input(
        &mut self,
        key: Option<(KeyCode, KeyModifiers)>,
    ) -> Option<Box<dyn Screen + Send>> {
        // if no key is pressed, return early for now
        let key = key?;
        // Remove error on input
        self.error = None;
        match key {
            (KeyCode::Enter, _) => return self.submit(),
            (KeyCode::Char(c), _) => self.write_char(c),
            (KeyCode::Up, _) | (KeyCode::BackTab, KeyModifiers::SHIFT) => self.prev(),
            (KeyCode::Down, _) | (KeyCode::Tab, _) => self.next(),
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
            if self.conf.password.is_some() {
                "^Q[QUIT] ↵[SUBMIT]"
            } else {
                "^Q[QUIT] ↵[SUBMIT] ⇵[NAV]"
            },
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

        if self.conf.password.is_none() {
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
        } else {
            let [_, user, pass, _] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Fill(1),
            ])
            .areas(col);

            let color1 = Style::new()
                .fg(self.conf.theme.base08)
                .bg(self.conf.theme.base00);
            let color2 = Style::new()
                .fg(self.conf.theme.base05)
                .bg(self.conf.theme.base00);

            f.render_widget(
                Paragraph::new(self.username.as_str())
                    .block(Block::bordered().title_top("USERNAME"))
                    .style(if self.selected == 0 { color1 } else { color2 }),
                user,
            );
            f.render_widget(
                Paragraph::new("*".repeat(self.password.len()))
                    .block(Block::bordered().title_top("PASSWORD"))
                    .style(if self.selected == 1 { color1 } else { color2 }),
                pass,
            );
        }
    }
}

impl RegisterScreen {
    pub fn new(conf: Conf, key: russh::keys::PublicKey) -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            selected: 0,
            key,
            error: None,
            conf,
        }
    }

    fn submit(&mut self) -> Option<Box<dyn Screen + Send>> {
        if self.conf.password.is_some() && self.selected == 0 {
            self.selected = 1;
            return None;
        }
        if self.username.is_empty() {
            self.error = Some("username can not be empty".to_string());
            return None;
        }
        if self
            .conf
            .password
            .as_ref()
            .is_some_and(|x| x != &self.password)
        {
            self.error = Some("Wrong password".to_string());
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
        match self.selected {
            0 => self.username.push(c),
            1 => self.password.push(c),
            _ => (),
        };
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

    fn next(&mut self) {
        if self.conf.password.is_some() {
            self.selected = 1.min(self.selected + 1);
        }
    }

    fn prev(&mut self) {
        if self.conf.password.is_some() {
            self.selected = self.selected.saturating_sub(1);
        }
    }
}
