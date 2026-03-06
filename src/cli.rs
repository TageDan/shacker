use std::{
    fs::File,
    io::{BufReader, Read, Write},
    path::PathBuf,
};

use clap::{Parser, Subcommand};
use serde_json::from_str;

use crate::database::{self, Flag};

#[derive(Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run the server (WIP)
    Run,
    /// Add/Delete/List flags
    ///  (should be used before ctf starts as it will change flag id's and hence remove solves)
    // TODO: fix this
    Flags {
        #[command(subcommand)]
        command: FlagCommands,
    },
}

#[derive(Subcommand)]
pub enum FlagCommands {
    /// list flags in a tidy list
    List,
    /// delete a flag
    Delete {
        #[arg(short)]
        id: i32,
    },
    /// load json file with flags into database
    Load {
        /// Path to read from (default stdin)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// print flags as json to file
    Write {
        /// Path to write to (default stdout)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// add a new flag
    Add {
        /// name of the flag
        #[arg(short, long)]
        name: String,
        /// description of the flag (including links to files / services)
        #[arg(short, long)]
        description: String,
        /// points for solving the flag
        #[arg(short, long)]
        points: i32,
        /// solution string
        #[arg(short, long)]
        flag: String,
    },
}
impl FlagCommands {
    pub fn run(&self) {
        match &self {
            FlagCommands::Delete { id } => database::delete_flag(*id).unwrap(),
            FlagCommands::List => {
                for flag in database::Flag::get_all().unwrap() {
                    println!(
                        "{} - {} - {} - {} - {}",
                        flag.id(),
                        flag.name(),
                        &flag.description()[..10.min(flag.description().len())],
                        flag.points(),
                        flag.flag()
                    )
                }
            }
            FlagCommands::Add {
                name,
                description,
                points,
                flag,
            } => database::create_flag(&name, &description, *points, &flag),
            FlagCommands::Write { path } => {
                let mut file: Box<dyn Write> = match path {
                    None => Box::new(std::io::stdout()),
                    Some(path) => {
                        let file = File::create(path).unwrap();
                        Box::new(file)
                    }
                };
                let flags = Flag::get_all().unwrap();
                let flags_string = serde_json::to_string(&flags).unwrap();
                file.write_all(flags_string.as_bytes()).unwrap();
            }
            FlagCommands::Load { path } => {
                let mut file: Box<dyn Read> = match path {
                    None => Box::new(std::io::stdin()),
                    Some(path) => {
                        let file = File::open(path).unwrap();
                        Box::new(file)
                    }
                };
                let mut content = String::new();
                file.read_to_string(&mut content).unwrap();
                let flags: Vec<Flag> = from_str(&content).unwrap();
                database::clear_flags();
                for flag in flags {
                    database::create_flag(
                        flag.name(),
                        flag.description(),
                        flag.points(),
                        flag.flag(),
                    );
                }
            }
        }
    }
}
