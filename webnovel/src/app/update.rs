use crate::app::state::ContentData;
use crate::net::crawler::fetch_novel;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

/// 应用状态更新消息
#[derive(Debug)]
pub enum UpdateMessage {
    Content(ContentData),
    Error(String),
}

/// 数据加载器
pub struct ContentLoader {
    sender: Sender<UpdateMessage>,
}

impl ContentLoader {
    pub fn new() -> (Self, Receiver<UpdateMessage>) {
        let (sender, receiver) = channel();
        (Self { sender }, receiver)
    }

    pub fn load(&self, url: String) {
        let tx = self.sender.clone();
        thread::spawn(move || {
            let result = match fetch_novel(&url) {
                Ok(page) => UpdateMessage::Content(ContentData::new(page.content, page.title)),
                Err(e) => UpdateMessage::Error(format!("Error: {}", e)),
            };
            let _ = tx.send(result);
        });
    }

    pub fn send_loading(&self, url: &str) {
        let _ = self
            .sender
            .send(UpdateMessage::Content(ContentData::loading(url)));
    }

    pub fn send_welcome(&self) {
        let _ = self
            .sender
            .send(UpdateMessage::Content(ContentData::welcome()));
    }
}

/// 从接收器获取最新内容
pub fn receive_updates(receiver: &Receiver<UpdateMessage>) -> Option<ContentData> {
    let mut last_result = None;

    // 获取所有待处理的消息，只保留最新的
    while let Ok(message) = receiver.try_recv() {
        match message {
            UpdateMessage::Content(data) => {
                last_result = Some(data);
            }
            UpdateMessage::Error(msg) => {
                last_result = Some(ContentData::error(msg));
            }
        }
    }

    last_result
}
