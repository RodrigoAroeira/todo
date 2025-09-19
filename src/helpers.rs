use std::fs::File;
use std::io::{self, BufRead, BufReader, Write, stdout};
use std::path::Path;
use std::time;

use anyhow::Result;
use crossterm::style::Print;
use crossterm::terminal::{
    self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
    enable_raw_mode,
};
use crossterm::{cursor, event, execute, queue};
// TODO: Replace this dependency with builtin logic
use unicode_width::UnicodeWidthChar;

use crate::globals;

pub fn clear_scr() -> io::Result<()> {
    execute!(stdout(), Clear(ClearType::All))?;
    Ok(())
}

pub fn goto(x: u16, y: u16) -> io::Result<()> {
    execute!(stdout(), cursor::MoveTo(x, y))?;
    Ok(())
}

pub fn goto_begin() -> io::Result<()> {
    goto(0, 0)
}

pub fn init_scr() -> io::Result<()> {
    execute!(stdout(), EnterAlternateScreen)?;
    goto_begin()?;
    clear_scr()?;
    enable_raw_mode()?;
    Ok(())
}

pub fn reset_scr() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

pub fn get_todos_dones<P>(path: P) -> Result<(Vec<String>, Vec<String>)>
where
    P: AsRef<Path>,
{
    if path.as_ref().is_dir() {
        panic!("Provided path is a directory");
    }
    let Ok(file) = File::open(&path) else {
        return Ok(Default::default());
    };

    let reader = BufReader::new(file);
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

fn split_to_fit(s: &str, max_width: usize) -> (&str, &str) {
    let mut width = 0;
    for (i, c) in s.char_indices() {
        let cw = c.width().unwrap_or(0);
        if width + cw > max_width {
            return s.split_at(i);
        }
        width += cw;
    }
    (s, "")
}

pub fn write_todos_dones(
    todos: &[String],
    dones: &[String],
    term_size: (u16, u16),
) -> io::Result<()> {
    let (cols, _) = term_size;
    let col_mid = cols / 2;
    let mut handle = stdout().lock();
    queue!(
        handle,
        Print(format!("TODO{}DONE\r\n", " ".repeat(col_mid as usize - 4)))
    )?;

    let mut proc = |strs: &[String], line_begin: &str, is_dones: bool| -> io::Result<()> {
        for (y, item) in strs.iter().enumerate() {
            let whole_str = format!("{} {}\r\n", line_begin, item);

            if is_dones {
                goto(col_mid, y as u16 + 1)?;
            }

            if whole_str.trim().chars().count() < col_mid as usize {
                queue!(handle, Print(whole_str))?;
                continue;
            }
            let (half1, half2) =
                split_to_fit(&whole_str, col_mid as usize - if is_dones { 0 } else { 1 });
            queue!(handle, Print(half1.to_string() + "\r\n"))?;
            queue!(
                handle,
                Print(format!(
                    "{space}{txt}",
                    space = " ".repeat(line_begin.chars().count() + 1),
                    txt = half2
                ))
            )?;
        }
        Ok(())
    };

    proc(todos, globals::TODO_INDICATOR, false)?;
    proc(dones, globals::DONE_INDICATOR, true)?;

    handle.flush()
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
