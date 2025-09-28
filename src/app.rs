use std::{
    io::{self, Write},
    mem,
    path::{Path, PathBuf},
    time::Duration,
};

use crossterm::{event::KeyCode, queue, style};

use crate::action::{Action, InsertAction};
use crate::helpers::{
    clear_scr, get_key_event, get_todos_dones, goto, goto_begin, handle_term_size, init_scr,
    save_to_file, split_to_fit,
};
use crate::{globals, screen_guard::ScreenGuard, tab::Tab};

enum InsertMode {
    New,
    Edit(String),
}

enum Mode {
    Normal,
    Insert(InsertMode),
    Help,
}

pub struct App {
    todos: Vec<String>,
    dones: Vec<String>,
    file_path: PathBuf,
    todos_idx: usize,
    dones_idx: usize,
    curr_tab: Tab,
    mode: Mode,
}

impl App {
    pub fn new<P>(file_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let (todos, dones) = get_todos_dones(&file_path)?;
        let s = Self {
            todos,
            dones,
            file_path: file_path.as_ref().to_path_buf(),
            todos_idx: 0,
            dones_idx: 0,
            curr_tab: Tab::Todos,
            mode: Mode::Normal,
        };
        Ok(s)
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        if let Err(e) = self.main_loop() {
            match e.to_string().as_str() {
                globals::NO_SAVE => return Ok(()),
                globals::BREAK => {}
                _ => Err(e)?,
            }
        }
        save_to_file(&self.file_path, &self.todos, &self.dones)?;
        Ok(())
    }

    fn main_loop(&mut self) -> anyhow::Result<()> {
        let _guard = ScreenGuard {};
        let mut term_size = Default::default();
        init_scr()?;
        loop {
            self.clamp_indexes();
            handle_term_size(&mut term_size)?;
            clear_scr()?;
            goto_begin()?;
            self.write_screen(term_size)?;
            self.write_status(term_size)?;

            let mid_scr = term_size.0 / 2;
            match self.curr_tab {
                Tab::Todos => goto(0, self.todos_idx as u16 + 1)?,
                Tab::Dones => goto(mid_scr, self.dones_idx as u16 + 1)?,
            }

            if let Some(event) = get_key_event(Duration::from_millis(1000 / 60))? {
                match self.mode {
                    Mode::Normal => {
                        if let Ok(action) = Action::try_from(event) {
                            self.execute_action(action)?;
                        }
                    }
                    Mode::Insert(_) => {
                        if let Ok(action) = InsertAction::try_from(event) {
                            self.handle_insert_mode(action);
                        }
                    }
                    Mode::Help => {
                        if let Ok(action) = Action::try_from(event) {
                            self.handle_help_mode(action);
                        }
                    }
                }
            }
        }
    }

    fn write_screen(&self, term_size: (u16, u16)) -> io::Result<()> {
        if matches!(self.mode, Mode::Help) {
            self.write_help_screen(term_size)?;
        } else {
            self.write_header(term_size)?;
            self.write_todos_dones(term_size)?;
        }
        Ok(())
    }

    fn write_header(&self, term_size: (u16, u16)) -> io::Result<()> {
        let col_mid = term_size.0 / 2;
        let mut handle = io::stdout().lock();
        let is_tab_todo = matches!(self.curr_tab, Tab::Todos);
        let draw_header =
            |s: &str, should_reverse: bool, handle: &mut io::StdoutLock<'_>| -> io::Result<()> {
                if should_reverse {
                    queue!(handle, style::SetAttribute(style::Attribute::Reverse))?;
                }

                queue!(handle, style::Print(s))?;

                if should_reverse {
                    queue!(handle, style::SetAttribute(style::Attribute::NoReverse))?;
                }

                Ok(())
            };

