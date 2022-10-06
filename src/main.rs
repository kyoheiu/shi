use miniserde::{json, Serialize};
use std::{io::Write, path::PathBuf};
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

fn main() -> Result<(), std::io::Error> {
    let app_path = {
        let mut path = dirs::home_dir().unwrap();
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

    let connection = sqlite::open(db_path).unwrap();
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
        let mut histories = vec![];
        connection
            .iterate(
                &format!(
                    "SELECT *
                    FROM history
                    ORDER BY id DESC
                    LIMIT {}",
                    DEFAULT_SIZE
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
            )
            .unwrap();
        print_histories(histories, Print::IgnorePath);
        Ok(())
    } else if args.len() >= 2 {
        match args[1].as_str() {
            "-a" | "--all" => {
                let mut histories = vec![];
                connection
                    .iterate(
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
                    )
                    .unwrap();
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
                    let path = match std::env::current_dir() {
                        Ok(path) => path,
                        Err(_) => PathBuf::from("UNKNOWN"),
                    };
                    connection
                        .execute(format!(
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
                        ))
                        .unwrap();
                    Ok(())
                }
            }
            "-d" | "--drop" => {
                connection
                    .execute(
                        "
                        DROP TABLE IF EXISTS history;
                        ",
                    )
                    .unwrap();
                println!("Dropped history.");
                Ok(())
            }
            "-p" | "--path" => {
                println!("Printing commands executed in directories that match the query...\n");
                let mut histories = vec![];
                connection
                    .iterate(
                        &format!(
                            "SELECT *
                            FROM history
                            WHERE path LIKE '%{}%'
                            ORDER BY id DESC
                            LIMIT {}",
                            args[2], DEFAULT_SIZE
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
                    )
                    .unwrap();
                print_histories(histories, Print::PrintPath);
                Ok(())
            }
            "-c" | "--command" => {
                println!("Printing commands that match the query...\n");
                let mut histories = vec![];
                connection
                    .iterate(
                        &format!(
                            "SELECT *
                            FROM history
                            WHERE command LIKE '%{}%'
                            ORDER BY id DESC
                            LIMIT {}",
                            args[2], DEFAULT_SIZE
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
                    )
                    .unwrap();
                print_histories(histories, Print::PrintPath);
                Ok(())
            }
            "-o" | "--output" => {
                let mut histories = vec![];
                connection
                    .iterate(
                        "SELECT *
                                FROM history",
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
                    )
                    .unwrap();
                let j = json::to_string(&histories);
                let j = j.as_bytes();
                let output = {
                    let mut path = app_path;
                    path.push("history.json");
                    path
                };
                let mut buffer = std::fs::File::create(&output).unwrap();
                buffer.write_all(j).unwrap();
                Ok(())
            }
            _ => Ok(()),
        }
    } else {
        println!("No args.");
        Ok(())
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
