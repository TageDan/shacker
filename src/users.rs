use std::{env, io::ErrorKind};

use sqlite::State;

const DB_FILE: &'static str = "db.db";

fn conn() -> sqlite::Connection {
    let mut path = env::home_dir().expect("no home dir found");
    path.push(".config");
    path.push("shacker");
    path.push(DB_FILE);
    sqlite::open(path).expect("couldn't open db")
}

pub fn create_missing_db() {
    let mut path = env::home_dir().expect("no home dir found");
    path.push(".config");
    path.push("shacker");

    std::fs::create_dir_all(&path).expect("failed to create db file");

    path.push(DB_FILE);
    match std::fs::File::create_new(path) {
        Ok(_) => (),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => (),
        Err(_) => panic!("failed to create db file"),
    };
}

pub fn create_user(name: String, password: String) -> std::io::Result<()> {
    const QUERY: &'static str = "SELECT * FROM users WHERE name = ?";
    let conn = conn();
    let mut statement = conn.prepare(QUERY).expect("error running sql query");
    statement
        .bind((1, name.as_str()))
        .expect("error binding parameter");
    if statement.next().is_ok_and(|x| x == State::Row) {
        return Err(std::io::Error::new(
            ErrorKind::AlreadyExists,
            "user already exists",
        ));
    }
    Ok(())
}
