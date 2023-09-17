#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use anyhow::{Error, Result};
use cv_convert::TryIntoCv;
use eframe::{egui, run_native, App, NativeOptions};
use egui::{Color32, ColorImage, Key, Label, RichText, ScrollArea, TextEdit, Vec2};
use egui_extras::RetainedImage;
use image::DynamicImage;
use opencv::{
    core::{Mat, Rect, Size, Vector},
    imgcodecs,
    imgproc::{cvt_color, COLOR_BGR2RGB, COLOR_RGB2BGR},
};
use rfd::FileDialog;
mod marker_utils;
use marker_utils::marker_processing::*;

mod file_explorer;
use file_explorer::FileExplorer;

fn main() {
    let window_options = NativeOptions {
        initial_window_size: Option::from(Vec2::new(1200., 800.)),
        ..Default::default()
    };
    run_native(
        "IdMyBee Markerzzzz",
        window_options,
        Box::new(|cc| Box::new(IdMyBeeApp::new(cc))),
    )
    .unwrap();
}

struct IdMyBeeApp<'a> {
    explorer: FileExplorer<'a>,
    cv_orig_image: Option<Mat>,
    cv_cropped_image: Option<Mat>,
    egui_orig_image: Option<RetainedImage>,
    egui_cropped_image: Option<RetainedImage>,
    out_x: u32,
    out_y: u32,
    zoom: f32,
    try_load: bool,
    load_img_res: Result<()>,
    crop_img_res: Result<()>,
    save_img_res: Result<()>,
}

