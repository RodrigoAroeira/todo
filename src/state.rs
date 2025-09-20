use crossterm::event::KeyCode;

use crate::globals;

#[derive(Clone, Copy)]
pub enum Tab {
    Todos,
    Dones,
}

impl Tab {
    fn toggle(self) -> Self {
        match self {
            Tab::Todos => Tab::Dones,
            Tab::Dones => Tab::Todos,
        }
    }
}

#[rustfmt::skip]
pub struct StateHandler<'a> {
    pub curr_tab:    &'a mut Tab,
    pub todos:       &'a mut Vec<String>,
    pub todos_idx:   &'a mut usize,
    pub dones:       &'a mut Vec<String>,
    pub dones_idx:   &'a mut usize,
    pub insert_mode: &'a mut bool,
}

impl StateHandler<'_> {
    #[rustfmt::skip]
    pub fn execute_action(&mut self, code: KeyCode) -> anyhow::Result<()> {
        if *self.insert_mode {
            self.handle_insert_mode(code);
            return Ok(())
        }
        match code {
            KeyCode::Enter     => self.handle_enter_press(),
            KeyCode::Tab       => *self.curr_tab = self.curr_tab.toggle(),
            KeyCode::Char('i') => self.start_insert_mode(),
            KeyCode::Char('j') => self.handle_cursor_move(KeyCode::Down),
            KeyCode::Char('k') => self.handle_cursor_move(KeyCode::Up),
            KeyCode::Char('J') => self.handle_move_item(KeyCode::Down),
            KeyCode::Char('K') => self.handle_move_item(KeyCode::Up),
            KeyCode::Char('d') => self.handle_delete(),
            KeyCode::Char('q') => anyhow::bail!(globals::BREAK),
            KeyCode::Char('Q') => anyhow::bail!(globals::NO_SAVE),
            _ => return Ok(()),
        }

        self.clamp_indexes();
        Ok(())
    }

    fn handle_enter_press(&mut self) {
        match self.curr_tab {
            Tab::Todos => {
                if self.todos.is_empty() {
                    return;
                }
                let value = self.todos.remove(*self.todos_idx);
                self.dones.push(value);
            }
            Tab::Dones => {
                if self.dones.is_empty() {
                    return;
                }
                let value = self.dones.remove(*self.dones_idx);
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
            KeyCode::Down => **idx += 1,
            KeyCode::Up => **idx = idx.saturating_sub(1),
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

        target_vec.remove(**idx);
    }

    fn start_insert_mode(&mut self) {
        *self.insert_mode = true;
        match self.curr_tab {
            Tab::Todos => self.todos.insert(*self.todos_idx, String::new()),
            Tab::Dones => self.dones.insert(*self.dones_idx, String::new()),
        }
    }

    fn handle_insert_mode(&mut self, code: KeyCode) {
        let buf = match self.curr_tab {
            Tab::Todos => &mut self.todos.get_mut(*self.todos_idx).unwrap(),
            Tab::Dones => &mut self.dones.get_mut(*self.dones_idx).unwrap(),
        };

        match code {
            KeyCode::Enter => *self.insert_mode = false,
            // Cancel operation and not save
            KeyCode::Esc => {
                self.handle_delete();
                *self.insert_mode = false;
            }
            KeyCode::Char(c) => buf.push(c),
            KeyCode::Backspace => {
                if !buf.is_empty() {
                    buf.remove(buf.len() - 1);
                }
            }
            _ => {}
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

        let idx_val = **idx; // dereference once for clarity

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
        **idx = new_idx;
    }

    fn clamp_indexes(&mut self) {
        *self.todos_idx = (*self.todos_idx).clamp(0, self.todos.len().saturating_sub(1));
        *self.dones_idx = (*self.dones_idx).clamp(0, self.dones.len().saturating_sub(1));
    }
}
