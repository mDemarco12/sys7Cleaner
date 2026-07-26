
#[tauri::command]
fn scan_message() -> String {
    "System 7 Cleaner scan engine ready.".into()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![scan_message])
        .run(tauri::generate_context!())
        .expect("error");
}
