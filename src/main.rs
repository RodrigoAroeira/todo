use std::path::PathBuf;

use crate::app::App;

mod action;
mod app;
mod globals;
mod helpers;
mod screen_guard;
mod tab;

fn main() -> anyhow::Result<()> {
    let untreated_path = PathBuf::from(match std::env::args().nth(1) {
        Some(path) => path,
        None => format!("{}/TODO", std::env::var("HOME").unwrap()),
    });

    let file_path = if untreated_path.is_dir() {
        untreated_path.join("TODO")
    } else {
        untreated_path
    };

    let mut app = App::new(file_path)?;

    app.run()
}
