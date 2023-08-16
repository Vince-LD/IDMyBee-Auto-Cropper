use anyhow::Result;
use cv_convert::{TryFromCv, TryIntoCv};
use eframe::{egui, run_native, App, NativeOptions};
use egui::ColorImage;
use egui_extras::RetainedImage;
use image::{DynamicImage, ImageBuffer, Rgba};
use opencv::{
    core::{Mat, Rect, Size},
    imgcodecs,
    imgproc::{cvt_color, COLOR_BGR2RGB, COLOR_RGBA2BGRA},
    prelude::MatTraitConst,
};
mod marker_utils;
use marker_utils::marker_processing::*;

fn main() {
    let window_options = NativeOptions::default();
    run_native(
        "IdMyBee Markerzzzz",
        window_options,
        Box::new(|cc| Box::new(IdMyBeeApp::new(cc))),
    )
    .unwrap();
}

struct IdMyBeeApp<'a> {
    img_path: &'a str,
    cv_orig_image: Option<Mat>,
    cv_cropped_image: Option<Mat>,
    egui_orig_image: Option<RetainedImage>,
    egui_cropped_image: Option<RetainedImage>,
    out_x: i32,
    out_y: i32,
    zoom: f32,
}

impl IdMyBeeApp<'_> {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = IdMyBeeApp {
            img_path: "C:/Users/20100/Documents/Rust/idmybee/ressources/test_cards/Photos-005/IMG_20230806_230443.jpg",
            cv_orig_image: None,
            cv_cropped_image: None,
            egui_orig_image: None,
            egui_cropped_image: None,
            out_x: 600,
            out_y: 300,
            zoom: 1.2
        };
        app.set_image(app.img_path);
        app.egui_orig_image = IdMyBeeApp::cv_img_to_egui_img(&app.cv_orig_image);
        app.cv_cropped_image = app.process_image();
        app.egui_cropped_image = IdMyBeeApp::cv_img_to_egui_img(&app.cv_cropped_image);
        app
    }

    fn load_image_from_path(&mut self, img_path: &str) -> Result<ColorImage> {
        let rgba_cv_img: Mat = imgcodecs::imread(img_path, imgcodecs::IMREAD_UNCHANGED)?;
        let mut brga_cv_img = rgba_cv_img.clone();
        println!(">>>>>>> {:?}", brga_cv_img.channels());
        cvt_color(&rgba_cv_img, &mut brga_cv_img, COLOR_BGR2RGB, 0).unwrap();
        self.cv_orig_image = Some(brga_cv_img.clone());
        let dyn_img: DynamicImage = brga_cv_img.try_into_cv().unwrap();
        let img_buff = dyn_img.to_rgba8();
        // let img_buff: ImageBuffer<Rgba<u8>, u8> = cv_img.try_into_cv().unwrap();
        let size = [dyn_img.width() as _, dyn_img.height() as _];
        let pixels = img_buff.into_flat_samples();
        Ok(ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
    }

    fn set_image(&mut self, img_path: &str) {
        let color_img = self.load_image_from_path(img_path).unwrap();
        // let texture: &egui::TextureHandle = self.egui_orig_image.get_or_insert_with(|| {
        //     // Load the texture only once.
        //     ui.ctx()
        //         .load_texture("Original Image", color_img, Default::default())
        // });
        self.egui_orig_image = Some(RetainedImage::from_color_image("Original Image", color_img));
    }

    fn cv_img_to_egui_img(cv_img: &Option<Mat>) -> Option<RetainedImage> {
        if let Some(cv_img) = cv_img {
            let dyn_img: DynamicImage = cv_img.try_into_cv().unwrap();
            let img_buff = dyn_img.to_rgba8();
            let size = [dyn_img.width() as _, dyn_img.height() as _];
            let pixels = img_buff.into_flat_samples();
            let color_img = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
            return Some(RetainedImage::from_color_image("Original Image", color_img));
        }
        None
    }

    fn process_image(&mut self) -> Option<Mat> {
        if let Some(img) = self.cv_orig_image.as_ref() {
            let out_size = Size::new(self.out_x, self.out_y);
            let img = resize_if_larger_dims(img.to_owned(), &out_size).unwrap();
            let (markers_coor, markers_id, _) = get_image_markers(&img).unwrap();
            let ordered_points = parse_markers(&markers_coor, &markers_id).unwrap();
            let warped_image = correct_image(&img, &ordered_points, &out_size, &self.zoom).unwrap();
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
            return Some(final_image);
        }
        None
    }
}

impl App for IdMyBeeApp<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("Files")
            .resizable(true)
            .show(ctx, |ui| {
                ui.label("Hi!!! :)");
                ui.allocate_space(ui.available_size());
            });
        egui::TopBottomPanel::bottom("Controls")
            .resizable(true)
            .show(ctx, |ui| {
                ui.separator();
                ui.label("Hello you...");
                ui.allocate_space(ui.available_size());
            });
        egui::SidePanel::left("Image")
            .resizable(true)
            .show(ctx, |ui| {
                ui.label("Original Image:");
                if let Some(img) = self.egui_orig_image.as_ref() {
                    img.show_max_size(ui, ui.available_size());
                }
                ui.allocate_space(ui.available_size());
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Cropped Image");
            if let Some(img) = self.egui_cropped_image.as_ref() {
                img.show_max_size(ui, ui.available_size());
            }
            ui.separator();
            ui.label(format!("Dimensions : {}x{}p", self.out_x, self.out_y));
            ui.separator();
            ui.label(format!("Zoom : {:.1}", self.zoom));
            ui.separator();
            ui.label(format!("Input : {}", self.img_path));
            ui.separator();
            ui.allocate_space(ui.available_size());
        });
    }
}