impl IdMyBeeApp<'_> {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // app.set_image(app.img_path);
        // app.egui_orig_image = IdMyBeeApp::cv_img_to_egui_img(&app.cv_orig_image);
        // app.cv_cropped_image = app.process_image();
        // app.egui_cropped_image = IdMyBeeApp::cv_img_to_egui_img(&app.cv_cropped_image);
        IdMyBeeApp {
            explorer: FileExplorer::new(),
            // img_path: "C:/Users/20100/Documents/Rust/idmybee/ressources/test_cards/Photos-001/IMG_20230805_231619.jpg",
            cv_orig_image: None,
            cv_cropped_image: None,
            egui_orig_image: None,
            egui_cropped_image: None,
            out_x: 600,
            out_y: 300,
            zoom: 1.2,
            try_load: false,
            load_img_res: Ok(()),
            crop_img_res: Ok(()),
            save_img_res: Ok(()),
        }
    }

    fn clear_orig_images(&mut self) {
        self.cv_orig_image = None;
        self.egui_orig_image = None;
        self.crop_img_res = Ok(());
        self.save_img_res = Ok(());
    }

    fn clear_cropped_images(&mut self) {
        self.cv_cropped_image = None;
        self.egui_cropped_image = None;
        self.crop_img_res = Ok(());
        self.save_img_res = Ok(());
    }

    fn clear_all_images(&mut self) {
        self.clear_orig_images();
        self.clear_cropped_images();
    }

    fn load_image_from_path(&mut self, img_path: &str) {
        self.try_load = true;
        let load_img_res = imgcodecs::imread(img_path, imgcodecs::IMREAD_UNCHANGED);
        let brg_cv_img: Mat;
        match load_img_res {
            Ok(img) => {
                brg_cv_img = img;
                self.load_img_res = Ok(());
            }
            Err(err) => {
                self.load_img_res = Err(err.into());
                self.clear_all_images();
                return;
            }
        };

        let mut rgb_cv_img = Mat::default();
        match cvt_color(&brg_cv_img, &mut rgb_cv_img, COLOR_BGR2RGB, 0) {
            Ok(_) => {
                self.cv_orig_image = Some(rgb_cv_img);
                self.load_img_res = Ok(());
            }
            Err(err) => {
                self.load_img_res = Err(err.into());
                self.clear_all_images();
                return;
            }
        };

        match IdMyBeeApp::cv_img_to_egui_img(
            &self.cv_orig_image,
            "Original Image",
            &mut self.egui_orig_image,
        ) {
            Ok(_) => {
                self.load_img_res = Ok(());
            }
            Err(err) => {
                self.load_img_res = Err(err);
                self.clear_all_images();
                return;
            }
        };

        self.load_img_res = IdMyBeeApp::cv_img_to_egui_img(
            &self.cv_orig_image,
            "Original Image",
            &mut self.egui_orig_image,
        );

        self.clear_cropped_images();
    }

    fn load_image_from_explorer(&mut self) {
        if let Some(img_path) = self.explorer.get_filepath() {
            self.load_image_from_path(&img_path);
        };
    }

    fn cv_img_to_egui_img(
        cv_img: &Option<Mat>,
        image_id: &str,
        dst: &mut Option<RetainedImage>,
    ) -> Result<()> {
        if let Some(cv_img) = cv_img {
            let dyn_img: DynamicImage = cv_img.try_into_cv()?;
            let img_buff = dyn_img.to_rgba8();
            let size = [dyn_img.width() as _, dyn_img.height() as _];
            let pixels = img_buff.into_flat_samples();
            let color_img = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
            *dst = Some(RetainedImage::from_color_image(image_id, color_img));
            return Ok(());
        }
        Err(anyhow::anyhow!("No opened image was found"))
    }

    fn process_image(&mut self) -> Result<Mat> {
        if let Some(img) = self.cv_orig_image.as_ref() {
            let out_size = Size::new(self.out_x as i32, self.out_y as i32);
            let img = resize_if_larger_dims(img.to_owned(), &out_size)?;
            let (markers_coor, markers_id, _) = get_image_markers(&img)?;
            if markers_coor.len() != 4 {
                return Err(anyhow::anyhow!("Error: {:?} markers were found instead of 4.\nThe image may be too blurred (i.e. not enough contrast at markers positions) or there may be stray reflections on the markers (makers not black and white). Also check that markers 0 to 4 are present on the picture.", 
                    markers_coor.len()
                ));
            }
            let ordered_points = parse_markers(&markers_coor, &markers_id)?;
            let warped_image = correct_image(&img, &ordered_points, &out_size, &self.zoom)?;
            let final_image = Mat::roi(
                &warped_image,
                Rect {
                    x: 0,
                    y: 0,
                    width: out_size.width,
                    height: out_size.height,
                },
            )?;
            return Ok(final_image);
        }
        let err_str = "No image was previously loaded. Select an image with the explorer in the left panel and then crop it.";
        Err(anyhow::anyhow!(err_str))
    }

    fn process_image_wrapper(&mut self, ui: &mut egui::Ui) {
        match self.process_image() {
            Ok(img) => {
                self.cv_cropped_image = Some(img);
                self.crop_img_res = IdMyBeeApp::cv_img_to_egui_img(
                    &self.cv_cropped_image,
                    "Cropped Image",
                    &mut self.egui_cropped_image,
                );
            }
            Err(err) => {
                IdMyBeeApp::display_error(ui, &err);
                self.crop_img_res = Err(err);
            }
        };
        if let Err(err) = self.crop_img_res.as_ref() {
            IdMyBeeApp::display_error(ui, err);
        };
    }

    fn display_error(ui: &mut egui::Ui, err: &Error) {
        let string_err: String = err.to_string();
        ui.label(RichText::new(string_err).color(Color32::RED));
    }

    fn save_cropped_image(&mut self) {
        if self.cv_cropped_image.is_some() {
            let mut out_full_path = self.explorer.output_img_dir.clone();
            out_full_path.push(&self.explorer.output_img_name);

            let mut rgb_img = Mat::default();
            match cvt_color(
                &self.cv_cropped_image.as_ref().unwrap(),
                &mut rgb_img,
                COLOR_RGB2BGR,
                0,
            ) {
                Ok(_) => self.save_img_res = Ok(()),
                Err(err) => self.save_img_res = Err(err.into()),
            }

            match imgcodecs::imwrite(&out_full_path.to_string_lossy(), &rgb_img, &Vector::new()) {
                Ok(_) => {
                    self.crop_img_res = Ok(());
                    self.explorer.update_paths();
                    self.save_img_res = Ok(());
                }
                Err(err) => {
                    println!("Error: {}", err);
                    self.save_img_res = Err(err.into())
                }
            }
        }
    }

    fn display_shortcuts(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.set_min_height(30.);
            ui.horizontal_centered(|ui| {
                ui.heading("Shortcuts");
                ui.add_space(20.);
                ui.horizontal_wrapped(|ui| {
                    ui.separator();
                    ui.add(Label::new(
                        RichText::new("S").color(Color32::LIGHT_BLUE).underline(),
                    ));
                    ui.label("Next file");
                    ui.add_space(10.);
                    if ui.input(|i| i.key_pressed(Key::Z)) {
                        self.explorer.previous_file();
                        if self.explorer.selected_file.is_some() {
                            self.load_image_from_explorer();
                        } else {
                            self.clear_all_images();
                        }
                    };

                    ui.separator();
                    ui.add(Label::new(
                        RichText::new("Z").color(Color32::LIGHT_BLUE).underline(),
                    ));
                    ui.label("Previous file");
                    ui.add_space(10.);
                    if ui.input(|i| i.key_pressed(Key::S)) {
                        self.explorer.next_file();
                        if self.explorer.selected_file.is_some() {
                            self.load_image_from_explorer();
                        } else {
                            self.clear_all_images();
                        }
                    };

                    ui.separator();
                    ui.add(Label::new(RichText::new("D").color(Color32::LIGHT_BLUE)));
                    ui.label("Zoom +");
                    ui.add_space(10.);
                    if ui.input(|i| i.key_pressed(Key::D)) {
                        self.zoom += 0.1;
                    };

                    ui.separator();
                    ui.add(Label::new(
                        RichText::new("Q").color(Color32::LIGHT_BLUE).underline(),
                    ));
                    ui.label("Zoom -");
                    ui.add_space(10.);
                    if ui.input(|i| i.key_pressed(Key::Q)) {
                        self.zoom -= 0.1;
                    }

                    ui.separator();
                    ui.add(Label::new(
                        RichText::new("Space")
                            .color(Color32::LIGHT_BLUE)
                            .underline(),
                    ));
                    ui.label("Crop image");
                    ui.add_space(10.);
                    if ui.input(|i| i.key_pressed(Key::Space)) {
                        self.process_image_wrapper(ui);
                        self.explorer.output_img_name = self.explorer.get_default_output_filename();
                    }

                    ui.separator();
                    ui.add(Label::new(
                        RichText::new("F").color(Color32::LIGHT_BLUE).underline(),
                    ));
                    ui.label("Selected output folder");
                    ui.add_space(10.);
                    if ui.input(|i| i.key_pressed(Key::F)) {
                        if let Some(path) = FileDialog::new().pick_folder() {
                            // self.current_dir = Some(path.display().to_string());
                            self.explorer.output_img_dir = path;
                        };
                    }

                    ui.separator();
                    ui.add(Label::new(
                        RichText::new("V").color(Color32::LIGHT_BLUE).underline(),
                    ));
                    ui.label("Selected output folder");
                    ui.add_space(10.);
                    if ui.input(|i| i.key_pressed(Key::V)) {
                        if let Some(path) = FileDialog::new().pick_folder() {
                            // self.current_dir = Some(path.display().to_string());
                            self.explorer.change_dir(&path)
                        };
                    }

                    ui.separator();
                    ui.add(Label::new(
                        RichText::new("R").color(Color32::LIGHT_BLUE).underline(),
                    ));
                    ui.label("Save cropped image");
                    ui.add_space(10.);
                    if ui.input(|i| i.key_pressed(Key::R)) {
                        self.save_cropped_image();
                    }
                });
            });
        });
    }

    fn integer_edit_field(ui: &mut egui::Ui, value: &mut u32, max_size: Vec2) -> egui::Response {
        let mut str_value = match value {
            0 => String::new(),
            _ => format!("{}", value),
        };

        let res = ui.add_sized(max_size, TextEdit::singleline(&mut str_value));
        if let Ok(result) = str_value.trim().parse() {
            *value = result;
        } else {
            *value = 0;
        }
        res
    }

    fn cropped_image_ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_sized(
                Vec2::new(ui.available_width(), 25.),
                Label::new(
                    RichText::new("Cropped image")
                        .heading()
                        .color(Color32::LIGHT_BLUE),
                ),
            );
        });
        ui.separator();

        if let Some(img) = self.egui_cropped_image.as_ref() {
            img.show_max_size(ui, ui.available_size());
            ui.separator();
        } else if self.egui_cropped_image.is_none() && self.crop_img_res.is_err() {
            IdMyBeeApp::display_error(ui, self.crop_img_res.as_ref().unwrap_err());
        }
    }

    fn crop_param_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            let slider = ui.add(egui::Slider::new(&mut self.zoom, 1.0..=2.5).text("Zoom"));
            if slider.drag_released() || slider.lost_focus() && slider.changed() {
                self.process_image_wrapper(ui);
            };
            ui.separator();
            ui.separator();
            if IdMyBeeApp::<'_>::integer_edit_field(ui, &mut self.out_x, Vec2::new(40., 15.))
                .lost_focus()
            {
                self.process_image_wrapper(ui)
            };
            ui.separator();
            // ui.add_space(7.);
            if IdMyBeeApp::<'_>::integer_edit_field(ui, &mut self.out_y, Vec2::new(40., 15.))
                .lost_focus()
            {
                self.process_image_wrapper(ui)
            };
            // ui.add_space(30.);
        });
        // ui.label(format!("Zoom : {:.1}", self.zoom));

        ui.separator();
    }
}

