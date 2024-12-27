pub mod version_toml;
use version_toml::VersionData;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// 在后端是同步函数，在前端使用invoke时是异步调用
#[tauri::command]
fn get_version_data() -> Option<VersionData> {
    match version_toml::get_version_data() {
        Ok(data) => {
            println!("get_version_data: {data:?}");
            Some(data)
        }
        Err(e) => {
            println!("get_version_data Error: {:?}", e);
            None
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_version_data])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
