use egui::{Button, Color32, Ui};
use std::fs;
use std::path::PathBuf;

pub struct FileExplorer {
    current_dir: PathBuf,
    selected_file: PathBuf,
    dir_vec: Vec<PathBuf>,
    file_vec: Vec<PathBuf>,
    dirnames: Vec<String>,
    filenames: Vec<String>,
}

impl FileExplorer {
    pub fn new(_: &eframe::CreationContext) -> Self {
        let mut fe = Self {
            current_dir: std::env::current_dir().unwrap(),
            selected_file: std::path::PathBuf::new(),
            dir_vec: Vec::new(),
            file_vec: Vec::new(),
            dirnames: Vec::new(),
            filenames: Vec::new(),
        };
        fe.update_paths();
        fe
    }

    pub fn update_paths(&mut self) {
        self.dir_vec.clear();
        self.file_vec.clear();
        self.filenames.clear();
        self.dirnames.clear();

        let dir_content = std::fs::read_dir(&self.current_dir)
            .unwrap()
            .map(|dir| dir.unwrap().path())
            .collect::<Vec<PathBuf>>();

        for path in dir_content.into_iter() {
            match path {
                _ if path.is_dir() => {
                    self.dir_vec.push(path.clone());
                    let dirname = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    self.dirnames.push(dirname);
                }
                _ if path.is_file() => {
                    self.file_vec.push(path.clone());
                    let filename = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    self.filenames.push(filename);
                }
                _ => (),
            }
        }
    }

    pub fn get_filename(&self) -> String {
        self.selected_file
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }

    pub fn add_dir_ui<G>(ui: &mut Ui, label: &str, r_size: f32, f: G)
    where
        G: FnOnce(),
    {
        // let size = []
        if ui
            // .add_sized(ui.available_size(), Button::new(label))
            // .clicked()
            .button(label)
            .clicked()
        {
            f()
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        // Affichez le chemin actuel en tant qu'en-tête.
        ui.horizontal(|ui| {
            ui.label("Current Path:");
            ui.monospace(self.current_dir.display().to_string());
        });
        ui.horizontal(|ui| {
            FileExplorer::add_dir_ui(ui, "<<< Previous", 1., || {
                self.current_dir.pop();
                self.update_paths();
            });
            FileExplorer::add_dir_ui(ui, "Update", 1., || {
                self.update_paths();
            });
        });

        let mut should_update = false;
        for (dirname, dir_path) in self.dirnames.iter().zip(self.dir_vec.iter()) {
            if ui.button(dirname).clicked() {
                self.current_dir = dir_path.clone();
                should_update = true;
            };
        }
        if should_update {
            self.update_paths();
        }

        for (filename, file_path) in self.filenames.iter().zip(self.file_vec.iter()) {
            if ui.selectable_label(false, filename).clicked() {
                self.selected_file = file_path.clone();
            };
        }
        ui.colored_label(Color32::LIGHT_BLUE, self.get_filename());
    }
}