        draw_header("TODO", is_tab_todo, &mut handle)?;
        queue!(handle, style::Print(" ".repeat(col_mid as usize - 4)))?;
        draw_header("DONE\r\n", !is_tab_todo, &mut handle)?;
        Ok(())
    }

    fn write_status(&self, term_size: (u16, u16)) -> io::Result<()> {
        let (_cols, rows) = term_size;
        goto(0, rows - 1)?;
        let txt = match self.mode {
            Mode::Normal => "NORMAL",
            Mode::Insert(InsertMode::New) => "INSERT",
            Mode::Insert(InsertMode::Edit(_)) => "EDIT",
            Mode::Help => "HELP",
        };

        let mut handle = io::stdout();
        queue!(handle, style::SetAttribute(style::Attribute::Reverse))?;
        queue!(handle, style::Print(txt))?;
        queue!(handle, style::SetAttribute(style::Attribute::NoReverse))?;
        Ok(())
    }

    fn write_help_screen(&self, term_size: (u16, u16)) -> io::Result<()> {
        let (cols, _) = term_size;
        let mut handle = io::stdout();

        // Helper to print a full line with newline
        let mut println = |s: &str| -> io::Result<()> {
            queue!(handle, style::Print(s))?;
            queue!(handle, style::Print("\r\n"))?;
            Ok(())
        };

        // Title bar
        println(&"=".repeat(cols as usize))?;
        println("HELP")?;
        println(&"=".repeat(cols as usize))?;
        println("")?;

        // Sections
        println("ACTIONS")?;
        println("  h        - Show this screen")?;
        println("  i / o    - Insert item above / below")?;
        println("  e        - Edit item under cursor")?;
        println("  J / K    - Move item under cursor down / up")?;
        println("  q        - Save and quit")?;
        println("  Q        - Quit without saving")?;
        println("")?;

        println("MOVEMENT")?;
        println("  j / k    - Move cursor down / up")?;
        println("  g / G    - Jump to beginning / end")?;
        println("")?;

        println("INSERT / EDIT MODE")?;
        println("  (type normally to edit text)")?;
        println("  Enter    - Save changes")?;
        println("  Esc      - Cancel")?;
        println("")?;

        println("LEAVING HELP")?;
        println("  q / Q    - Quit help screen")?;
        println("")?;

        println(&"=".repeat(cols as usize))?;

        handle.flush()?;
        Ok(())
    }

    fn write_todos_dones(&self, term_size: (u16, u16)) -> io::Result<()> {
        let (cols, _) = term_size;
        let col_mid = cols / 2;
        let mut handle = io::stdout().lock();

        let mut draw_items = |items: &[String],
                              line_begin: &str,
                              is_active_tab: bool,
                              selected_idx: usize,
                              col_offset: u16|
         -> io::Result<()> {
            let mut current_line = 1;
            for (idx, item) in items.iter().enumerate() {
                let should_highlight = is_active_tab && idx == selected_idx;
                let full_line = format!("{} {}", line_begin, item);
                let (first_line, rest_lines) = split_to_fit(
                    &full_line,
                    col_mid as usize - if col_offset > 0 { 0 } else { 1 },
                );

                // Start at the first line for this item

                // Draw first line
                goto(col_offset, current_line)?;
                if should_highlight {
                    queue!(handle, style::SetAttribute(style::Attribute::Reverse))?;
                }
                queue!(handle, style::Print(first_line))?;
                if should_highlight {
                    queue!(handle, style::SetAttribute(style::Attribute::NoReverse))?;
                }
                current_line += 1;

                let padding = " ".repeat(line_begin.len() + 1);
                for line in rest_lines {
                    goto(col_offset, current_line)?;
                    if should_highlight {
                        queue!(handle, style::SetAttribute(style::Attribute::Reverse))?;
                    }
                    queue!(handle, style::Print(format!("{}{}", padding, line)))?;
                    if should_highlight {
                        queue!(handle, style::SetAttribute(style::Attribute::NoReverse))?;
                    }
                    current_line += 1;
                }
            }
            Ok(())
        };

        draw_items(
            &self.todos,
            globals::TODO_INDICATOR,
            matches!(self.curr_tab, Tab::Todos),
            self.todos_idx,
            0,
        )?;
        draw_items(
            &self.dones,
            globals::DONE_INDICATOR,
            matches!(self.curr_tab, Tab::Dones),
            self.dones_idx,
            col_mid,
        )?;

        handle.flush()?;
        Ok(())
    }

    fn execute_action(&mut self, action: Action) -> anyhow::Result<()> {
        match action {
            Action::Enter => self.handle_enter_press(),
            Action::SwitchTab => self.curr_tab = self.curr_tab.toggle(),
            Action::Insert(direction) => self.start_insert_mode(direction),
            Action::Edit => self.start_edit_mode(),
            Action::MoveCursor(direction) => self.handle_cursor_move(direction),
            Action::MoveItem(direction) => self.handle_move_item(direction),
            Action::GotoBegin => self.goto_list_pos(0),
            Action::GotoEnd => self.goto_list_pos(usize::MAX),
            Action::Delete => self.handle_delete(),
            Action::SaveQuit => anyhow::bail!(globals::BREAK),
            Action::NoSaveQuit => anyhow::bail!(globals::NO_SAVE),
            Action::ShowHelp => self.mode = Mode::Help,
        }

        Ok(())
    }

    fn handle_enter_press(&mut self) {
        match self.curr_tab {
            Tab::Todos => {
                if self.todos.is_empty() {
                    return;
                }
                let value = self.todos.remove(self.todos_idx);
                self.dones.push(value);
            }
            Tab::Dones => {
                if self.dones.is_empty() {
                    return;
                }
                let value = self.dones.remove(self.dones_idx);
                self.todos.push(value);
            }
        }
    }

    fn handle_cursor_move(&mut self, direction: KeyCode) {
        let idx = match self.curr_tab {
            Tab::Todos => &mut self.todos_idx,
            Tab::Dones => &mut self.dones_idx,
        };
        match direction {
            KeyCode::Down => *idx += 1,
            KeyCode::Up => *idx = idx.saturating_sub(1),
            _ => unreachable!("This spot should't be reachable"),
        };
    }

    fn handle_delete(&mut self) {
        let (target_vec, idx) = match self.curr_tab {
            Tab::Todos => (&mut self.todos, &self.todos_idx),
            Tab::Dones => (&mut self.dones, &self.dones_idx),
        };

        if target_vec.is_empty() {
            return;
        }

        target_vec.remove(*idx);
    }

    fn start_insert_mode(&mut self, direction: KeyCode) {
        self.mode = Mode::Insert(InsertMode::New);

        let (list, idx) = match self.curr_tab {
            Tab::Todos => (&mut self.todos, &mut self.todos_idx),
            Tab::Dones => (&mut self.dones, &mut self.dones_idx),
        };

        let insert_idx = match direction {
            KeyCode::Up => *idx,
            KeyCode::Down => *idx + 1,
            _ => unreachable!(),
        }
        .clamp(0, list.len());

        list.insert(insert_idx, String::new());
        *idx = insert_idx;
    }

    fn start_edit_mode(&mut self) {
        let Some(snap) = self.get_current_buffer().cloned() else {
            return;
        };
        self.mode = Mode::Insert(InsertMode::Edit(snap))
    }

    fn disable_insert_mode(&mut self) {
        self.mode = Mode::Normal;
    }

    fn get_current_buffer(&self) -> Option<&String> {
        match self.curr_tab {
            Tab::Todos => self.todos.get(self.todos_idx),
            Tab::Dones => self.dones.get(self.dones_idx),
        }
    }

    fn handle_insert_mode(&mut self, code: InsertAction) {
        let buf = match self.curr_tab {
            Tab::Todos => self.todos.get_mut(self.todos_idx).unwrap(),
            Tab::Dones => self.dones.get_mut(self.dones_idx).unwrap(),
        };

        match code {
            InsertAction::Enter => self.disable_insert_mode(),
            // Cancel operation and not save
            InsertAction::Cancel => {
                match mem::replace(&mut self.mode, Mode::Normal) {
                    Mode::Insert(InsertMode::Edit(snap)) => *buf = snap,
                    Mode::Insert(InsertMode::New) => self.handle_delete(),
                    _ => unreachable!(),
                };
                self.disable_insert_mode();
            }
            InsertAction::Char(c) => buf.push(c),
            InsertAction::DeleteChar => {
                if !buf.is_empty() {
                    buf.remove(buf.len() - 1);
                }
            }
        }
    }

    fn handle_move_item(&mut self, direction: KeyCode) {
        let (vec, idx) = match self.curr_tab {
            Tab::Todos => (&mut self.todos, &mut self.todos_idx),
            Tab::Dones => (&mut self.dones, &mut self.dones_idx),
        };

        if vec.is_empty() {
            return;
        }

        let idx_val = *idx;

        let new_idx = match direction {
            KeyCode::Down => {
                if idx_val == vec.len() - 1 {
                    return;
                }
                idx_val + 1
            }
            KeyCode::Up => {
                if idx_val == 0 {
                    return;
                }
                idx_val - 1
            }
            _ => unreachable!(),
        };

        vec.swap(idx_val, new_idx);
        *idx = new_idx;
    }

    fn goto_list_pos(&mut self, pos: usize) {
        let idx = match self.curr_tab {
            Tab::Todos => &mut self.todos_idx,
            Tab::Dones => &mut self.dones_idx,
        };

        // No need for bound checking due to clamping
        *idx = pos;
    }

    fn handle_help_mode(&mut self, action: Action) {
        match action {
            Action::SaveQuit | Action::NoSaveQuit => self.mode = Mode::Normal,
            _ => {}
        }
    }

    fn clamp_indexes(&mut self) {
        self.todos_idx = self.todos_idx.clamp(0, self.todos.len().saturating_sub(1));
        self.dones_idx = self.dones_idx.clamp(0, self.dones.len().saturating_sub(1));
    }
}
