use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    SaveAndQuit,
    Suspend,
    ToggleBookmarkMenu,
    ToggleTitleFooter,
    ToggleBookmarkAtCursor,
    ClearAllBookmarks,
    ToggleHelp,
    FocusLeft,
    FocusRight,
    MoveUp,
    MoveDown,
    Enter,
    AutoScroll,
    ConfirmDelete,
    CancelDelete,
    None,
}

pub fn resolve_event(ev: Event) -> Action {
    match ev {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
            KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Suspend,

            KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
            KeyCode::Char('Q') => Action::SaveAndQuit,
            KeyCode::Char('b') => Action::ToggleBookmarkMenu,
            KeyCode::Char('s') => Action::ToggleTitleFooter,
            KeyCode::Char('m') => Action::ToggleBookmarkAtCursor,
            KeyCode::Char('M') => Action::ClearAllBookmarks,
            KeyCode::Char('h') | KeyCode::Left => Action::FocusLeft,
            KeyCode::Char('l') | KeyCode::Right => Action::FocusRight,
            KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
            KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
            KeyCode::Enter => Action::Enter,
            KeyCode::Char(' ') => Action::AutoScroll,
            KeyCode::Char('?') => Action::ToggleHelp,
            KeyCode::Char('y') | KeyCode::Char('Y') => Action::ConfirmDelete,
            KeyCode::Char('n') | KeyCode::Char('N') => Action::CancelDelete,
            _ => Action::None,
        },
        _ => Action::None,
    }
}
