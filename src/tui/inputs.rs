use crate::tui::state::AppState;
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    SaveAndQuit,
    Suspend,
    ToggleBookmarkMenu,
    ToggleTocMenu,
    ToggleTitleFooter,
    ToggleBookmarkAtCursor,
    ClearAllBookmarks,
    ToggleHelp,
    ToggleLineSpace,
    MoveUp,
    MoveDown,
    NextChapter,
    PrevChapter,
    Enter,
    AutoScroll,
    ConfirmDelete,
    CancelDelete,
    PageUp,
    PageDown,
    None,
}

pub fn resolve_event(ev: Event, state: &AppState) -> Action {
    match ev {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
            KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Suspend,

            KeyCode::Char('q') | KeyCode::Esc => Action::SaveAndQuit,
            KeyCode::Char('Q') => Action::Quit,
            KeyCode::Char('b') => Action::ToggleBookmarkMenu,
            KeyCode::Char('t') => Action::ToggleTocMenu,
            KeyCode::Char('s') => Action::ToggleTitleFooter,
            KeyCode::Char('n') | KeyCode::Char('N') if state.show_delete_confirmation => {
                Action::CancelDelete
            }
            KeyCode::Char('n') => Action::NextChapter,
            KeyCode::Char('p') => Action::PrevChapter,
            KeyCode::Char('m') => Action::ToggleBookmarkAtCursor,
            KeyCode::Char('M') => Action::ClearAllBookmarks,
            KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
            KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
            KeyCode::Char('l') => Action::ToggleLineSpace,
            KeyCode::Enter => Action::Enter,
            KeyCode::Char(' ') => Action::AutoScroll,
            KeyCode::Char('?') => Action::ToggleHelp,
            KeyCode::Char('y') | KeyCode::Char('Y') if state.show_delete_confirmation => {
                Action::ConfirmDelete
            }
            KeyCode::PageUp => Action::PageUp,
            KeyCode::PageDown => Action::PageDown,
            _ => Action::None,
        },
        _ => Action::None,
    }
}
