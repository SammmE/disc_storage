#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod compress;
mod config;

use std::path::PathBuf;

use config::{get_config, Config};
use eframe::egui;
use log::info;
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
}

impl Default for DiscStorage {
    fn default() -> Self {
        Self {
            config: get_config(),
            files: vec![],
            allowed_to_close: true,
            show_confirmation_dialog: false,
            name: String::new(),
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
                if ui.button("Get files to discord storage").clicked() {
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
                mode_store(self, ui);
            } else {
                mode_retrieve(self, ui);
            }
        });
    }
}

fn mode_store(state: &mut DiscStorage, ui: &mut egui::Ui) {
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

    if ui
        .button("Start")
        .on_hover_text("Start the process")
        .clicked()
    {
        info!("Compressing files");
        state.allowed_to_close = false;
    }
}

fn mode_retrieve(state: &mut DiscStorage, ui: &mut egui::Ui) {
    let storage = state.config.storage.clone();
    for (i, storage) in storage.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("Storage {}", i + 1));
            ui.label(storage.name.clone());
            if ui.button("Retrieve").clicked() {
                info!("Retrieving files");
                state.allowed_to_close = false;
            }
        });
    }
}
