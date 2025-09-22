use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::time;

use anyhow::Result;
use crossterm::style;
use crossterm::terminal::{
    self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
    enable_raw_mode,
};
use crossterm::{cursor, event, execute, queue};
// TODO: Replace this dependency with builtin logic
use unicode_width::UnicodeWidthChar;

use crate::globals;
use crate::tab::Tab;

pub fn clear_scr() -> io::Result<()> {
    execute!(io::stdout(), Clear(ClearType::All))?;
    Ok(())
}

pub fn goto(x: u16, y: u16) -> io::Result<()> {
    execute!(io::stdout(), cursor::MoveTo(x, y))?;
    Ok(())
}

pub fn goto_begin() -> io::Result<()> {
    goto(0, 0)
}

pub fn init_scr() -> io::Result<()> {
    execute!(io::stdout(), EnterAlternateScreen)?;
    goto_begin()?;
    clear_scr()?;
    enable_raw_mode()?;
    Ok(())
}

pub fn reset_scr() -> io::Result<()> {
    execute!(io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

pub fn get_todos_dones<P>(path: P) -> Result<(Vec<String>, Vec<String>)>
where
    P: AsRef<Path>,
{
    if path.as_ref().is_dir() {
        anyhow::bail!("Provided path is a directory");
    }

    let Ok(file) = File::open(&path) else {
        return Ok(Default::default());
    };

    let reader = io::BufReader::new(file);
    let mut todos = Vec::new();
    let mut dones = Vec::new();

    for line in reader.lines().map_while(Result::ok) {
        if let Some(s) = line.strip_prefix(globals::TODO_PREFIX) {
            todos.push(s.to_string());
        } else if let Some(s) = line.strip_prefix(globals::DONE_PREFIX) {
            dones.push(s.to_string());
        } else {
            anyhow::bail!("Malformed line: {:?}", line);
        }
    }

    Ok((todos, dones))
}

pub fn split_to_fit(s: &str, max_width: usize) -> (&str, Vec<&str>) {
    let mut width = 0;

    // Find the optimal split point
    for (i, c) in s.char_indices() {
        let cw = c.width().unwrap_or(1);

        // If adding this character would exceed max width, split before it
        if width + cw > max_width {
            if i == 0 {
                // Edge case: first character already exceeds max width
                // Split after this character to avoid infinite recursion
                let (first_char, remainder) = s.split_at(c.len_utf8());
                return (first_char, vec![remainder]);
            }

            // Split at the current position
            let (first_part, remainder) = s.split_at(i);
            return (first_part, split_remainder(remainder, max_width));
        }

        width += cw;
    }

    // The entire string fits within max_width
    (s, Vec::new())
}

// Helper function to split the remainder recursively
fn split_remainder(s: &str, max_width: usize) -> Vec<&str> {
    if s.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut current = s;

    while !current.is_empty() {
        let mut width = 0;
        let mut split_index = current.len();

        for (i, c) in current.char_indices() {
            let cw = c.width().unwrap_or(1);
            if width + cw > max_width {
                split_index = i;
                break;
            }
            width += cw;
        }

        let (chunk, remainder) = current.split_at(split_index);
        result.push(chunk);
        current = remainder;
    }

    result
}

pub fn handle_term_size(term_size: &mut (u16, u16)) -> io::Result<()> {
    // use std::sync::LazyLock;
    // // Program is not multithreaded/async, so it's fine for now
    // static mut LAST_CHECK: LazyLock<time::Instant> = LazyLock::new(time::Instant::now);
    // const DELTA: time::Duration = time::Duration::from_millis(0);
    //
    // #[allow(static_mut_refs)]
    // unsafe {
    //     let now = time::Instant::now();
    //     if now.duration_since(*LAST_CHECK) < DELTA {
    //         return Ok(());
    //     }
    //     *LAST_CHECK = now;
    // }
    *term_size = terminal::size()?;
    Ok(())
}

pub fn get_key_event(timeout: time::Duration) -> io::Result<Option<event::KeyCode>> {
    if !event::poll(timeout)? {
        return Ok(None);
    }

    match event::read()? {
        event::Event::Key(event) => Ok(Some(event.code)),
        _ => Ok(None),
    }
}

pub fn save_to_file<P>(path: P, todos: &[String], dones: &[String]) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    if todos.is_empty() && dones.is_empty() && !path.exists() {
        return Ok(());
    }

    let mut file = File::create(path)?;
    let mut proc = |strs: &[String], str_begin: &str| -> io::Result<()> {
        for s in strs {
            writeln!(file, "{}{}", str_begin, s)?;
        }
        Ok(())
    };

    proc(todos, globals::TODO_PREFIX)?;
    proc(dones, globals::DONE_PREFIX)?;

    println!("Saved state to {}", path.display());

    Ok(())
}
