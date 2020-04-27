use std::sync::mpsc::{sync_channel, SyncSender};

pub struct Task {
    pub msg: String,
    pub sender: SyncSender<Result<String, Error>>,
}

impl Task {
    pub fn new(msg: String) -> Self {
        let (sender, _receiver) = sync_channel::<Result<String, Error>>(2);
        Task {
            msg: msg,
            sender: sender,
        }
    }
}
