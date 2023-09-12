use anyhow::{Error, Result};
use cv_convert::TryIntoCv;
use eframe::{egui, run_native, App, NativeOptions};
use egui::{Button, Color32, ColorImage, Key, RichText};
use egui_extras::RetainedImage;
use image::DynamicImage;
use opencv::{
    core::{Mat, Rect, Size},
    imgcodecs,
    imgproc::{cvt_color, COLOR_BGR2RGB},
};
mod marker_utils;
use marker_utils::marker_processing::*;

mod file_explorer;
use file_explorer::FileExplorer;

fn main() {
    let window_options = NativeOptions {
        drag_and_drop_support: true,
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
    out_x: i32,
    out_y: i32,
    zoom: f32,
    try_load: bool,
    load_img_res: Result<()>,
    crop_img_res: Result<()>,
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
        }
    }

    fn clear_orig_images(&mut self) {
        self.cv_orig_image = None;
        self.egui_orig_image = None;
        self.crop_img_res = Ok(());
    }

    fn clear_cropped_images(&mut self) {
        self.cv_cropped_image = None;
        self.egui_cropped_image = None;
        self.crop_img_res = Ok(());
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

    fn process_image(&mut self) -> Result<()> {
        if let Some(img) = self.cv_orig_image.as_ref() {
            let out_size = Size::new(self.out_x, self.out_y);
            let img = resize_if_larger_dims(img.to_owned(), &out_size)?;
            let (markers_coor, markers_id, _) = get_image_markers(&img)?;
            let ordered_points = parse_markers(&markers_coor, &markers_id)?;
            if markers_coor.len() != 4 {
                return Err(anyhow::anyhow!("Error: {:?} markers were found instead of 4.\nThe image may be too blurred (i.e. not enough contrast at markers positions) or there may be stray reflections on the markers (makers not black and white). Also check that markers 0 to 4 are present on the picture.", 
                    markers_coor.len()
                ));
            }
            let warped_image = correct_image(&img, &ordered_points, &out_size, &self.zoom)?;
            let final_image = Mat::roi(
                &warped_image,
                Rect {
                    x: 0,
                    y: 0,
                    width: out_size.width,
                    height: out_size.height,
                },
            )
            .unwrap();
            self.crop_img_res = Ok(());
            self.cv_cropped_image = Some(final_image);
            return IdMyBeeApp::cv_img_to_egui_img(
                &self.cv_cropped_image,
                "Cropped Image",
                &mut self.egui_cropped_image,
            );
        }
        let err_str = "No image was previously loaded. Try to press the 'Load Image' button.";
        // self.crop_img_res = Err(anyhow::anyhow!(err_str));
        Err(anyhow::anyhow!(err_str))
    }

    // fn try_show_orig_image(&self, ui: &egui::Ui) {

    // }

    fn display_error(ui: &mut egui::Ui, err: &Error) {
        let string_err: String = err.to_string();
        ui.label(RichText::new(string_err).color(Color32::RED));
    }
}

impl App for IdMyBeeApp<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("Navbar").show(ctx, |ui| self.explorer.file_navbar(ui));
        egui::SidePanel::left("Files")
            .resizable(true)
            .show(ctx, |ui| {
                if self.explorer.file_list_ui(ui) {
                    self.load_image_from_explorer();
                }
                ui.allocate_space(ui.available_size());
            });

        egui::TopBottomPanel::bottom("Controls")
            .resizable(true)
            .show(ctx, |ui| {
                ui.label("Commands");
                ui.add(egui::Slider::new(&mut self.zoom, 1.0..=2.5).text("Zoom"));
                if ui.add(Button::new("Process Image")).clicked() && self.cv_orig_image.is_some() {
                    self.crop_img_res = match self.process_image() {
                        Ok(()) => Ok(()),
                        Err(err) => {
                            IdMyBeeApp::display_error(ui, &err);
                            Err(err)
                        }
                    };
                }
                ui.allocate_space(ui.available_size());
            });
        egui::SidePanel::left("Image")
            .resizable(true)
            .show(ctx, |ui| {
                ui.label("Original Image:");
                if let Some(img) = self.egui_orig_image.as_ref() {
                    img.show_max_size(ui, ui.available_size());
                } else if self.try_load
                    && self.egui_orig_image.is_none()
                    && self.load_img_res.is_err()
                {
                    IdMyBeeApp::display_error(ui, self.load_img_res.as_ref().unwrap_err());
                }
                ui.allocate_space(ui.available_size());
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Cropped Image");
            if let Some(img) = self.egui_cropped_image.as_ref() {
                img.show_max_size(ui, ui.available_size());
                ui.separator();
            } else if self.egui_cropped_image.is_none() && self.crop_img_res.is_err() {
                IdMyBeeApp::display_error(ui, self.crop_img_res.as_ref().unwrap_err());
            }
            ui.label(format!("Dimensions : {}x{}p", self.out_x, self.out_y));
            ui.separator();
            ui.label(format!("Zoom : {:.1}", self.zoom));
            ui.separator();
            // ui.label(format!("Input : {}", &self.img_path.clone().unwrap()));
            // ui.separator();
            ui.allocate_space(ui.available_size());
        });

        egui::TopBottomPanel::bottom("Shortcuts").show(ctx, |ui| {
            if ctx.input(|i| i.key_pressed(Key::V)) && self.explorer.selected_file.is_some() {
                self.load_image_from_explorer();
            }
            if ctx.input(|i| i.key_pressed(Key::Space)) {
                self.crop_img_res = match self.process_image() {
                    Ok(()) => Ok(()),
                    Err(err) => {
                        IdMyBeeApp::display_error(ui, &err);
                        Err(err)
                    }
                };
            }
            if ctx.input(|i| i.key_pressed(Key::D)) {
                self.zoom += 0.1;
            };
            if ctx.input(|i| i.key_pressed(Key::Q)) {
                self.zoom -= 0.1;
            }
            if ctx.input(|i| i.key_pressed(Key::Z)) {
                self.explorer.previous_file();
                if self.explorer.selected_file.is_some() {
                    self.load_image_from_explorer();
                } else {
                    self.clear_all_images();
                }
            };
            if ctx.input(|i| i.key_pressed(Key::S)) {
                self.explorer.next_file();
                if self.explorer.selected_file.is_some() {
                    self.load_image_from_explorer();
                } else {
                    self.clear_all_images();
                }
            };
        });
    }
}
