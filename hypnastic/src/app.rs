use crate::editor::ActiveFileData;
use hypnagogic_core::config::read_config;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use std::{env, fs};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Hypnastic {
    pub config: HypnasticConfig,
    pub active_file: Option<ActiveFileData>,
    pub dirty: bool,
    pub show_quit_confirm: bool,
    pub allowed_to_close: bool,
    pub find_new_template_path: bool,
    pub picker_string: String,
    pub path_valid: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HypnasticConfig {
    template_path: Option<PathBuf>,
}

impl Default for Hypnastic {
    fn default() -> Self {
        Self {
            config: HypnasticConfig::default(),
            active_file: None,
            dirty: false,
            show_quit_confirm: false,
            allowed_to_close: false,
            find_new_template_path: false,
            picker_string: String::new(),
            path_valid: true,
        }
    }
}

impl Hypnastic {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let config = Self::load_config().expect("Failed to load or create config");
        // verify that the template path is valid
        let find_new_template_path = if let Some(template_path) = &config.template_path {
            if !template_path.exists() {
                true
            } else {
                false
            }
        } else {
            true
        };

        Self {
            config,
            find_new_template_path,
            ..Hypnastic::default()
        }
    }

    pub fn load_config() -> Result<HypnasticConfig, Box<dyn std::error::Error>> {
        let running_dir = env::current_dir()?;

        let config_file = running_dir.join("hypnastic_conf.toml");
        if config_file.exists() {
            let config = std::fs::read_to_string(config_file)?;
            let config: HypnasticConfig = toml::from_str(&config)?;
            Ok(config)
        } else {
            let config = HypnasticConfig::default();
            std::fs::write(config_file, toml::to_string(&config)?)?;
            Ok(config)
        }
    }

    pub fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let running_dir = env::current_dir()?;
        let config_file = running_dir.join("hypnastic_conf.toml");
        std::fs::write(config_file, toml::to_string(&self.config)?)?;
        Ok(())
    }

    pub fn create_new_file(&mut self) {}

    pub fn open_file(&mut self) {
        let found_file = rfd::FileDialog::new()
            .add_filter("Hypnogogic Config File", &["toml"])
            .pick_file();
        if let Some(file) = found_file {
            let file = fs::File::open(file).unwrap();
        }
    }

    pub fn save_file_to(&mut self, path: PathBuf) {
        if let Some(active_file) = &mut self.active_file {
            let serialized = toml::to_string(active_file).unwrap();
            fs::write(path, serialized).unwrap();
        }
    }

    pub fn save_active_file(&mut self) {
        if let Some(active_file) = &mut self.active_file {
            let path = active_file.path.clone();
            self.save_file_to(path);
        }
        self.dirty = false;
    }

    pub fn save_active_file_as(&mut self) {
        if self.active_file.is_some() {
            let found_file = rfd::FileDialog::new()
                .add_filter("Hypnogogic Config File", &["toml"])
                .save_file();
            if let Some(file) = found_file {
                self.save_file_to(file);
            }
        }
        self.dirty = false;
    }
}

impl eframe::App for Hypnastic {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New...").clicked() {
                        self.create_new_file();
                    }
                    if ui.button("Open...").clicked() {
                        self.open_file();
                    }
                    if ui
                        .add_enabled(self.active_file.is_some(), egui::Button::new("Save"))
                        .clicked()
                    {
                        self.save_active_file();
                    }
                    if ui.button("Save As...").clicked() {
                        self.save_active_file_as();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::warn_if_debug_build(ui);
            if let Some(active_file) = &mut self.active_file {
            } else {
                ui.label("You have not yet opened a file");
                if ui.button("Create New File").clicked() {
                    self.create_new_file();
                }
                if ui.button("Open File").clicked() {
                    self.open_file();
                }
            }
        });

        if self.find_new_template_path {
            egui::Window::new("Template Path").show(ctx, |ui| {
                ui.label("The template path you have set in your config file is invalid, or you have no set a path. Please select a new template path.");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.picker_string);
                    if ui.button("browse...").clicked() {
                        if let Some(template_path) = rfd::FileDialog::new()
                            .add_filter("Hypnogogic Template File", &["toml"])
                            .pick_folder()
                        {
                            self.picker_string = template_path.to_string_lossy().to_string();
                        }
                    }
                });
                if !self.path_valid {
                    ui.label("The path you have selected is invalid. Please select a valid path.");
                }
                if ui.button("Confirm").clicked() {
                    if !PathBuf::from(self.picker_string.clone()).exists() {
                        self.path_valid = false;
                    } else {
                        self.path_valid = true;
                        self.config.template_path = Some(PathBuf::from(self.picker_string.clone()));
                        self.find_new_template_path = false;
                    }
                }
            });
        }

        if self.show_quit_confirm {
            egui::Window::new("Save changes before quitting?").show(ctx, |ui| {
                ui.label("You currently have unsaved changes to the active file. Would you like to save them before quitting?");
                ui.horizontal(|ui| {
                    if ui.button("Yes").clicked() {
                        self.allowed_to_close = true;
                        self.show_quit_confirm = false;
                        self.save_active_file();
                        _frame.close();
                    }
                    if ui.button("No").clicked() {
                        self.allowed_to_close = true;
                        self.show_quit_confirm = false;
                        _frame.close();
                    }
                    if ui.button("Cancel").clicked() {
                        self.allowed_to_close = false;
                        self.show_quit_confirm = false;
                    }
                });
            });
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn on_close_event(&mut self) -> bool {
        self.save_config().unwrap();
        if self.active_file.is_none() || (self.dirty && self.allowed_to_close) {
            true
        } else {
            self.show_quit_confirm = true;
            false
        }
    }
}
