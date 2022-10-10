use super::error::ShiError;
use super::help::HELP;
use serde::Serialize;
use sqlite::Connection;
use std::io::Write;
use tabled::{locator::ByColumnName, Disable, Table, Tabled};

const DEFAULT_SIZE: usize = 50;
#[derive(Tabled, Serialize)]
struct History {
    id: usize,
    command: String,
    time: String,
    path: String,
}

impl History {
    fn new() -> History {
        History {
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
        let mut path = dirs::home_dir().unwrap_or_else(|| panic!("Cannot detect home directory."));
        path = path.join(".shi");
        if !path.exists() {
            std::fs::create_dir(&path)?;
        }
        path
    };
    let db_path = {
        let mut path = app_path.clone();
        path.push(".history");
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
        print_histories(vec, Print::IgnorePath);
        Ok(())
    } else {
        match args[1].as_str() {
            "-h" | "--help" => {
                println!("{}", HELP);
                Ok(())
            }
            "-a" | "--all" => {
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
                print_histories(histories, Print::PrintPath);
                Ok(())
            }
            "-i" | "--insert" => {
                if args.len() == 2
                    || args[2].starts_with(' ')
                    || (args[2] == "shi" && args.len() == 3)
                {
                    Ok(())
                } else {
                    let command = args[2..args.len()].join(" ");
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
            "-d" | "--delete" => {
                if args.len() == 2 {
                    println!("Missing id.");
                    Ok(())
                } else {
                    let keys = &args[2..args.len()];
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
            "-r" | "--remove" => {
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
                    &format!(
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
                print_histories(histories, Print::PrintPath);
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
                    &format!(
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
                print_histories(histories, Print::PrintPath);
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
                        match wtr.write_record(&history) {
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
                let mut buffer = std::fs::File::create(&output).unwrap();
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
                    print_histories(vec, Print::IgnorePath);
                    Ok(())
                }
            }
        }
    }
}

fn print_histories(mut histories: Vec<History>, ignore_path: Print) {
    if histories.is_empty() {
        println!("No history.");
    } else {
        histories.sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());
        let mut table = Table::new(histories);
        table.with(tabled::Style::psql());
        if ignore_path == Print::IgnorePath {
            table.with(Disable::column(ByColumnName::new("path")));
        }
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
        &format!(
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
