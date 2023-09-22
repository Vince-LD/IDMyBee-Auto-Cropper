use configparser::ini::Ini;
use egui::Key;
use ucfirst::ucfirst;
pub struct AppShortcuts {
    pub crop_image: (Key, String),
    pub decrease_zoom: (Key, String),
    pub increase_zoom: (Key, String),
    pub next_file: (Key, String),
    pub previous_file: (Key, String),
    pub save_crop_image: (Key, String),
    pub select_input_dir: (Key, String),
    pub select_output_dir: (Key, String),
}

impl AppShortcuts {
    fn conf_to_key(config: &Ini, key_name: &str, default_key: &str) -> (Key, String) {
        let key_str = ucfirst(
            &config
                .get("shortcuts", key_name)
                .unwrap_or(String::from(default_key)),
        );

        let k: Key = serde_json::from_str(&format!("\"{}\"", key_str)).unwrap();
        (k, key_str)
    }

    pub fn new(config: &Ini) -> AppShortcuts {
        AppShortcuts {
            next_file: AppShortcuts::conf_to_key(config, "nex_file", "S"),
            previous_file: AppShortcuts::conf_to_key(config, "previous_file", "Z"),
            increase_zoom: AppShortcuts::conf_to_key(config, "increase_zoom", "D"),
            decrease_zoom: AppShortcuts::conf_to_key(config, "decrease_zoom", "Q"),
            crop_image: AppShortcuts::conf_to_key(config, "crop_image", "Space"),
            select_input_dir: AppShortcuts::conf_to_key(config, "select_input_dir", "F"),
            select_output_dir: AppShortcuts::conf_to_key(config, "select_output_dir", "V"),
            save_crop_image: AppShortcuts::conf_to_key(config, "save_crop_image", "R"),
        }
    }
}
