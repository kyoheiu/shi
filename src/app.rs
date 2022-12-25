use super::error::ShiError;
use super::help::HELP;
use serde::Serialize;
use sqlite::Connection;
use std::io::Write;
use tabled::{locator::ByColumnName, Disable, Table, Tabled};

const DEFAULT_SIZE: usize = 50;
const SHI_DIR: &str = "shi";
const DB_NAME: &str = ".history";

#[derive(Tabled, Serialize)]
struct History {
    number: usize,
    id: usize,
    command: String,
    time: String,
    path: String,
}

impl History {
    fn new() -> History {
        History {
            number: 0,
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
            dirs::data_local_dir().unwrap_or_else(|| panic!("Cannot detect home directory."));
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
                if args.len() == 2
                    || args[2].starts_with(' ')
                    || (args[2] == "shi" && args.len() == 3)
                {
                    Ok(())
                } else {
                    let command = args[2..].join(" ");
                    let path = std::env::current_dir()?;
                    connection.execute(format!(
                        "
                            INSERT INTO history (time, command, path)
                            VALUES (
                                datetime('now', 'localtime'), 
                                '{}',
                                '{}'
                            );
                            ",
                        command,
                        path.display()
                    ))?;
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
                        let key: usize = key.parse()?;
                        connection.execute(format!(
                            "
                            DELETE FROM history
                            WHERE id = {};
                        ",
                            key
                        ))?;
                        println!("Deleted id {}.", key);
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
                    let rows: Result<usize, std::num::ParseIntError> = args[3].parse();
                    match rows {
                        Ok(rows) => rows,
                        Err(_) => DEFAULT_SIZE,
                    }
                } else {
                    DEFAULT_SIZE
                };
                connection.iterate(
                    format!(
                        "SELECT *
                            FROM history
                            WHERE path LIKE '%{}%'
                            ORDER BY id DESC
                            LIMIT {}",
                        args[2], rows
                    ),
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
                print_histories_to_choose(histories, Print::PrintPath)?;
                Ok(())
            }
            "-c" | "--command" => {
                println!("Printing commands that match the query...\n");
                let mut histories = vec![];
                let rows = if args.len() == 4 {
                    let rows: Result<usize, std::num::ParseIntError> = args[3].parse();
                    match rows {
                        Ok(rows) => rows,
                        Err(_) => DEFAULT_SIZE,
                    }
                } else {
                    DEFAULT_SIZE
                };
                connection.iterate(
                    format!(
                        "SELECT *
                            FROM history
                            WHERE command LIKE '%{}%'
                            ORDER BY id DESC
                            LIMIT {}",
                        args[2], rows
                    ),
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
    let len = histories.len();
    for (i, h) in histories.iter_mut().enumerate() {
        h.number = len - i;
    }
    let mut table = Table::new(&histories);
    table.with(tabled::Style::psql());
    if ignore_path == Print::IgnorePath {
        table.with(Disable::column(ByColumnName::new("path")));
    }
    println!("{}", table);
    print!("Input number to copy command: > ");
    std::io::stdout().flush()?;
    let i = get_input_number()?;
    if let Some(h) = histories.get(len - i) {
        let commands: Vec<&str> = h.command.split_ascii_whitespace().collect();
        copy_command(commands)?;
    }
    Ok(())
}

fn print_histories(mut histories: Vec<History>) {
    if histories.is_empty() {
        println!("No history.");
    } else {
        histories.sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());
        let mut table = Table::new(histories);
        table.with(tabled::Style::psql());
        table.with(Disable::column(ByColumnName::new("number")));
        println!("{}", table);
    }
}

fn select_histories(connection: Connection, rows: Option<usize>) -> Result<Vec<History>, ShiError> {
    let mut histories = vec![];
    let rows = match rows {
        Some(rows) => rows,
        None => DEFAULT_SIZE,
    };
    connection.iterate(
        format!(
            "SELECT *
                    FROM history
                    ORDER BY id DESC
                    LIMIT {}",
            rows
        ),
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
                    _ => {}
                }
            }
            histories.push(history);
            true
        },
    )?;
    Ok(histories)
}

fn copy_command(commands: Vec<&str>) -> Result<(), ShiError> {
    let commands = commands.join(" ");
    let copy_command = std::env::var("SHI_CLIP");
    match copy_command {
        Ok(copy_command) => {
            let mut p = std::process::Command::new(copy_command)
                .stdin(std::process::Stdio::piped())
                .spawn()?;
            let mut stdin = p.stdin.take().expect("Failed to open stdin.");
            std::thread::spawn(move || {
                stdin
                    .write_all(commands.as_bytes())
                    .expect("Failed to pass command via stdin.");
            });
        }
        Err(_) => return Err(ShiError::Env),
    }
    Ok(println!("Copied commands to the clipboard."))
}

fn get_input_number() -> Result<usize, ShiError> {
    let mut input = String::new();
    let stdin = std::io::stdin();
    stdin.read_line(&mut input)?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        Err(ShiError::Input)
    } else if let Ok(i) = input.trim().parse() {
        Ok(i)
    } else {
        Err(ShiError::ParseInt)
    }
}
