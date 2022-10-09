mod help;

use serde::Serialize;
use sqlite::Connection;
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
        match select_histories(connection, None) {
            Ok(vec) => {
                print_histories(vec, Print::IgnorePath);
                Ok(())
            }
            Err(e) => {
                eprintln!("{}", e);
                Ok(())
            }
        }
    } else {
        match args[1].as_str() {
            "-h" | "--help" => {
                println!("{}", help::HELP);
                Ok(())
            }
            "-a" | "--all" => {
                let mut histories = vec![];
                match connection.iterate(
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
                ) {
                    Ok(_) => {
                        print_histories(histories, Print::PrintPath);
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        Ok(())
                    }
                }
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
                    match connection.execute(format!(
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
                    )) {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            eprintln!("{}", e);
                            Ok(())
                        }
                    }
                }
            }
            "-d" | "--delete" => {
                if args.len() == 2 {
                    println!("Missing id.");
                    Ok(())
                } else {
                    let keys = &args[2..args.len()];
                    for key in keys {
                        let key: usize = key.parse().unwrap();
                        match connection.execute(format!(
                            "
                            DELETE FROM history
                            WHERE id = {};
                        ",
                            key
                        )) {
                            Ok(_) => {
                                println!("Deleted id {}.", key);
                            }
                            Err(e) => {
                                eprintln!("{}", e);
                            }
                        }
                    }
                    Ok(())
                }
            }
            "-r" | "--remove" => {
                print!("Are you sure to delete all history? [y/N] ");
                std::io::stdout().flush().unwrap();
                let mut buffer = String::new();
                std::io::stdin().read_line(&mut buffer).unwrap();
                match buffer.as_str() {
                    "y" => {
                        match connection.execute(
                            "
                        DROP TABLE IF EXISTS history;
                        ",
                        ) {
                            Ok(_) => {
                                println!("Dropped history.");
                                Ok(())
                            }
                            Err(e) => {
                                eprintln!("{}", e);
                                Ok(())
                            }
                        }
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
                match connection.iterate(
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
                ) {
                    Ok(_) => {
                        print_histories(histories, Print::PrintPath);
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        Ok(())
                    }
                }
            }
            "-c" | "--command" => {
                println!("Printing commands that match the query...\n");
                let mut histories = vec![];
                match connection.iterate(
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
                ) {
                    Ok(_) => {
                        print_histories(histories, Print::PrintPath);
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        Ok(())
                    }
                }
            }
            "-o" | "--output" => {
                let mut wtr = csv::WriterBuilder::new().from_writer(vec![]);
                connection
                    .iterate(
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
                    )
                    .unwrap();

                let data = String::from_utf8(wtr.into_inner().unwrap()).unwrap();

                let data = data.as_bytes();
                let output = {
                    let mut path = app_path;
                    path.push("history.csv");
                    path
                };
                let mut buffer = std::fs::File::create(&output).unwrap();
                match buffer.write_all(data) {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        eprintln!("{}", e);
                        Ok(())
                    }
                }
            }
            _ => {
                if args.len() >= 3 {
                    println!("Invalid args.");
                    Ok(())
                } else {
                    let rows: Result<usize, std::num::ParseIntError> = args[1].parse();
                    match rows {
                        Ok(rows) => match select_histories(connection, Some(rows)) {
                            Ok(vec) => {
                                print_histories(vec, Print::IgnorePath);
                                Ok(())
                            }
                            Err(e) => {
                                eprintln!("{}", e);
                                Ok(())
                            }
                        },
                        Err(_) => {
                            eprintln!("Cannot parse input as number.");
                            Ok(())
                        }
                    }
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

fn select_histories(
    connection: Connection,
    rows: Option<usize>,
) -> Result<Vec<History>, sqlite::Error> {
    let mut histories = vec![];
    let rows = match rows {
        Some(rows) => rows,
        None => DEFAULT_SIZE,
    };
    match connection.iterate(
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
    ) {
        Ok(_) => Ok(histories),
        Err(e) => Err(e),
    }
}
