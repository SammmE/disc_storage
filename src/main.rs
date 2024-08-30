#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod compress;
mod config;
mod discord;

use std::{path::PathBuf, sync::Arc};

use std::thread;

use config::{get_config, Config};
use eframe::egui;
use egui::mutex::Mutex;
use log::{debug, info};
use rfd::FileDialog;

use compress::CompressType;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 340.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Discord Storage",
        options,
        Box::new(|_cc| Ok(Box::<DiscStorage>::default())),
    )
}

struct DiscStorage {
    config: Config,
    files: Vec<PathBuf>,
    allowed_to_close: bool,
    show_confirmation_dialog: bool,
    name: String,
    main_progress: Arc<Mutex<f32>>,
    sub_progress: Arc<Mutex<f32>>,
    thread_handle: Option<thread::JoinHandle<()>>,
    remove_files_confirm: bool,
    remove_file: i32,
}

impl Default for DiscStorage {
    fn default() -> Self {
        Self {
            config: get_config(),
            files: vec![],
            allowed_to_close: true,
            show_confirmation_dialog: false,
            name: String::new(),
            main_progress: Arc::new(Mutex::new(0.0)),
            sub_progress: Arc::new(Mutex::new(0.0)),
            thread_handle: None,
            remove_files_confirm: false,
            remove_file: 0,
        }
    }
}

impl eframe::App for DiscStorage {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ctx.input(|i| i.viewport().close_requested()) {
                if self.allowed_to_close {
                    config::set_config(self.config.clone());
                    info!("Config saved");
                } else {
                    ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                    self.show_confirmation_dialog = true;
                }
            }

            if self.show_confirmation_dialog {
                egui::Window::new("Are you sure you want to quit?")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("No").clicked() {
                                self.show_confirmation_dialog = false;
                                self.allowed_to_close = false;
                            }

                            if ui.button("Yes").clicked() {
                                self.show_confirmation_dialog = false;
                                self.allowed_to_close = true;
                                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });
                    });
            }

            ui.horizontal(|ui| {
                ui.heading("Discord Storage");
                if ui.button("Add files to discord storage").clicked() {
                    self.config.mode = config::Mode::Store;
                }
                if ui.button("Get files from discord storage").clicked() {
                    self.config.mode = config::Mode::Retrieve;
                }
            });
            ui.separator();
            let name_label = ui.label("Your Discord bot token: ");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.config.token)
                    .labelled_by(name_label.id);
                if ui.button("Save").on_hover_text("Save the token").clicked() {
                    config::set_config(self.config.clone());
                    info!("Config saved");
                }
            });
            ui.separator();

            if self.config.mode == config::Mode::Store {
                mode_store(self, ui, ctx);
            } else {
                mode_retrieve(self, ui, ctx);
            }

            if !self.allowed_to_close {
                ui.add(egui::ProgressBar::new(*self.main_progress.lock()).text("Main Progress"));
                ui.add(egui::ProgressBar::new(*self.sub_progress.lock()).text("Main Progress"));
            }
        });
    }
}

