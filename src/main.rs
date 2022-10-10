mod app;
mod error;
mod help;

use error::ShiError;

fn main() -> Result<(), ShiError> {
    match app::run() {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("{}", e);
            Ok(())
        }
    }
}
