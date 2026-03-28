use std::time::{Duration, Instant};

use ratatui::{
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::Text,
    widgets::{Block, Paragraph},
};

use crate::{conf::Conf, screens::flags::BrowseScreen};
use crate::{
    database::User,
    screens::{
        register::RegisterScreen,
        screen::{Screen, draw_screen_border},
    },
};

pub struct HomeScreen {
    conf: Conf,
    time: Instant,
    key: russh::keys::PublicKey,
}

impl Screen for HomeScreen {
    fn handle_input(
        &mut self,
        _key: (KeyCode, KeyModifiers),
    ) -> Option<Box<dyn Screen + Sync + Send>> {
        if Instant::now() - self.time > Duration::from_secs(3) {
            match User::login(self.key.clone()) {
                Some(u) => Some(Box::new(BrowseScreen::new(u, self.conf.clone()))),
                None => Some(Box::new(RegisterScreen::new(
                    self.conf.clone(),
                    self.key.clone(),
                ))),
            }
        } else {
            None
        }
    }
    fn render(&mut self, f: &mut ratatui::Frame) {
        let area = draw_screen_border(
            f,
            vec!["HOME"],
            0,
            "WELCOME TO SSHACK",
            None,
            None,
            &self.conf,
        );

        let [_, banner_part, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Max(self.conf.banner.lines().count() as u16 + 1),
            Constraint::Fill(1),
        ])
        .areas(area);

        let [_, banner, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(
                self.conf
                    .banner
                    .lines()
                    .map(|x| x.chars().count())
                    .max()
                    .unwrap_or(0) as u16,
            ),
            Constraint::Fill(1),
        ])
        .areas(banner_part);
        f.render_widget(
            Paragraph::new(Text::raw(&self.conf.banner))
                .fg(self.conf.theme.base08)
                .bg(self.conf.theme.base00),
            banner,
        );
    }
}

impl HomeScreen {
    pub fn new(conf: Conf, key: russh::keys::PublicKey) -> Self {
        Self {
            time: Instant::now(),
            conf,
            key,
        }
    }
}
