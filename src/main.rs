use std::path::PathBuf;

use crate::app::App;

use dirs::home_dir;

mod action;
mod app;
mod globals;
mod helpers;
mod screen_guard;
mod tab;

fn main() -> anyhow::Result<()> {
    let home = home_dir().unwrap();
    let untreated_path = PathBuf::from(match std::env::args().nth(1) {
        Some(path) => path,
        None => format!("{}/TODO", home.display()),
    });

    let file_path = if untreated_path.is_dir() {
        untreated_path.join("TODO")
    } else {
        untreated_path
    };

    let mut app = App::new(file_path)?;

    app.run()
}
