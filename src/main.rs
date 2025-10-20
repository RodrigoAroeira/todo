use std::path::PathBuf;

use crate::app::App;

use anyhow::Context;
use dirs::home_dir;

mod action;
mod app;
mod globals;
mod helpers;
mod screen_guard;
mod tab;

fn main() -> anyhow::Result<()> {
    let untreated_path: PathBuf = match std::env::args().nth(1) {
        Some(path) => PathBuf::from(path),
        None => home_dir()
            .context("Unable to find home directory")?
            .join("TODO"),
    };

    let file_path = if untreated_path.is_dir() {
        untreated_path.join("TODO")
    } else {
        untreated_path
    };

    let mut app = App::new(file_path)?;

    app.run()
}
