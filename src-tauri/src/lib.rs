use std::env;
use tauri::{Manager, Emitter};
use serde::{Deserialize, Serialize};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct FileOpenEvent {
    path: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            // 获取命令行参数
            let args: Vec<String> = env::args().collect();
            
            // 如果有文件路径参数，发送文件打开事件
            if args.len() > 1 {
                let file_path = args[1].clone(); // 克隆字符串避免借用问题
                if file_path.ends_with(".md") || file_path.ends_with(".markdown") {
                    // 立即发送事件，并添加标记表示这是启动时打开的文件
                    let app_handle = app.handle().clone(); // 克隆 handle
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(100)); // 减少延迟
                        let _ = app_handle.emit("file-open", FileOpenEvent {
                            path: file_path,
                        });
                        // 发送标记表示这是启动时的文件打开
                        let _ = app_handle.emit("startup-file-opened", ());
                    });
                }
            }
            
            Ok(())
        })

        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
