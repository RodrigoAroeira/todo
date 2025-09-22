use crossterm::event::KeyCode;

pub enum Action {
    Enter,
    SwitchTab,
    Insert(KeyCode),
    Edit,
    MoveCursor(KeyCode),
    MoveItem(KeyCode),
    GotoBegin,
    GotoEnd,
    Delete,
    SaveQuit,
    NoSaveQuit,
}

impl TryFrom<KeyCode> for Action {
    type Error = ();

    fn try_from(value: KeyCode) -> Result<Self, Self::Error> {
        use KeyCode::{Down, Up};
        Ok(match value {
            KeyCode::Enter => Self::Enter,
            KeyCode::Tab => Self::SwitchTab,
            KeyCode::Char('i') => Self::Insert(Up),
            KeyCode::Char('o') => Self::Insert(Down),
            KeyCode::Char('e') => Self::Edit,
            KeyCode::Char('k') => Self::MoveCursor(Up),
            KeyCode::Char('j') => Self::MoveCursor(Down),
            KeyCode::Char('J') => Self::MoveItem(Down),
            KeyCode::Char('K') => Self::MoveItem(Up),
            KeyCode::Char('g') => Self::GotoBegin,
            KeyCode::Char('G') => Self::GotoEnd,
            KeyCode::Char('d') => Self::Delete,
            KeyCode::Char('q') => Self::SaveQuit,
            KeyCode::Char('Q') => Self::NoSaveQuit,
            _ => return Err(()),
        })
    }
}

pub enum InsertAction {
    Char(char),
    DeleteChar,
    Enter,
    Cancel,
}

impl TryFrom<KeyCode> for InsertAction {
    type Error = ();

    fn try_from(value: KeyCode) -> Result<Self, Self::Error> {
        use KeyCode::{Backspace, Char, Enter, Esc};

        Ok(match value {
            Char(c) => Self::Char(c),
            Backspace => Self::DeleteChar,
            Enter => Self::Enter,
            Esc => Self::Cancel,
            _ => return Err(()),
        })
    }
}
