mod cli;
mod conf;
mod database;
mod screens;
mod server;
mod theme;
use std::{error::Error, time::Duration};

use clap::Parser;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{
        self,
        event::{KeyCode, KeyModifiers, poll},
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
        screen: Box::new(screens::home::HomeScreen::new(
            conf,
            russh::keys::PublicKey::read_openssh_file(
                &std::env::home_dir()
                    .ok_or(
                        "to run locally you still need a public key under ~/.shh/id_ed_25519.pub, this is just so that your account can be identified correctly",
                    )?
                    .join(".ssh/id_ed25519.pub"),
            )?,
        )),
    };
    loop {
        terminal.draw(|f| app.render(f))?;
        if poll(Duration::from_millis(50)).is_ok_and(|x| x) {
            if let Some(k) = crossterm::event::read()?.as_key_press_event() {
                match (k.code, k.modifiers) {
                    (KeyCode::Char('q'), KeyModifiers::CONTROL) => return Ok(()),
                    k => {
                        if let Some(t) = app.screen.handle_input(Some(k)) {
                            app.screen = t;
                        }
                    }
                }
            }
        }
        if let Some(t) = app.screen.handle_input(None) {
            app.screen = t;
        }
    }
}

struct App {
    screen: Box<dyn screens::screen::Screen + Send>,
}

impl App {
    fn render(&mut self, f: &mut Frame) {
        self.screen.render(f)
    }

    fn new(conf: Conf, key: russh::keys::PublicKey) -> Self {
        Self {
            screen: Box::new(HomeScreen::new(conf, key)),
        }
    }
}
