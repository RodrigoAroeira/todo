mod globals;
mod helpers;
mod macros;
mod screen_guard;
mod state;

use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};

use helpers::{
    clear_scr, get_key_event, get_todos_dones, goto, goto_begin, handle_term_size, init_scr,
    reset_scr, save_to_file, write_todos_dones,
};
use screen_guard::ScreenGuard;
use state::{StateHandler, Tab};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);

    let untreated_path = PathBuf::from(match args.next() {
        Some(path) => path,
        None => format!("{}/TODO", std::env::var("HOME").unwrap()),
    });

    let file_path = if untreated_path.is_dir() {
        untreated_path.join("TODO")
    } else {
        untreated_path
    };

    if file_path.exists() && !file_path.is_file() {
        anyhow::bail!("Path {} exists and is not a file", file_path.display());
    }

    let (mut todos, mut dones) = get_todos_dones(&file_path)
        .context(format!("Couldn't read from {}", file_path.display()))?;

    let mut todos_idx = 0;
    let mut dones_idx = 0;
    let mut insert_mode = false;

    let mut term_size = crossterm::terminal::size()?;
    init_scr().context("Failed to initialize screen")?;
    let _guard = ScreenGuard {};

    let mut curr_tab = Tab::Todos;
    loop {
        handle_term_size(&mut term_size)?;
        clear_scr()?;
        goto_begin()?;
        write_todos_dones(&todos, &dones, term_size, curr_tab)?;

        let mid_scr = term_size.0 / 2;
        match curr_tab {
            Tab::Todos => goto(0, todos_idx as u16 + 1)?,
            Tab::Dones => goto(mid_scr, dones_idx as u16 + 1)?,
        }

        if let Some(code) = get_key_event(Duration::from_millis(1000 / 60))? {
            #[rustfmt::skip]
            let mut handler = StateHandler {
                curr_tab:    &mut curr_tab,
                todos:       &mut todos,
                todos_idx:   &mut todos_idx,
                dones:       &mut dones,
                dones_idx:   &mut dones_idx,
                insert_mode: &mut insert_mode,
            };

            // TODO: Maybe improve break condition
            if let Err(e) = handler.execute_action(code) {
                match e.to_string().as_str() {
                    globals::BREAK => break,
                    globals::NO_SAVE => return Ok(()),
                    _ => Err(e)?,
                }
            }
        }
    }

    reset_scr()?;
    if [&todos, &dones].iter().any(|slice| !slice.is_empty()) {
        save_to_file(file_path, &todos, &dones)?;
    }
    Ok(())
}
