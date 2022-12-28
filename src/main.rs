mod app;
mod error;
mod help;
mod link;

fn main() {
    match app::run() {
        Ok(_) => (),
        Err(e) => eprintln!("{}", e),
    }
}
