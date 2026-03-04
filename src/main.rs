mod users;
use std::{arch::x86_64::_SIDD_NEGATIVE_POLARITY, error::Error, process::exit};

use ratatui::{
    DefaultTerminal, Frame, Viewport,
    crossterm::{
        self,
        event::{KeyCode, KeyEvent, KeyModifiers},
    },
    layout::{Constraint, Layout},
    widgets::{Block, Paragraph},
};

mod screens;

fn main() -> Result<(), Box<dyn Error>> {
    users::create_missing_db();
    ratatui::run(app)?;
    ratatui::restore();
    Ok(())
}

fn app(terminal: &mut DefaultTerminal) -> Result<(), Box<dyn Error>> {
    let mut app = App {
        screen: Box::new(screens::home::HomeScreen::default()),
    };
    loop {
        terminal.draw(|f| app.render(f))?;
        if let Some(k) = crossterm::event::read()?.as_key_press_event() {
            match (k.code, k.modifiers) {
                (KeyCode::Char('q'), KeyModifiers::CONTROL) => return Ok(()),
                k => {
                    if let Some(t) = app.screen.handle_input(k) {
                        app.screen = t;
                    }
                }
            }
        }
    }
}

struct App {
    screen: Box<dyn screens::screen::Screen>,
}

impl App {
    fn render(&mut self, f: &mut Frame) {
        self.screen.render(f)
    }
}
