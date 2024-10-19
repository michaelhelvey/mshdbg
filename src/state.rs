use std::path::PathBuf;

use crate::filetree;

pub enum Message {
    PushFileTree {
        at_path: PathBuf,
        entries: Vec<filetree::FileTreeEntry>,
    },
}
