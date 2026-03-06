use std::{env, error::Error, io::ErrorKind};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

const DB_FILE: &str = "db.db";

fn conn() -> Connection {
    let mut path = env::home_dir().expect("no home dir found");
    path.push(".shacker");
    path.push(DB_FILE);
    Connection::open(path).expect("couldn't open db")
}

pub fn create_missing_db() {
    let mut path = env::home_dir().expect("no home dir found");
    path.push(".shacker");

    std::fs::create_dir_all(&path).expect("failed to create db file");

    path.push(DB_FILE);
    match std::fs::File::create_new(path) {
        Ok(_) => (),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => (),
        Err(_) => panic!("failed to create db file"),
    };
    let conn = conn();
    const QUERY: &str = "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY AUTOINCREMENT, name STRING, password_hash STRING)";
    conn.execute(QUERY, [])
        .expect("could not create table users");
    const QUERY2: &str = "CREATE TABLE IF NOT EXISTS flags (id INTEGER PRIMARY KEY AUTOINCREMENT, name STRING, description STRING, points INTEGER, flag STRING)";
    conn.execute(QUERY2, [])
        .expect("could not create table flags");
    const QUERY3: &str = "CREATE TABLE IF NOT EXISTS solved (id INTEGER PRIMARY KEY AUTOINCREMENT, uid INTEGER, fid INTEGER, FOREIGN KEY (uid) REFERENCES users(id), FOREIGN KEY (fid) REFERENCES flags(id))";
    conn.execute(QUERY3, [])
        .expect("could not create table solved");
}

pub struct User {
    name: String,
    id: i32,
    points: i32,
}

impl User {
    fn new(name: String, id: i32) -> Self {
        let mut s = Self {
            name,
            id,
            points: 0,
        };
        let _ = s.reload();
        s
    }
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn register_user(name: &String, password: &String) -> Result<User, Box<dyn Error>> {
        let conn = conn();

        const QUERY: &str = "SELECT EXISTS (SELECT * FROM users WHERE name = ?1)";

        let mut stmt = conn.prepare(QUERY)?;
        let mut user = stmt.query([&name])?;

        if user.next()?.is_some_and(|x| x.get(0).is_ok_and(|x| x)) {
            return Err(Box::new(std::io::Error::new(
                ErrorKind::AlreadyExists,
                "user already exists",
            )));
        }

        const QUERY2: &str = "INSERT INTO users (name, password_hash) VALUES (?1,?2)";
        let mut stmt = conn.prepare(QUERY2)?;

        let hash = bcrypt::hash(password, 13)?;

        stmt.execute([name, &hash])?;

        // explicit drop so that we can borrow again
        drop(user);
        const QUERY3: &str = "SELECT id, name FROM users WHERE name = ?1";
        let mut stmt = conn.prepare(QUERY3)?;
        let mut user = stmt.query([&name])?;
        let user = user
            .next()?
            .ok_or("Error getting the newly registered user, try login in")?;
        let user = User::new(user.get(1)?, user.get(0)?);

        Ok(user)
    }

    // TODO: this function is vulnerable to timing attacks that allow user enumeration. This isn't really a concern but it might still be fun to look into fixing it.
    pub fn login(username: &str, password: &str) -> Result<User, Box<dyn Error>> {
        let conn = conn();

        const QUERY: &str = "SELECT * FROM users WHERE name = (?1)";

        let mut stmt = conn.prepare(QUERY)?;
        let mut user = stmt.query([&username])?;
        let user = user.next()?.ok_or("invalid password or username")?;
        let hash: String = user.get(2)?;

        if bcrypt::verify(password, &hash)? {
            Ok(User::new(user.get(1)?, user.get(0)?))
        } else {
            Err("invalid password or username".into())
        }
    }

    pub fn calculate_points(&self) -> Result<i32, Box<dyn Error>> {
        let conn = conn();
        const QUERY: &str = "SELECT IFNULL(SUM(f.points),0) FROM flags f JOIN solved c ON c.fid = f.id JOIN users u ON u.id = c.uid WHERE u.id = (?1)";
        let mut stmt = conn.prepare(QUERY).unwrap();
        let mut points = stmt.query([&self.id])?;
        let points = match points.next()? {
            Some(r) => r.get(0)?,
            None => 0,
        };
        Ok(points)
    }

