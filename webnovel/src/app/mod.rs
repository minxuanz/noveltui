pub mod event;
pub mod state;
pub mod update;

// 重新导出常用类型
pub use event::{AppAction, EventHandler, InputMode};
pub use state::{AppState, ContentData, InputHistory};
pub use update::{receive_updates, ContentLoader, UpdateMessage};
