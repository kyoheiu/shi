use super::error::ShiError;
use super::help::HELP;
use crossterm::style::Stylize;
use serde::Serialize;
use sqlite::{Connection, State};
use std::io::Write;
use tabled::{settings::locator::ByColumnName, settings::Disable, settings::Style, Table, Tabled};

const DEFAULT_SIZE: usize = 50;
const SHI_DIR: &str = "shi";
const DB_NAME: &str = ".history";

#[derive(Tabled, Serialize)]
struct History {
    link: String,
    id: usize,
    command: String,
    time: String,
    path: String,
}

impl History {
    fn new() -> History {
        History {
            link: "".to_owned(),
            id: 0,
            command: "".to_string(),
            time: "".to_string(),
            path: "".to_string(),
        }
    }
}

#[derive(PartialEq)]
enum Print {
    IgnorePath,
    PrintPath,
}
pub fn run() -> Result<(), ShiError> {
    let app_path = {
        let mut path =
            dirs::data_local_dir().unwrap_or_else(|| panic!("Cannot detect data local directory."));
        path = path.join(SHI_DIR);
        if !path.exists() {
            std::fs::create_dir(&path)?;
        }
        path
    };
    let db_path = {
        let mut path = app_path.clone();
        path.push(DB_NAME);
        path
    };

    let connection =
        sqlite::open(db_path).unwrap_or_else(|_| panic!("Cannot create or open sqlite database."));
    connection
        .execute(
            "
            CREATE TABLE IF NOT EXISTS history (
            id INTEGER PRIMARY KEY,
            command TEXT NOT NULL,
            time DATETIME NOT NULL,
            path TEXT NOT NULL
        );
        ",
        )
        .unwrap();

    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        let vec = select_histories(connection, None)?;
        print_histories_to_choose(vec, Print::IgnorePath)?;
        Ok(())
    } else {
        match args[1].as_str() {
            "-h" | "--help" => {
                println!("{}", HELP);
                Ok(())
            }
            "-a" | "--all" => {
                if args.len() > 2 {
                    eprintln!("Invalid argument: See help.");
                }
                let mut histories = vec![];
                connection.iterate(
                    "SELECT *
                            FROM history
                            ORDER BY id DESC
                            ",
                    |pairs| {
                        let mut history = History::new();
                        for &(column, value) in pairs.iter() {
                            match column {
                                "id" => {
                                    history.id = value.map(|x| x.parse().unwrap()).unwrap();
                                }
                                "command" => {
                                    history.command = value.unwrap().to_owned();
                                }
                                "time" => {
                                    history.time = value.unwrap().to_owned();
                                }
                                "path" => {
                                    history.path = value.unwrap().to_owned();
                                }
                                _ => {}
                            }
                        }
                        histories.push(history);
                        true
                    },
                )?;
                print_histories(histories);
                Ok(())
            }
            "-i" | "--insert" => {
                if args.len() == 2 || args[2].starts_with(' ') {
                    Ok(())
                } else {
                    let command = args[2..].join(" ");
                    let path = std::env::current_dir()?;
                    let path = path
                        .to_str()
                        .unwrap_or_else(|| panic!("Cannot convert path to UTF8."));
                    let query = " INSERT INTO history (time, command, path) VALUES ( datetime('now', 'localtime'), :command, :path);";
                    let mut statement = connection.prepare(query)?;
                    statement.bind(&[(":command", command.as_str()), (":path", path)][..])?;
                    while let Ok(State::Row) = statement.next() {
                        continue;
                    }
                    Ok(())
                }
            }
            "-r" | "--remove" => {
                if args.len() == 2 {
                    println!("Missing id.");
                    Ok(())
                } else {
                    let keys = &args[2..];
                    for key in keys {
                        let query = " DELETE FROM history WHERE id = :id;";
                        let mut statement = connection.prepare(query)?;
                        statement.bind((":id", key.as_str()))?;
                        while let Ok(State::Row) = statement.next() {
                            println!("Deleted id {}.", key);
                        }
                    }
                    Ok(())
                }
            }
            "--drop" => {
                print!("Are you sure to delete all history? [y/N] ");
                std::io::stdout().flush()?;
                let mut buffer = String::new();
                std::io::stdin().read_line(&mut buffer)?;
                match buffer.as_str() {
                    "y" => {
                        connection.execute(
                            "
                        DROP TABLE IF EXISTS history;
                        ",
                        )?;
                        println!("Dropped history.");
                        Ok(())
                    }
                    _ => {
                        println!("Canceled.");
                        Ok(())
                    }
                }
            }
            "-p" | "--path" => {
                println!("Printing commands executed in directories that match the query...\n");
                let mut histories = vec![];
                let rows = if args.len() == 4 {
                    args[3].to_owned()
                } else {
                    DEFAULT_SIZE.to_string()
                };
                let query =
                    "SELECT * FROM history WHERE path LIKE '%'||?||'%' ORDER BY id DESC LIMIT ?";
                for row in connection
                    .prepare(query)?
                    .into_iter()
                    .bind(&[(1, args[2].as_str()), (2, &rows)][..])?
                    .map(|row| row.unwrap())
                {
                    let mut history = History::new();
                    history.id = row.read::<i64, _>("id") as usize;
                    history.command = row.read::<&str, _>("command").to_string();
                    history.time = row.read::<&str, _>("time").to_string();
                    history.path = row.read::<&str, _>("path").to_string();
                    histories.push(history);
                }
                print_histories_to_choose(histories, Print::PrintPath)?;
                Ok(())
            }
            "-c" | "--command" => {
                println!("Printing commands that match the query...\n");
                let mut histories = vec![];
                let rows = if args.len() == 4 {
                    args[3].to_owned()
                } else {
                    DEFAULT_SIZE.to_string()
                };
                let query =
                    "SELECT * FROM history WHERE command LIKE '%'||?||'%' ORDER BY id DESC LIMIT ?";
                for row in connection
                    .prepare(query)?
                    .into_iter()
                    .bind(&[(1, args[2].as_str()), (2, &rows)][..])?
                    .map(|row| row.unwrap())
                {
                    let mut history = History::new();
                    history.id = row.read::<i64, _>("id") as usize;
                    history.command = row.read::<&str, _>("command").to_string();
                    history.time = row.read::<&str, _>("time").to_string();
                    history.path = row.read::<&str, _>("path").to_string();
                    histories.push(history);
                }
                print_histories_to_choose(histories, Print::PrintPath)?;
                Ok(())
            }
            "-o" | "--output" => {
                let mut wtr = csv::WriterBuilder::new().from_writer(vec![]);
                connection.iterate(
                    "SELECT *
                                FROM history",
                    |pairs| {
                        let mut history: [&str; 4] = ["", "", "", ""];
                        for &(column, value) in pairs.iter() {
                            match column {
                                "id" => {
                                    history[0] = value.unwrap();
                                }
                                "command" => {
                                    history[1] = value.unwrap();
                                }
                                "time" => {
                                    history[2] = value.unwrap();
                                }
                                "path" => {
                                    history[3] = value.unwrap();
                                }
                                _ => {}
                            }
                        }
                        match wtr.write_record(history) {
                            Ok(_) => true,
                            Err(e) => {
                                eprintln!("{}", e);
                                false
                            }
                        }
                    },
                )?;

                let data = String::from_utf8(wtr.into_inner()?)?;
                let data = data.as_bytes();
                let output = {
                    let mut path = app_path;
                    path.push("history.csv");
                    path
                };
                let mut buffer = std::fs::File::create(output).unwrap();
                buffer.write_all(data)?;
                Ok(())
            }
            _ => {
                if args.len() >= 3 {
                    println!("Invalid args.");
                    Ok(())
                } else {
                    let rows = args[1].parse()?;
                    let vec = select_histories(connection, Some(rows))?;
                    print_histories_to_choose(vec, Print::IgnorePath)?;
                    Ok(())
                }
            }
        }
    }
}

