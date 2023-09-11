use eframe::{run_native, App, NativeOptions};
use egui::Context;
mod file_explorer;
use file_explorer::FileExplorer;

struct MainState {
    explorer: FileExplorer,
}

impl App for MainState {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
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
