use std::{error::Error, option::Option};

use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Constraint, Layout, Margin, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState},
};

use crate::{
    conf::Conf,
    database::{self, Flag},
    screen::{Screen, draw_screen_border},
};

#[derive(PartialEq)]
enum AdminScreenState {
    Browse,
    Submit,
}

pub struct AdminScreen {
    state: AdminScreenState,
    flags: Vec<Flag>,
    error: Option<String>,
    table_state: TableState,
    scroll: u16,
    submission: String,
    filter: SearchFilter,
    conf: Conf,
}

impl Screen for AdminScreen {
    fn handle_input(
        &mut self,
        key: Option<(KeyCode, KeyModifiers)>,
    ) -> Option<Box<dyn Screen + Send>> {
        // if no key is pressed, return early for now
        let key = key?;
        None
    }
    fn render(&mut self, f: &mut Frame) {
        let commands = match self.state {
            AdminScreenState::Browse => {
                if self.filter.ui_active {
                    "^Q[QUIT] Esc[BACK] ⇥[KIND] ⇄[VALUE] ↵[APPLY]"
                } else {
                    "^Q[QUIT] ⇵[NAV] ↵[SELECT] ^R[RELOAD] ^⇄[TABS] ^F[FILTER]"
                }
            }
            AdminScreenState::Submit => "^Q[QUIT] Esc[BACK] ⇵[SCROLL] ↵[SUBMIT] ^R[RELOAD]",
        };
        let area = draw_screen_border(
            f,
            if self.conf.about.is_some() {
                vec!["FLAGS", "LEADERBOARD", "ABOUT"]
            } else {
                vec!["FLAGS", "LEADERBOARD"]
            },
            0,
            commands,
            self.error.as_deref(),
            None,
            &self.conf,
        );
    }
}

impl AdminScreen {
    fn submit(&mut self) -> Option<Box<dyn Screen + Send>> {
        None
    }

    pub fn new(conf: Conf) -> Self {
        let mut error = None;
        let flags = match database::Flag::get_all() {
            Ok(f) => f,
            Err(e) => {
                error = Some(e.to_string());
                vec![]
            }
        };
        Self {
            state: AdminScreenState::Browse,
            table_state: TableState::default().with_selected(0),
            flags,
            error,
            scroll: 0,
            filter: SearchFilter::new(),
            submission: String::new(),
            conf,
        }
    }

    fn focus_next(&mut self) {
        match self.state {
            AdminScreenState::Browse => self.table_state.select_next(),
            AdminScreenState::Submit => self.scroll = self.scroll.saturating_add(1),
        }
    }

    fn focus_prev(&mut self) {
        match self.state {
            AdminScreenState::Browse => self.table_state.select_previous(),
            AdminScreenState::Submit => self.scroll = self.scroll.saturating_sub(1),
        }
    }

    fn escape(&mut self) -> Option<Box<dyn Screen + Send>> {
        match self.state {
            AdminScreenState::Submit => {
                self.scroll = 0;
                self.submission.clear();
                self.state = AdminScreenState::Browse;
                return None;
            }
            _ => None,
        }
    }

    fn erase(&mut self) {
        match self.state {
            AdminScreenState::Browse => (),
            AdminScreenState::Submit => {
                let _ = self.submission.pop();
            }
        }
    }

    fn write_char(&mut self, c: char) {
        match self.state {
            AdminScreenState::Browse => (),
            AdminScreenState::Submit => self.submission.push(c),
        }
    }

