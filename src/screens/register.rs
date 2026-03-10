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
        browse::BrowseScreen,
        home::HomeScreen,
        screen::{Screen, draw_screen_border},
    },
};

pub struct RegisterScreen {
    username: String,
    password: String,
    confirm: String,
    selected: u8,
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
            (KeyCode::Tab, _) | (KeyCode::Down, _) => self.focus_next(),
            (KeyCode::BackTab, KeyModifiers::SHIFT) | (KeyCode::Up, _) => self.focus_prev(),
            (KeyCode::Esc, _) => return Some(Box::new(HomeScreen::new(self.conf.clone()))),
            (KeyCode::Backspace, _) => self.delete(),
            _ => (),
        };
        None
    }
    fn render(&mut self, f: &mut ratatui::Frame) {
        let area = draw_screen_border(
            f,
            "REGISTER",
            "QUIT: <CTRL+Q> - NAVIGATE: <UP|DOWN|TAB> - GO BACK: <ESC> - SUBMIT: <ENTER>",
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
        let [_, user, pass, conf, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .areas(col);

        let color = match self.selected {
            0 => Style::new()
                .fg(self.conf.theme.base08)
                .bg(self.conf.theme.base00),
            _ => Style::new()
                .fg(self.conf.theme.base07)
                .bg(self.conf.theme.base00),
        };
        f.render_widget(
            Paragraph::new(self.username.as_str())
                .block(Block::bordered().title_top("USERNAME"))
                .style(color),
            user,
        );

        let color = match self.selected {
            1 => Style::new()
                .fg(self.conf.theme.base08)
                .bg(self.conf.theme.base00),
            _ => Style::new()
                .fg(self.conf.theme.base07)
                .bg(self.conf.theme.base00),
        };
        f.render_widget(
            Paragraph::new("*".repeat(self.password.len()))
                .block(Block::bordered().title_top("PASSWORD"))
                .style(color),
            pass,
        );

        let color = match self.selected {
            2 => Style::new()
                .fg(self.conf.theme.base08)
                .bg(self.conf.theme.base00),
            _ => Style::new()
                .fg(self.conf.theme.base07)
                .bg(self.conf.theme.base00),
        };
        f.render_widget(
            Paragraph::new("*".repeat(self.confirm.len()))
                .block(Block::bordered().title_top("CONFIRM_PASSWORD"))
                .style(color),
            conf,
        );
    }
}

impl RegisterScreen {
    pub fn new(conf: Conf) -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            confirm: String::new(),
            selected: 0,
            error: None,
            conf,
        }
    }

    fn submit(&mut self) -> Option<Box<dyn Screen + Send + Sync>> {
        if self.selected == 2 {
            if self.password != self.confirm {
                self.error = Some("passwords doesn't match".to_string());
                return None;
            }
            if self.username.is_empty() || self.password.is_empty() {
                self.error = Some("password can not be empty".to_string());
                return None;
            }
            match database::User::register_user(&self.username, &self.password) {
                Err(e) => {
                    self.error = Some(e.to_string());
                    return None;
                }
                Ok(u) => {
                    return Some(Box::new(BrowseScreen::new(u, self.conf.clone())));
                }
            }
        }
        self.focus_next();
        None
    }

    fn write_char(&mut self, c: char) {
        match self.selected {
            0 => self.username.push(c),
            1 => self.password.push(c),
            2 => self.confirm.push(c),
            _ => (),
        }
    }

    fn focus_next(&mut self) {
        self.selected += 1;
        self.selected = 2.min(self.selected);
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
            2 => {
                self.confirm.pop();
            }
            _ => (),
        }
    }
}