fn mode_store(state: &mut DiscStorage, ui: &mut egui::Ui, _ctx: &egui::Context) {
    ui.separator();
    ui.horizontal(|ui| {
        ui.label("Name: ");
        ui.text_edit_singleline(&mut state.name);
    });
    ui.separator();
    ui.horizontal(|ui| {
        ui.heading("Sources");
        ui.label("Add files/folders to your Discord bot's storage");
        if ui.button("+ file").on_hover_text("Add files").clicked() {
            let files = FileDialog::new().pick_files();
            if let Some(files) = files {
                state.files.extend(files);
            }
        }
        if ui.button("+ folder").on_hover_text("Add folders").clicked() {
            let files = FileDialog::new().pick_folders();
            if let Some(files) = files {
                state.files.extend(files);
            }
        }
        if ui
            .button("Clear")
            .on_hover_text("Clear files/folders")
            .clicked()
        {
            state.files.clear();
        }
    });

    let mut to_remove = Vec::new();
    for (i, file) in state.files.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.label(file.to_string_lossy());
            if ui
                .button("Remove")
                .on_hover_text("Remove file/folder")
                .clicked()
            {
                to_remove.push(i);
            }
        });
    }

    // Remove items from the end to start to prevent shifting issues
    to_remove.sort_by(|a, b| b.cmp(a));
    for index in to_remove {
        state.files.remove(index);
    }

    ui.separator();

    ui.heading("Compression options");
    egui::ComboBox::from_label("Compression")
        .selected_text(format!("{:?}", state.config.compress_type))
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut state.config.compress_type,
                CompressType::LZMA,
                "Lzma (smaller compression)",
            );
            ui.selectable_value(
                &mut state.config.compress_type,
                CompressType::Zstd,
                "Zstd (faster compression)",
            )
        });
    ui.add(egui::Slider::new(&mut state.config.compression_level, 0..=9).text("Compression level"));
    ui.separator();
    if state.name.is_empty()
        || state.files.is_empty()
        || state.config.token.is_empty()
        || state.files.is_empty()
    {
        ui.label(egui::RichText::new("Please fill all fields").color(egui::Color32::RED));
    }

    if ui
        .button("Start")
        .on_hover_text("Start the process")
        .clicked()
    {
        // make sure all fields are filled
        if !state.name.is_empty()
            || !state.files.is_empty()
            || !state.config.token.is_empty()
            || !state.files.is_empty()
        {
            info!("Compressing files");
            state.allowed_to_close = false;

            info!("Added files to config");
            state.config.storage.push(config::Storage {
                name: state.name.clone(),
                files: state.files.clone(),
            });

            // RULES FOR PROGRESS BARS:
            // 1. Main progress bar should be the overall progress. For storing, there are 3 steps: compressing, encrypting, re-compressing, and uploading.
            // 2. Sub progress bar should be the progress of the current step. These will be handled by the compressors, encryptors, encoders or uploaders.
            // 3. The progress bars should be updated in the main thread, but the actual work should be done in a separate thread.

            if state.thread_handle.is_none() {
                let mp = state.main_progress.clone();
                state.thread_handle = Some(thread::spawn(move || {
                    // placeholder for progress bars
                    loop {
                        *mp.lock() += 0.1;
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        debug!("Main progress: {}", *mp.lock());
                    }
                }));
            }

            // Get the latest progress values

            // ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}

fn mode_retrieve(state: &mut DiscStorage, ui: &mut egui::Ui, _ctx: &egui::Context) {
    if state.config.storage.is_empty() {
        ui.label("No storage found");
        return;
    } else if state.remove_files_confirm == true && state.remove_file == -1 {
        ui.label("Are you sure you want to delete the files?");
        ui.horizontal(|ui| {
            if ui.button("No").clicked() {
                state.remove_file = -1;
                state.remove_files_confirm = false;
            }
            if ui.button("Yes").clicked() {
                state.remove_files_confirm = false;
            }
        });
    } else if state.remove_files_confirm == false && state.remove_file != -1 {
        // remove the file
        state.config.storage[state.remove_file as usize]
            .files
            .clear();

        state.remove_file = -1;
    } else {
        let storage = state.config.storage.clone();
        egui::ScrollArea::vertical()
            .max_width(f32::INFINITY)
            .show(ui, |ui| {
                for (i, storage) in storage.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("Storage {}", i + 1));
                        ui.label(storage.name.clone());
                        if ui.button("Retrieve").clicked() {
                            info!("Retrieving files");
                            state.allowed_to_close = false;
                        }
                        if ui.button("Delete").clicked() {
                            state.config.storage.remove(i);
                            config::set_config(state.config.clone());
                        }
                    });
                }
            });
    }
}
