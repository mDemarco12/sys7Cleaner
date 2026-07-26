// Prevents an additional console window from appearing on Windows; irrelevant
// on macOS but kept as the standard Tauri entry-point boilerplate.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    system7_cleaner::run();
}
