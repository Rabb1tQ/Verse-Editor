use std::env;
use std::process::Command;
use tauri::{Emitter};
use serde::{Deserialize, Serialize};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// 打开文件所在位置并选中文件
#[tauri::command]
fn open_file_location(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // Windows: 使用 explorer /select 命令
        Command::new("explorer")
            .args(["/select,", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "macos")]
    {
        // macOS: 使用 open -R 命令
        Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "linux")]
    {
        // Linux: 打开文件所在目录（大多数文件管理器不支持选中功能）
        let parent = std::path::Path::new(&path)
            .parent()
            .ok_or("无法获取父目录")?;
        Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    
    Ok(())
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
        .invoke_handler(tauri::generate_handler![greet, open_file_location])
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
