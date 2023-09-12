use anyhow::{Error, Result};
use egui::{Button, Color32, Label, RichText, ScrollArea, Ui};
use std::cmp::min;
use std::ffi::OsStr;
use std::fs::{self, DirEntry};
use std::path::PathBuf;
use std::rc::Rc;

pub struct FileExplorer<'a> {
    pub current_dir: PathBuf,
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
            selected_file: None,
            selected_file_index: None,
            dir_vec: Vec::new(),
            file_vec: Vec::new(),
            dirnames: Vec::new(),
            filenames: Vec::new(),
            allowed_extensions: vec!["png", "jpg", "jpeg", "tiff"],
            err: Ok(()),
        };
        fe.err = fe.update_paths();
        fe
    }

    pub fn update_paths(&mut self) -> Result<()> {
        let dir_content = std::fs::read_dir(&self.current_dir)?
            .filter_map(|dir| dir.ok())
            .collect::<Vec<DirEntry>>()
            .iter()
            .map(|dir| dir.path())
            .collect::<Vec<PathBuf>>();

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
                _ if path.is_file()
                    && self.allowed_extensions.contains(
                        &path
                            .extension()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default(),
                    ) =>
                {
                    if let Some(filename) = path.file_name() {
                        self.file_vec.push(path.clone());
                        let str_filename = filename.to_string_lossy().to_string();
                        self.filenames.push(str_filename);
                        //  self.allowed_extensions.contains(&filename.to_str().unwrap_or("")) {
                    }
                }
                _ => (),
            }
        }
        Ok(())
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
            .map(|path| path.to_string_lossy().to_string())
    }

    pub fn previous_file(&mut self) {
        if !self.file_vec.is_empty() {
            let new_index = match self.selected_file_index {
                Some(index) => index.saturating_sub(1),
                None => 0,
            };
            self.selected_file_index = Some(min(self.file_vec.len(), new_index));
            let new_selected_file = match self.selected_file {
                Some(_) => self.file_vec.get(new_index).unwrap().clone(),
                None => self.file_vec.get(0).unwrap().clone(),
            };
            self.selected_file = Some(new_selected_file)
        }
    }

    pub fn next_file(&mut self) {
        if !self.file_vec.is_empty() {
            let new_index = match self.selected_file_index {
                Some(index) => index + 1,
                None => 0,
            };
            self.selected_file_index = Some(min(self.file_vec.len(), new_index));
            let new_selected_file = match self.selected_file {
                Some(_) => self.file_vec.get(new_index).unwrap().clone(),
                None => self.file_vec.get(0).unwrap().clone(),
            };
            self.selected_file = Some(new_selected_file)
        }
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
    pub fn file_navbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("Previous").clicked() {
                self.current_dir.pop();
                self.err = self.update_paths();
            };
            if ui.button("Update").clicked() {
                self.err = self.update_paths();
            };
            ui.label("Current Path:");
            ui.label(
                RichText::new(self.current_dir.display().to_string())
                    .color(Color32::LIGHT_BLUE)
                    .underline(),
            );
        });
    }

    pub fn file_list_ui(&mut self, ui: &mut Ui) -> bool {
        // Affichez le chemin actuel en tant qu'en-tête.

        let mut is_file_clicked = false;
        ScrollArea::vertical().show(ui, |ui| {
            let mut should_update = false;
            for (dirname, dir_path) in self.dirnames.iter().zip(self.dir_vec.iter()) {
                if ui.button(dirname).clicked() {
                    self.current_dir = dir_path.clone();
                    should_update = true;
                };
            }
            if should_update {
                self.err = self.update_paths();
            }

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