impl App for IdMyBeeApp<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("Navbar").show(ctx, |ui| self.explorer.file_navbar(ui));

        egui::TopBottomPanel::bottom("Shortcuts").show(ctx, |ui| self.display_shortcuts(ui));

        egui::SidePanel::left("Files")
            .default_width(300.)
            .resizable(true)
            .show(ctx, |ui| {
                if self.explorer.file_list_ui(ui) {
                    self.load_image_from_explorer();
                }
                ui.allocate_space(ui.available_size());
            });

        egui::SidePanel::left("Image")
            .default_width(450.)
            .resizable(true)
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_sized(
                            Vec2::new(ui.available_width(), 30.),
                            Label::new(
                                RichText::new("Original image")
                                    .heading()
                                    .color(Color32::LIGHT_BLUE),
                            ),
                        );
                        ui.separator();
                    });
                    if let Some(img) = self.egui_orig_image.as_ref() {
                        img.show_max_size(ui, ui.available_size());
                        ui.vertical_centered(|ui| {
                            ui.separator();
                            self.crop_param_ui(ui);
                            if ui.button("Process Image").clicked() && self.cv_orig_image.is_some()
                            {
                                self.process_image_wrapper(ui)
                            }
                        });
                    } else if self.try_load
                        && self.egui_orig_image.is_none()
                        && self.load_img_res.is_err()
                    {
                        IdMyBeeApp::display_error(ui, self.load_img_res.as_ref().unwrap_err());
                    }
                });
                ui.allocate_space(ui.available_size());
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.cropped_image_ui(ui);
            if self
                .explorer
                .img_saving_ui(ui, self.egui_cropped_image.is_some())
            {
                self.save_cropped_image();
            };
            if let Err(err) = self.save_img_res.as_ref() {
                IdMyBeeApp::display_error(ui, err);
            }
            ui.allocate_space(ui.available_size());
        });
    }
}
