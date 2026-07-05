// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tauri::command]
async fn read_trace_file(path: String) -> Result<String, String> {
    const MAX_BYTES: u64 = 256 * 1024 * 1024; // 256MB safety limit

    let metadata = tokio::fs::metadata(&path)
        .await
        .map_err(|e| format!("Failed to stat file: {}", e))?;

    if metadata.len() > MAX_BYTES {
        return Err(format!(
            "Trace file too large ({} bytes, max {} bytes)",
            metadata.len(),
            MAX_BYTES
        ));
    }

    tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![read_trace_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run()
}