fn print_histories_to_choose(
    mut histories: Vec<History>,
    ignore_path: Print,
) -> Result<(), ShiError> {
    if histories.is_empty() {
        println!("No history.");
    }
    histories.sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());
    histories.pop();
    let len = histories.len();
    for (i, h) in histories.iter_mut().enumerate() {
        h.link = super::link::to_base32(len - i);
    }
    let mut table = Table::new(&histories);
    table.with(Style::psql());
    if ignore_path == Print::IgnorePath {
        table.with(Disable::column(ByColumnName::new("path")));
    }
    println!("{}", table);

    match std::env::var("SHI_CLIP") {
        Err(_) => {
            Ok(println!("If you'd like to copy a command to the clipboard, set any clipboard utility such as xclip or wl-copy as $SHI_CLIP."))
        }
        Ok(copy) => {

    print!("Enter the link (left-most chars) to copy the command > ");
    std::io::stdout().flush()?;
    let i = get_input_link()?;
    if let Some(h) = histories.get(len - i) {
        copy_command(copy, &h.command)?;
    }
    Ok(())
        }
    }
}

fn print_histories(mut histories: Vec<History>) {
    if histories.is_empty() {
        println!("No history.");
    } else {
        histories.pop();
        histories.sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());
        let mut table = Table::new(histories);
        table.with(Style::psql());
        table.with(Disable::column(ByColumnName::new("number")));
        println!("{}", table);
    }
}

fn select_histories(connection: Connection, rows: Option<usize>) -> Result<Vec<History>, ShiError> {
    let mut histories = vec![];
    let rows = match rows {
        Some(rows) => rows + 1,
        None => DEFAULT_SIZE + 1,
    };
    let query = "SELECT * FROM history ORDER BY id DESC LIMIT ?";
    for row in connection
        .prepare(query)?
        .into_iter()
        .bind(&[(1, rows.to_string().as_str())][..])?
        .map(|row| row.unwrap())
    {
        let mut history = History::new();
        history.id = row.read::<i64, _>("id") as usize;
        history.command = row.read::<&str, _>("command").to_string();
        history.time = row.read::<&str, _>("time").to_string();
        history.path = row.read::<&str, _>("path").to_string();
        histories.push(history);
    }
    Ok(histories)
}

fn copy_command(copy_command: String, commands: &str) -> Result<(), ShiError> {
    println!("Copying command => {}", commands.green());
    let mut p = std::process::Command::new(copy_command)
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    let mut stdin = p.stdin.take().expect("Failed to open stdin.");
    stdin
        .write_all(commands.as_bytes())
        .expect("Failed to pass command via stdin.");
    Ok(println!("✌ Copied to the clipboard."))
}

fn get_input_link() -> Result<usize, ShiError> {
    let mut input = String::new();
    let stdin = std::io::stdin();
    stdin.read_line(&mut input)?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        Err(ShiError::Input)
    } else if let Some(i) = super::link::from_base32(trimmed) {
        Ok(i)
    } else {
        Err(ShiError::ParseInt)
    }
}
