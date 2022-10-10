mod app;
mod error;
mod help;

fn main() {
    match app::run() {
        Ok(_) => (),
        Err(e) => eprintln!("{}", e),
    }
}
