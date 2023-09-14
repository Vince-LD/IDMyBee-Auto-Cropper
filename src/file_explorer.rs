use anyhow::Result;

use egui::{Button, Color32, Label, RichText, ScrollArea, Ui};
use std::cmp::min;
use std::ffi::OsStr;
use std::fs::{self, DirEntry};
use std::path::PathBuf;
use std::rc::Rc;

pub struct FileExplorer<'a> {
    pub current_dir: PathBuf,
    pub split_current_dir: Vec<String>,
    pub selected_file: Option<PathBuf>,
    pub selected_file_index: Option<usize>,
    dir_vec: Vec<PathBuf>,
    file_vec: Vec<PathBuf>,
    dirnames: Vec<String>,
    filenames: Vec<String>,
    pub allowed_extensions: Vec<&'a str>,
    pub err: Result<()>,
}

impl FileExplorer<'_> {
    pub fn new() -> Self {
        let mut fe = Self {
            current_dir: std::env::current_dir().unwrap(),
            split_current_dir: Vec::new(),
            selected_file: None,
            selected_file_index: None,
            dir_vec: Vec::new(),
            file_vec: Vec::new(),
            dirnames: Vec::new(),
            filenames: Vec::new(),
            allowed_extensions: vec!["png", "jpg", "jpeg", "tiff"],
            err: Ok(()),
        };
        fe.change_dir(&fe.current_dir.clone());
        // fe.set_split_current_dir();
        fe
    }

    pub fn update_paths(&mut self) {
        self.selected_file_index = None;
        let dir_content = match std::fs::read_dir(&self.current_dir) {
            Ok(content) => content
                .filter_map(|dir| dir.ok())
                .collect::<Vec<DirEntry>>()
                .iter()
                .map(|dir| dir.path())
                .collect::<Vec<PathBuf>>(),
            Err(err) => {
                self.err = Err(err.into());
                return;
            }
        };

        self.dir_vec.clear();
        self.file_vec.clear();
        self.filenames.clear();
        self.dirnames.clear();

        for path in dir_content.into_iter() {
            match path {
                _ if path.is_dir() => {
                    if let Some(dirname) = path.file_name() {
                        self.dir_vec.push(path.clone());
                        let str_dirname = dirname.to_string_lossy().to_string();
                        self.dirnames.push(str_dirname);
                    }
                }
                _ if path.is_file() && imghdr::from_file(&path).is_ok() => {
                    if let Some(filename) = path.file_name() {
                        println!("{:?} is a picture", filename);
                        self.file_vec.push(path.clone());
                        let str_filename = filename.to_string_lossy().to_string();
                        self.filenames.push(str_filename);
                        //  self.allowed_extensions.contains(&filename.to_str().unwrap_or("")) {
                    }
                }
                _ => (),
            }
        }
        self.err = Ok(())
    }

    pub fn get_filename(&self) -> Option<String> {
        // Si Some alors on transforme en Some(str) sinon on renvoie None
        self.selected_file.as_ref().map(|path| {
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        })
    }

    pub fn get_filepath(&self) -> Option<String> {
        // Si Some alors on transforme en Some(str) sinon on renvoie None
        self.selected_file
            .as_ref()
            .map(|path| path.display().to_string())
    }

    pub fn previous_file(&mut self) {
        if !self.file_vec.is_empty() {
            let new_index = match self.selected_file_index {
                Some(index) => index.saturating_sub(1),
                None => 0,
            };
            self.selected_file_index = Some(new_index);
            let new_selected_file = match self.selected_file {
                Some(_) => self.file_vec[new_index].clone(),
                None => self.file_vec[0].clone(),
            };
            self.selected_file = Some(new_selected_file)
        }
    }

    pub fn next_file(&mut self) {
        if !self.file_vec.is_empty() {
            let new_index = match self.selected_file_index {
                Some(index) => min(self.file_vec.len() - 1, index + 1),
                None => 0,
            };
            self.selected_file_index = Some(new_index);
            let new_selected_file = match self.selected_file {
                Some(_) => self.file_vec[new_index].clone(),
                None => self.file_vec[0].clone(),
            };
            self.selected_file = Some(new_selected_file)
        }
    }

    #[cfg(target_os = "windows")]
    pub fn set_split_current_dir(&mut self) {
        self.split_current_dir.clear();
        self.split_current_dir.push(String::from("C:/"));
        for last_dir in self.current_dir.iter().skip(2) {
            let dir = last_dir.to_string_lossy().to_string();
            println!("Splitting directory {:?}", dir);
            self.split_current_dir.push(dir);
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn set_split_current_dir(&mut self) {
        self.split_current_dir.clear();
        for last_dir in self.current_dir.iter() {
            let dir = last_dir.to_string_lossy().to_string();
            println!("Splitting directory {:?}", dir);
            self.split_current_dir.push(dir);
        }
    }

    pub fn change_dir(&mut self, new_dir: &PathBuf) {
        self.current_dir = dunce::canonicalize(new_dir).unwrap_or_default();
        println!("{:?}", self.current_dir.to_str());
        self.set_split_current_dir();
        self.update_paths();
    }

    pub fn file_navbar(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.set_min_height(30.);
            ui.horizontal_centered(|ui| {
                if ui.button("Open").clicked() {
                    self.current_dir.pop();
                    self.split_current_dir.pop();
                    self.update_paths();
                };
                if ui.button("Update").clicked() {
                    self.update_paths();
                };
                // ui.label("Current Path:");
                if ui.button("<<<").clicked() {
                    self.current_dir.pop();
                    self.split_current_dir.pop();
                    self.update_paths();
                };
                ui.horizontal_wrapped(|ui| {
                    for (i, dir) in self.split_current_dir.iter().enumerate() {
                        ui.separator();
                        if ui
                            .selectable_label(
                                false,
                                RichText::new(dir).color(Color32::LIGHT_BLUE).underline(),
                            )
                            .clicked()
                        {
                            self.current_dir = self.split_current_dir[0..=i].iter().collect();
                            self.split_current_dir.truncate(i + 1);
                            self.update_paths();
                            break;
                        };
                    }
                })
            });
        });
    }

    pub fn file_list_ui(&mut self, ui: &mut Ui) -> bool {
        // Affichez le chemin actuel en tant qu'en-tête.
        let mut is_file_clicked = false;

        ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.set_min_height(25.);
                ui.heading("Directories");
            });
            ui.separator();
            if ui.button("../").clicked() {
                self.current_dir.pop();
                self.split_current_dir.pop();
                self.update_paths();
            };
            // let mut is_dir_clicked = false;
            for (dirname, dir_path) in self.dirnames.iter().zip(self.dir_vec.iter()) {
                if ui.button(dirname).clicked() {
                    self.change_dir(&dir_path.clone());
                    return;
                };
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.set_min_height(25.);
                ui.heading("Files");
            });
            ui.separator();

            for (i, (filename, file_path)) in
                self.filenames.iter().zip(self.file_vec.iter()).enumerate()
            {
                let is_colored =
                    self.selected_file_index.is_some() && self.selected_file_index.unwrap() == i;
                if ui.selectable_label(is_colored, filename).clicked() {
                    self.selected_file = Some(file_path.clone());
                    self.selected_file_index = Some(i);
                    is_file_clicked = true;
                };
            }
        });
        is_file_clicked
    }
}