    pub fn reload(&mut self) -> Result<(), Box<dyn Error>> {
        self.points = self.calculate_points()?;
        Ok(())
    }

    pub fn points(&self) -> i32 {
        self.points
    }

    pub fn id(&self) -> i32 {
        self.id
    }
}

#[derive(Serialize, Deserialize)]
pub struct Flag {
    name: String,
    description: String,
    flag: String,
    points: i32,
    #[serde(default)]
    id: i32,
    #[serde(skip_serializing, default)]
    solved: bool,
}

impl Flag {
    pub fn get_all_with_user(user: &User) -> Result<Vec<Flag>, Box<dyn Error>> {
        let conn = conn();
        const QUERY: &str = "SELECT f.*, (IFNULL(s.id, 0) AND f.id = s.fid) FROM flags f LEFT JOIN solved s ON (s.fid = f.id AND s.uid = ?1)";
        let mut stmt = conn.prepare(QUERY)?;
        let res = stmt.query_map([&user.id()], |row| {
            Ok(Flag::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })?;
        let res = res.filter_map(|x| x.ok()).collect();
        Ok(res)
    }

    pub fn get_all() -> Result<Vec<Flag>, Box<dyn Error>> {
        let conn = conn();
        const QUERY: &str = "SELECT * FROM flags";
        let mut stmt = conn.prepare(QUERY)?;
        let res = stmt.query_map([], |row| {
            Ok(Flag::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                false,
            ))
        })?;
        let res = res.filter_map(|x| x.ok()).collect();
        Ok(res)
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    fn new(
        id: i32,
        name: String,
        description: String,
        points: i32,
        flag: String,
        solved: bool,
    ) -> Self {
        Self {
            name,
            description,
            flag,
            points,
            id,
            solved,
        }
    }

    pub fn row_parts(&self) -> [&str; 4] {
        [
            &self.name,
            &self.description,
            format!("{}", self.points).leak(),
            if self.solved { "[x]" } else { "[ ]" },
        ]
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn points(&self) -> i32 {
        self.points
    }

    pub fn flag(&self) -> &str {
        &self.flag
    }

    pub fn solved(&self) -> bool {
        self.solved
    }

    pub fn mark_solved_for_user(&self, id: i32) -> Result<(), Box<dyn Error>> {
        let conn = conn();
        const QUERY: &str = "SELECT EXISTS ( SELECT * FROM solved WHERE uid = ?1 AND fid = ?2)";

        let mut stmt = conn.prepare(QUERY)?;
        let mut entry = stmt.query([&id, &self.id])?;

        if entry.next()?.is_some_and(|x| x.get(0).is_ok_and(|x| x)) {
            return Err(Box::new(std::io::Error::new(
                ErrorKind::AlreadyExists,
                "flag already solved",
            )));
        }

        const QUERY2: &str = "INSERT INTO solved (uid, fid) VALUES (?1,?2)";
        let mut statement = conn.prepare(QUERY2)?;

        statement.execute([&id, &self.id])?;
        Ok(())
    }
}

pub fn clear_flags() {
    let conn = conn();
    const QUERY: &str = "DELETE FROM flags";
    conn.execute(QUERY, []).unwrap();
}

pub fn create_flag(name: &str, description: &str, points: i32, flag: &str) {
    let conn = conn();
    const QUERY: &str = "INSERT INTO flags (name, description,points, flag) VALUES (?1,?2,?3,?4)";
    let mut stmt = conn.prepare(QUERY).unwrap();
    stmt.execute([name, description, &points.to_string(), flag])
        .unwrap();
}

pub fn delete_flag(id: i32) -> Result<(), Box<dyn Error>> {
    let conn = conn();
    const QUERY: &str = "DELETE FROM flags WHERE id = ?1";
    let mut stmt = conn.prepare(QUERY).unwrap();
    let c = stmt.execute([id]).unwrap();
    if c == 0 {
        return Err(format!("no flag with id: {}", id).into());
    }
    Ok(())
}
