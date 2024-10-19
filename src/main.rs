use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
};

use color_eyre::{eyre::eyre, Result};
use eframe::egui;
use filetree::fs_tree;
use tracing::{debug, error, info};

mod constants;
mod filetree;
mod state;
mod utils;

const APP_NAME: &str = "MSH DBG";

struct App {
    file_tree: filetree::FileTreeState,
    message_producer: Sender<state::Message>,
    message_receiver: Receiver<state::Message>,
}

impl App {
    fn new(cwd: PathBuf) -> Self {
        let (tx, rx) = mpsc::channel::<state::Message>();
        let app = Self {
            message_producer: tx.clone(),
            message_receiver: rx,
            file_tree: filetree::FileTreeState::new(cwd),
        };

        // Kick off our initial load of the file tree
        app.file_tree.load_entries(tx);

        app
    }

    fn update_state(&mut self, msg: state::Message) {
        match msg {
            state::Message::PushFileTree { at_path, entries } => {
                self.file_tree.insert_entries_at_path(at_path, entries);
            }
        };
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // check our messages queue and process anything that's arrived since the last frame
        while let Ok(msg) = self.message_receiver.try_recv() {
            // do something with msg
            self.update_state(msg);
        }

        // render the next frame based on the current state
        let frame = egui::Frame::default()
            .inner_margin(egui::Margin::symmetric(0.0, 8.0))
            .fill(constants::PANEL_BG);

        egui::SidePanel::left("file_tree")
            .max_width(400.0)
            .default_width(300.0)
            .frame(frame)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::TOP), |ui| {
                    fs_tree(ui, &mut self.file_tree, self.message_producer.clone(), 1);
                });
            });
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    utils::init_tracing()?;

    debug!("initializing eframe application");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };

    // Eventually we want to fetch this from command line arguments
    let cwd = std::env::current_dir().unwrap();

    match eframe::run_native(APP_NAME, options, Box::new(|_| Ok(Box::new(App::new(cwd))))) {
        Ok(_) => {
            info!("application exited successfully");
            Ok(())
        }
        Err(e) => {
            error!("application exited with error: {e}");
            Err(eyre!("application exited with error: {e}"))
        }
    }
}
