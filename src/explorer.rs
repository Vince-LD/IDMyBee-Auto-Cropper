use egui::{Color32, Ui};
use std::fs;
use std::path::PathBuf;

pub struct FileExplorer {
    current_path: PathBuf,
}

impl FileExplorer {
    pub fn new(_: &eframe::CreationContext<'_>) -> Self {
        Self {
            current_path: dirs::home_dir().unwrap_or_default(),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        // Affichez le chemin actuel en tant qu'en-tête.
        ui.horizontal(|ui| {
            ui.label("Current Path:");
            ui.monospace(self.current_path.display().to_string());
        });

        // Liste des fichiers et dossiers dans le répertoire actuel.
        if let Ok(entries) = fs::read_dir(&self.current_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let entry_path = entry.path();
                    let entry_name = entry_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    // Affichez les dossiers en bleu et les fichiers en noir.
                    let color = if entry_path.is_dir() {
                        Color32::BLUE
                    } else {
                        Color32::WHITE
                    };

                    ui.horizontal(|ui| {
                        ui.colored_label(color, entry_name);

                        // Si l'élément est un dossier, ajoutez un bouton pour naviguer.
                        if entry_path.is_dir() && ui.button("Open").clicked() {
                            self.current_path = entry_path;
                        }
                    });
                }
            }
        }
    }
}
