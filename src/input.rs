use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// 约定：这里不直接操作 App 状态，只做“按键 -> 语义动作”的映射。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    Suspend,
    ToggleBookmarkMenu,
    ToggleTitleFooter,
    ToggleBookmarkAtCursor,
    FocusLeft,
    FocusRight,
    MoveUp,
    MoveDown,
    Enter,
    None,
}

pub fn action_from_event(ev: Event) -> Action {
    match ev {
        // need to check for KeyEventKind::Press
        Event::Key(key) if key.kind == KeyEventKind::Press => action_from_key_event(key),
        _ => Action::None,
    }
}

fn action_from_key_event(key: KeyEvent) -> Action {
    // ctrl-c
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Action::Quit;
    }

    // ctrl-z
    if key.code == KeyCode::Char('z') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Action::Suspend;
    }

    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('b') => Action::ToggleBookmarkMenu,
        KeyCode::Char('s') => Action::ToggleTitleFooter,
        KeyCode::Char('m') => Action::ToggleBookmarkAtCursor,
        KeyCode::Char('h') | KeyCode::Left => Action::FocusLeft,
        KeyCode::Char('l') | KeyCode::Right => Action::FocusRight,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
        KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
        KeyCode::Enter => Action::Enter,
        _ => Action::None,
    }
}