    fn draw_table(&mut self, f: &mut Frame, a: Rect) {
        let header = ["Name", "Description", "Points", "Solved"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .fg(self.conf.theme.base08)
            .bg(self.conf.theme.base00)
            .italic()
            .bold()
            .height(1);

        let rows = self.flags.iter().enumerate().map(|(i, f)| {
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

        let table = Table::new(
            rows,
            [
                Constraint::Fill(1),
                Constraint::Fill(2),
                Constraint::Fill(1),
                // seven for 'Solved' +1
                Constraint::Length(7),
            ],
        )
        .header(header)
        .row_highlight_style(
            Style::new()
                .fg(self.conf.theme.base05)
                .bg(self.conf.theme.base03),
        )
        .highlight_symbol(" >")
        .block(
            Block::new().borders(Borders::RIGHT).border_style(
                Style::new()
                    .fg(self.conf.theme.base01)
                    .bg(self.conf.theme.base00),
            ),
        );

        f.render_stateful_widget(table, a, &mut self.table_state);
    }

    fn draw_preview(&self, f: &mut Frame<'_>, a: Rect) -> Result<(), Box<dyn Error>> {
        let flag = self
            .flags
            .get(
                self.table_state
                    .selected()
                    .ok_or("failed to get selected flag")?,
            )
            .ok_or("failed to get selected flag")?;
        let [header, description, submission] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .areas(a);

        let style1 = match self.state {
            AdminScreenState::Browse => Style::new()
                .fg(self.conf.theme.base04)
                .bg(self.conf.theme.base00),
            AdminScreenState::Submit => Style::new()
                .fg(self.conf.theme.base05)
                .bg(self.conf.theme.base00),
        };

        let style2 = match self.state {
            AdminScreenState::Browse => Style::new()
                .fg(self.conf.theme.base04)
                .bg(self.conf.theme.base00),
            AdminScreenState::Submit => Style::new()
                .fg(self.conf.theme.base08)
                .bg(self.conf.theme.base00),
        };

        let title = Paragraph::new(format!(
            "{}\nPoints - {}{}",
            flag.name(),
            flag.points(),
            if flag.solved() { " - SOLVED" } else { "" }
        ))
        .style(style2)
        .bold()
        .italic()
        .centered()
        .block(Block::new().borders(Borders::BOTTOM).border_style(style1));
        f.render_widget(title, header);

        let description_text = Paragraph::new(flag.description())
            .wrap(ratatui::widgets::Wrap { trim: false })
            .scroll((self.scroll, 0))
            .style(style1);

        f.render_widget(description_text, description);

        let input_box = Paragraph::new(self.submission.as_str())
            .block(Block::bordered().title_top("Submit Flag").style(style2));

        f.render_widget(input_box, submission);

        Ok(())
    }
}

struct SearchFilter {
    search_kind: SearchFilterKind,
    ui_active: bool,
}

impl SearchFilter {
    fn new() -> Self {
        Self {
            search_kind: SearchFilterKind::None,
            ui_active: false,
        }
    }

    fn handle_input(&mut self, key: (KeyCode, KeyModifiers)) -> bool {
        match key {
            (KeyCode::Enter, _) => {
                self.ui_active = false;
                return true;
            }
            (KeyCode::Esc, _) => {
                self.ui_active = false;
            }
            (KeyCode::BackTab, _) => match self.search_kind {
                SearchFilterKind::None => self.search_kind = SearchFilterKind::Solved(false),
                SearchFilterKind::Solved(_) => {
                    self.search_kind = SearchFilterKind::Search(String::new())
                }
                SearchFilterKind::Search(_) => self.search_kind = SearchFilterKind::None,
            },
            (KeyCode::Tab, _) => match self.search_kind {
                SearchFilterKind::None => {
                    self.search_kind = SearchFilterKind::Search(String::new())
                }
                SearchFilterKind::Search(_) => self.search_kind = SearchFilterKind::Solved(false),

                SearchFilterKind::Solved(_) => self.search_kind = SearchFilterKind::None,
            },
            (KeyCode::Char(c), _) => match self.search_kind {
                SearchFilterKind::Search(ref mut string) => string.push(c),
                _ => (),
            },
            (KeyCode::Backspace, _) => match self.search_kind {
                SearchFilterKind::Search(ref mut string) => {
                    string.pop();
                }
                _ => (),
            },
            (KeyCode::Left | KeyCode::Right, _) => match self.search_kind {
                SearchFilterKind::Solved(ref mut toggle) => {
                    *toggle = !*toggle;
                }
                _ => (),
            },
            _ => (),
        }
        false
    }

    fn render(&self, f: &mut Frame, area: Rect, conf: &Conf) {
        let [_, col, _] = Layout::horizontal([Constraint::Fill(1); 3]).areas(area);
        let [_, row, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(8),
            Constraint::Fill(1),
        ])
        .areas(col);
        f.render_widget(Clear, row);
        f.render_widget(
            Block::bordered()
                .title("Filter")
                .fg(conf.theme.base05)
                .bg(conf.theme.base01),
            row,
        );
        let area = row.inner(Margin::new(1, 1));
        let [_, kind, value, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .areas(area);
        match self.search_kind {
            SearchFilterKind::None => {
                f.render_widget(
                    Paragraph::new("None").block(Block::bordered().title("Kind")),
                    kind,
                );
            }
            SearchFilterKind::Solved(b) => {
                f.render_widget(
                    Paragraph::new("Solved").block(Block::bordered().title("Kind")),
                    kind,
                );
                f.render_widget(
                    Paragraph::new(format!("{}", b).as_str())
                        .block(Block::bordered().title("value"))
                        .fg(conf.theme.base08),
                    value,
                );
            }
            SearchFilterKind::Search(ref s) => {
                f.render_widget(
                    Paragraph::new("Search").block(Block::bordered().title("Kind")),
                    kind,
                );
                f.render_widget(
                    Paragraph::new(format!("{}", s).as_str())
                        .block(Block::bordered().title("value"))
                        .fg(conf.theme.base08),
                    value,
                );
            }
        }
    }
}

enum SearchFilterKind {
    None,
    Solved(bool),
    Search(String),
}
