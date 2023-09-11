use eframe::{run_native, App, NativeOptions};
use egui::{Color32, Context};
mod file_explorer;
use file_explorer::FileExplorer;

struct MainState {
    explorer: FileExplorer,
}

impl App for MainState {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("Errors").show(ctx, |ui| {
            if let Err(err) = self.explorer.err.as_ref() {
                ui.colored_label(Color32::YELLOW, err.to_string());
            }
            ui.colored_label(Color32::LIGHT_BLUE, self.explorer.get_filename());
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.explorer.ui(ui);
        });
    }
}

fn main() {
    let window_options = NativeOptions {
        drag_and_drop_support: true,
        ..Default::default()
    };
    run_native(
        "IdMyBee Markerzzzz",
        window_options,
        Box::new(|cc| {
            Box::new(MainState {
                explorer: FileExplorer::new(cc),
            })
        }),
    )
    .unwrap();
    // let options = eframe::Options::default();
    // eframe::run::<MainState>(options);
}
