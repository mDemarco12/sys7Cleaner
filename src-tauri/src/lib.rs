mod commands;
mod progress;

use commands::{CustomTargets, ScanRegistry};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(ScanRegistry::default())
        .manage(CustomTargets::default())
        .invoke_handler(tauri::generate_handler![
            commands::list_targets,
            commands::add_custom_target,
            commands::remove_custom_target,
            commands::start_scan,
            commands::cancel_scan,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
