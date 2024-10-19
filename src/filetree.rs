use eframe::egui::{self, Widget};
use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
};
use tracing::{debug, error};

use crate::{constants, state};

pub enum DirFetchState {
    NotStarted,
    Pending,
    Complete,
}

pub enum FileTreeEntry {
    File {
        entry: fs::DirEntry,
    },
    Dir {
        entry: fs::DirEntry,
        fetch_state: DirFetchState,
        toggled: bool,
        next: Option<Box<FileTreeState>>,
    },
}

impl PartialEq for FileTreeEntry {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (FileTreeEntry::File { entry: e1 }, FileTreeEntry::File { entry: e2 }) => {
                e1.path() == e2.path()
            }
            (FileTreeEntry::Dir { entry: e1, .. }, FileTreeEntry::Dir { entry: e2, .. }) => {
                e1.path() == e2.path()
            }
            _ => false,
        }
    }
}

impl Eq for FileTreeEntry {}

impl PartialOrd for FileTreeEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (FileTreeEntry::File { entry: e1 }, FileTreeEntry::File { entry: e2 }) => e1
                .file_name()
                .to_string_lossy()
                .to_lowercase()
                .partial_cmp(&e2.file_name().to_string_lossy().to_lowercase()),
            (FileTreeEntry::Dir { entry: e1, .. }, FileTreeEntry::Dir { entry: e2, .. }) => e1
                .file_name()
                .to_string_lossy()
                .to_lowercase()
                .partial_cmp(&e2.file_name().to_string_lossy().to_lowercase()),
            (FileTreeEntry::File { .. }, FileTreeEntry::Dir { .. }) => {
                Some(std::cmp::Ordering::Greater)
            }
            (FileTreeEntry::Dir { .. }, FileTreeEntry::File { .. }) => {
                Some(std::cmp::Ordering::Less)
            }
        }
    }
}

impl Ord for FileTreeEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl FileTreeEntry {
    pub fn file_name(&self) -> String {
        match &self {
            FileTreeEntry::File { entry } => entry.file_name().to_string_lossy().to_string(),
            FileTreeEntry::Dir { entry, .. } => entry.file_name().to_string_lossy().to_string(),
        }
    }

    pub fn on_click(&mut self, tx: mpsc::Sender<state::Message>) {
        match self {
            FileTreeEntry::File { entry } => {
                debug!(
                    "file clicked {:?}; todo: show file in text view",
                    entry.path()
                );
            }
            FileTreeEntry::Dir {
                entry,
                fetch_state,
                toggled,
                ..
            } => {
                match fetch_state {
                    // don't do anything, because another operation is already going on
                    DirFetchState::Pending => {}
                    DirFetchState::Complete => {
                        // swap toggled state, but nothing else to fetch
                        // TODO: look into refetching
                        *toggled = !*toggled;
                    }
                    DirFetchState::NotStarted => {
                        // we need to fetch for the first time
                        *toggled = !*toggled;
                        *fetch_state = DirFetchState::Pending;

                        let path = entry.path().clone();
                        thread::spawn(move || match load_entries_at_path(&path) {
                            Ok(entries) => {
                                tx.send(state::Message::PushFileTree {
                                    at_path: path,
                                    entries,
                                })
                                .expect("app message receiver is gone");
                            }
                            Err(e) => {
                                error!("error loading directory entries: {e:?}");
                            }
                        });
                    }
                }
            }
        };
    }
}

pub struct FileTreeState {
    pub root: PathBuf,
    pub entries: Vec<FileTreeEntry>,
}

fn load_entries_at_path<P: AsRef<Path>>(path: P) -> io::Result<Vec<FileTreeEntry>> {
    let mut entries = fs::read_dir(path)?
        .map(|dir_entry| {
            dir_entry.map(|dir_entry| {
                if dir_entry.path().is_dir() {
                    FileTreeEntry::Dir {
                        entry: dir_entry,
                        fetch_state: DirFetchState::NotStarted,
                        toggled: false,
                        next: None,
                    }
                } else {
                    FileTreeEntry::File { entry: dir_entry }
                }
            })
        })
        .collect::<io::Result<Vec<FileTreeEntry>>>()?;

    entries.sort();

    Ok(entries)
}

impl FileTreeState {
    pub fn new(cwd: PathBuf) -> Self {
        Self {
            entries: vec![],
            root: cwd,
        }
    }

    pub fn load_entries(&self, tx: mpsc::Sender<state::Message>) {
        let root = self.root.clone();
        thread::spawn(move || match load_entries_at_path(&root) {
            Ok(entries) => {
                tx.send(state::Message::PushFileTree {
                    at_path: root,
                    entries,
                })
                .expect("app message receiver is gone");
            }
            Err(e) => {
                error!("error loading directory entries: {e:?}");
            }
        });
    }

    pub fn insert_entries_at_path(&mut self, path: PathBuf, entries: Vec<FileTreeEntry>) {
        // first check if these are our entries
        if path == self.root {
            self.entries = entries;
            return;
        }

        // otherwise they are for one of our (grand++)children, so we need to recurse
        for entry in self.entries.iter_mut() {
            if let FileTreeEntry::Dir {
                entry,
                next,
                fetch_state,
                ..
            } = entry
            {
                if path.eq(&entry.path()) {
                    *fetch_state = DirFetchState::Complete;
                    // we found the entry so we can set the new children and be done
                    *next = Some(Box::new(FileTreeState {
                        root: path,
                        entries,
                    }));
                    return;
                }

                if let (Some(tree), true) = (next, path.starts_with(entry.path())) {
                    return tree.insert_entries_at_path(path, entries);
                }
            }
        }
    }
}

fn file_button(ui: &mut egui::Ui, text: &str, left_padding: f32) -> egui::Response {
    let margin = egui::Margin {
        bottom: 1.0,
        top: 1.0,
        left: left_padding,
        right: 20.0,
    };
    let mut frame = egui::Frame::default().inner_margin(margin).begin(ui);

    let button = egui::Button::new(text)
        .frame(false)
        .wrap_mode(egui::TextWrapMode::Truncate);
    let response = button.ui(&mut frame.content_ui);

    let frame_response = frame.allocate_space(ui);
    if frame_response.hovered() {
        frame.frame.fill = constants::PANEL_BG_HOVER;
    }

    frame.paint(ui);

    response
}

pub fn fs_tree(
    ui: &mut egui::Ui,
    tree: &mut FileTreeState,
    tx: mpsc::Sender<state::Message>,
    depth: u16,
) {
    for fs_entry in &mut tree.entries {
        let button = file_button(ui, fs_entry.file_name().as_str(), f32::from(20 * depth));

        if button.clicked() {
            fs_entry.on_click(tx.clone())
        }

        match fs_entry {
            FileTreeEntry::File { .. } => {}
            FileTreeEntry::Dir { next, toggled, .. } => {
                let should_show_children = *toggled
                    && next
                        .as_ref()
                        .and_then(|state| match state.entries.is_empty() {
                            true => None,
                            false => Some(true),
                        })
                        .unwrap_or(false);

                if should_show_children {
                    let next_tree = next.as_mut().unwrap();
                    fs_tree(ui, next_tree, tx.clone(), depth + 1);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_path_membership() {
        let root = PathBuf::from("/home/user/dev/project");
        let path = PathBuf::from("/home/user/dev/project/examples/basic/foo");

        let is_parent = path.starts_with(root);
        assert!(is_parent);
    }
}
