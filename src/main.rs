mod cli;
mod conf;
mod database;
mod screens;
mod server;
mod theme;
use std::error::Error;

use clap::Parser;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{
        self,
        event::{KeyCode, KeyModifiers},
    },
};

use crate::{
    cli::{Args, Commands},
    conf::Conf,
    screens::home::HomeScreen,
};

// inspired by: https://github.com/Eugeny/russh/blob/main/russh/examples/ratatui_app.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    database::create_missing_db();
    let args = Args::parse();
    match args.command {
        Commands::Run { local } => {
            let conf = Conf::get();
            if !local {
                let mut server = server::AppServer::new(conf);
                server.run().await.expect("Failed running server");
            } else {
                let mut term = ratatui::init();
                let res = app(&mut term, conf);
                ratatui::restore();
                return res;
            }
        }
        Commands::Flags { command } => command.run(),
    }

    Ok(())
}

fn app(terminal: &mut DefaultTerminal, conf: Conf) -> Result<(), Box<dyn Error>> {
    let mut app = App {
        screen: Box::new(screens::home::HomeScreen::new(conf)),
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
    screen: Box<dyn screens::screen::Screen + Send + Sync>,
}

impl App {
    fn render(&mut self, f: &mut Frame) {
        self.screen.render(f)
    }

    fn new(conf: Conf) -> Self {
        Self {
            screen: Box::new(HomeScreen::new(conf)),
        }
    }
}
