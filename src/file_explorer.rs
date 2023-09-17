use anyhow::Result;

use egui::{
    Button, Color32, Label, RichText, ScrollArea, SelectableLabel, Style, TextStyle, Ui, Vec2,
};
use rfd::FileDialog;
use same_file::is_same_file;
use std::cmp::min;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};

pub struct FileExplorer<'a> {
    pub current_dir: PathBuf,
    pub split_current_dir: Vec<String>,
    pub selected_file: Option<PathBuf>,
    pub selected_file_index: Option<usize>,
    pub output_img_name: String,
    pub output_img_dir: PathBuf,
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
            output_img_name: String::new(),
            output_img_dir: PathBuf::new(),
            dir_vec: Vec::new(),
            file_vec: Vec::new(),
            dirnames: Vec::new(),
            filenames: Vec::new(),
            allowed_extensions: vec![
                "bmp", "dib", "jpeg", "jpg", "jpe", "jp2", "png", "webp", "avif", "pbm", "pgm",
                "ppm", "pxm", "pnm", "pfm", "sr", "ras", "tiff", "tif", "exr", "hdr", "pic",
            ],
            err: Ok(()),
        };
        fe.change_dir(&fe.current_dir.clone());
        // if let Some(path) = FileDialog::new().pick_folder() {
        //     fe.change_dir(&path);
        // }
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
                _ if path.is_file() && self.is_file_valid_image_ext(&path) => {
                    if let Some(filename) = path.file_name() {
                        println!("{:?} is a picture", filename);
                        self.file_vec.push(path.clone());
                        let str_filename = filename.to_string_lossy().to_string();
                        self.filenames.push(str_filename);
                    }
                }
                _ => (),
            }
        }
        if let Some(path) = self.selected_file.as_ref() {
            self.selected_file_index = self
                .file_vec
                .iter()
                .position(|r| is_same_file(r, path).unwrap_or(false));
        }
        self.err = Ok(())
    }

    fn is_file_valid_image_ext(&self, img_path: &Path) -> bool {
        self.allowed_extensions.contains(
            &img_path
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
                .as_ref(),
        )
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
        self.output_img_dir = new_dir.clone();
        println!("{:?}", self.current_dir.to_str());
        self.set_split_current_dir();
        self.update_paths();
    }

    pub fn file_navbar(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.set_min_height(30.);
            ui.horizontal_centered(|ui| {
                if ui.button("Select folder").clicked() {
                    if let Some(path) = FileDialog::new().pick_folder() {
                        // self.current_dir = Some(path.display().to_string());
                        self.change_dir(&path)
                    };
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
            let available_width = ui.available_width();
            let title_size = Vec2::new(available_width, 30.);
            let elem_size = Vec2::new(available_width, 20.);
            ui.horizontal(|ui| {
                ui.add_sized(
                    title_size,
                    Label::new(
                        RichText::new("Directories")
                            .heading()
                            .color(Color32::LIGHT_BLUE),
                    ),
                );
            });
            ui.separator();
            if ui
                .add_sized(elem_size, SelectableLabel::new(false, "../"))
                .clicked()
            {
                self.current_dir.pop();
                self.split_current_dir.pop();
                self.update_paths();
            };
            // let mut is_dir_clicked = false;
            for (dirname, dir_path) in self.dirnames.iter().zip(self.dir_vec.iter()) {
                if ui
                    .add_sized(elem_size, SelectableLabel::new(false, dirname))
                    .clicked()
                {
                    self.change_dir(&dir_path.clone());
                    return;
                };
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.add_sized(
                    title_size,
                    Label::new(RichText::new("Images").heading().color(Color32::LIGHT_BLUE)),
                );
            });
            ui.separator();

            for (i, (filename, file_path)) in
                self.filenames.iter().zip(self.file_vec.iter()).enumerate()
            {
                let is_colored =
                    self.selected_file_index.is_some() && self.selected_file_index.unwrap() == i;
                if ui
                    .add_sized(elem_size, SelectableLabel::new(is_colored, filename))
                    .clicked()
                {
                    self.selected_file = Some(file_path.clone());
                    self.selected_file_index = Some(i);
                    is_file_clicked = true;
                    self.output_img_name = self.get_default_output_filename();
                };
            }
        });
        is_file_clicked
    }

    pub fn get_default_output_filename(&self) -> String {
        let mut out_filename = String::new();
        if let Some(filename) = self.selected_file.as_ref() {
            let ext = filename
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            out_filename.push_str(&filename.file_stem().unwrap_or_default().to_string_lossy());
            out_filename.push_str("_crop.");
            out_filename.push_str(&ext);
        }
        out_filename
    }

    pub fn img_saving_ui(&mut self, ui: &mut egui::Ui, is_visible: bool) -> bool {
        // let mut tmp_out = self.output_img_path.clone();
        // ui.horizontal_wrapped(|ui| {
        //     if ui.text_edit_singleline(&mut tmp_out).lost_focus() {
        //         let mut tmp_out_path = PathBuf::from(tmp_out);
        //         if let Some(parent_dir) = tmp_out_path.parent() {
        //             if parent_dir.is_dir() && self.is_file_valid_image_ext(&tmp_out_path) {
        //                 self.output_img_path.clear();
        //                 self.output_img_path.push_str(&tmp_out_path.)
        //             }
        //         }
        //     }
        // });
        let mut must_save = false;
        ui.add_visible_ui(is_visible, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("Select out directory").clicked() {
                        if let Some(path) = FileDialog::new()
                            .set_directory(&self.output_img_dir)
                            .pick_folder()
                        {
                            self.output_img_dir = path
                        }
                    }
                    ui.separator();
                    ui.add(
                        Label::new(self.output_img_dir.to_string_lossy().to_string()).wrap(true),
                    );
                    // ui.label(RichText::new("Output directory:").underline());
                    // if ui
                    //     .link(self.output_img_dir.to_string_lossy().to_string())
                    //     .clicked()
                    // {
                    //     if let Some(path) = FileDialog::new()
                    //         .set_directory(&self.output_img_dir)
                    //         .pick_folder()
                    //     {
                    //         self.output_img_dir = path;
                    //     }
                    // };
                });
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    // let label_style = Style::default().text_style(TextStyle::Button);
                    ui.add(Label::new(RichText::new("Image file name")));
                    ui.separator();
                    ui.text_edit_singleline(&mut self.output_img_name);
                    if ui.button("Save").clicked() {
                        let mut out_full_path = self.output_img_dir.clone();
                        out_full_path.push(&self.output_img_name);
                        ui.label(RichText::new(format!("Saved {:?}", out_full_path)));
                        must_save = true;
                    }
                });
            });
        });
        must_save
    }
}
