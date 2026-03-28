use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Constraint, Rect},
    style::Stylize,
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};

use crate::{
    conf::Conf,
    database::User,
    screens::{
        flags::BrowseScreen,
        home::HomeScreen,
        screen::{Screen, draw_screen_border},
    },
};

pub struct LeaderboardScreen {
    user: Option<User>,
    users: Vec<User>,
    error: Option<String>,
    conf: Conf,
    leaderboard: TableState,
}

impl Screen for LeaderboardScreen {
    fn handle_input(
        &mut self,
        key: (KeyCode, KeyModifiers),
    ) -> Option<Box<dyn Screen + Sync + Send>> {
        self.error = None;
        match key {
            (KeyCode::Tab, _) | (KeyCode::Down, _) => self.focus_next(),
            (KeyCode::BackTab, KeyModifiers::SHIFT) | (KeyCode::Up, _) => self.focus_prev(),
            (KeyCode::Char('r'), KeyModifiers::CONTROL) => return self.reload(),
            (KeyCode::Left, KeyModifiers::CONTROL) => {
                return match &self.user {
                    Some(u) => Some(Box::new(BrowseScreen::new(u.clone(), self.conf.clone()))),
                    None => None, // TODO: make this an error
                };
            }
            _ => (),
        };
        None
    }
    fn render(&mut self, f: &mut Frame) {
        let commands = "QUIT<CTRL+Q> NAV<UP|DOWN|TAB> RLOAD<CTRL+R> NAV TABS<CTRL+LEFT|RIGHT>";
        let area = draw_screen_border(
            f,
            vec!["FLAGS", "LEADERBOARD"],
            1,
            commands,
            self.error.as_deref(),
            self.user.as_ref(),
            &self.conf,
        );

        self.draw_leaderboard(f, area);
    }
}

impl LeaderboardScreen {
    pub fn new(user: Option<User>, conf: Conf) -> Self {
        let mut users = User::get_all().unwrap_or_default();
        users.sort_by_key(|x| -x.points());

        Self {
            user,
            conf,
            users,
            error: None,
            leaderboard: TableState::new(),
        }
    }

    fn focus_next(&mut self) {
        self.leaderboard = self
            .leaderboard
            .with_offset(self.leaderboard.offset().saturating_add(1));
    }

    fn focus_prev(&mut self) {
        self.leaderboard = self
            .leaderboard
            .with_offset(self.leaderboard.offset().saturating_sub(1));
    }

    fn draw_leaderboard(&mut self, f: &mut Frame, a: Rect) {
        let header = ["Name", "Points"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .fg(self.conf.theme.base08)
            .bg(self.conf.theme.base00)
            .italic()
            .bold()
            .height(1);

        let rows = self.users.iter().enumerate().map(|(i, f)| {
            f.row_parts()
                .into_iter()
                .map(Cell::from)
                .collect::<Row>()
                .fg(self.conf.theme.base05)
                .bg(if i % 2 == 1 {
                    self.conf.theme.base00
                } else {
                    self.conf.theme.base01
                })
                .height(1)
        });

        let table = Table::new(rows, [Constraint::Fill(1), Constraint::Fill(1)])
            .header(header)
            .block(Block::new().borders(Borders::RIGHT));

        f.render_stateful_widget(table, a, &mut self.leaderboard);
    }

    fn reload(&mut self) -> Option<Box<dyn Screen + Sync + Send>> {
        self.users = User::get_all().ok()?;
        self.users.sort_by_key(|x| -x.points());
        None
    }
}
