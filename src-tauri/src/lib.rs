mod commands;
mod progress;

use commands::ScanRegistry;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ScanRegistry::default())
        .invoke_handler(tauri::generate_handler![
            commands::list_targets,
            commands::start_scan,
            commands::cancel_scan,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
