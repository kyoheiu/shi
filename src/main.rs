use std::fmt::Write as _;

const DEFAULT_SIZE: usize = 50;

fn main() {
    let connection = sqlite::open("test").unwrap();
    connection
        .execute(
            "
        CREATE TABLE IF NOT EXISTS history (
            id INTEGER PRIMARY KEY,
            time DATETIME NOT NULL,
            command TEXT NOT NULL
        );
        ",
        )
        .unwrap();

    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        connection
            .iterate("SELECT * FROM history", |pairs| {
                let mut count: usize = 0;
                let mut result = String::new();
                for &(column, value) in pairs.iter() {
                    if column == "id" {
                        result.push_str(value.unwrap());
                    } else {
                        let _ = write!(result, " {}", value.unwrap());
                    }
                    count += 1;
                }
                println!("{}", result);
                count != DEFAULT_SIZE
            })
            .unwrap();
    } else if args[1] == "insert" && args.len() > 2 {
        if args[2].starts_with(' ') {
            return;
        } else {
            let command = args[2..args.len()].join(" ");
            connection
                .execute(format!(
                    "
                    INSERT INTO history (time, command) VALUES (datetime('now', 'localtime'), '{}');
                    ",
                    command
                ))
                .unwrap();
        }
    } else if args[1] == "rm" {
        connection
            .execute(
                "
        DROP TABLE IF EXISTS history;
        ",
            )
            .unwrap();
    }
}
