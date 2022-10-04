use std::path::PathBuf;

use tabled::{Table, Tabled};

const DEFAULT_SIZE: usize = 10;

#[derive(Tabled)]
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

fn main() -> Result<(), std::io::Error> {
    let db_path = {
        let mut path = dirs::home_dir().unwrap();
        path = path.join(".shi");
        if !path.exists() {
            std::fs::create_dir(&path)?;
        }
        path = path.join("history");
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
        histories.sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());
        let mut table = Table::new(histories);
        table.with(tabled::Style::psql());
        println!("{}", table);
        Ok(())
    } else if args[1] == "insert" && args.len() > 2 {
        if args[2].starts_with(' ') || (args[2] == "shi" && args.len() == 3) {
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
    } else if args[1] == "rm" {
        connection
            .execute(
                "
        DROP TABLE IF EXISTS history;
        ",
            )
            .unwrap();
        println!("Dropped history.");
        Ok(())
    } else {
        println!("No args.");
        Ok(())
    }
}
