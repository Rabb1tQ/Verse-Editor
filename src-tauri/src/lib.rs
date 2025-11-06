use std::env;
use std::process::Command;
use std::sync::Mutex;
use tauri::{Emitter, Manager};
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

// 存储启动时要打开的文件路径
struct StartupFile(Mutex<Option<String>>);

// 前端通知后端已准备好，返回是否有启动文件
#[tauri::command]
fn frontend_ready(app: tauri::AppHandle) -> Result<bool, String> {
    println!("前端已准备好");
    
    // 获取启动文件路径
    let startup_file = app.state::<StartupFile>();
    let file_path_option = startup_file.0.lock().unwrap().take();
    
    // 如果有启动文件，发送事件并返回 true
    if let Some(file_path) = file_path_option {
        println!("发送文件打开事件: {}", file_path);
        app.emit("file-open", FileOpenEvent {
            path: file_path,
        }).map_err(|e| e.to_string())?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![greet, open_file_location, frontend_ready])
        .setup(|app| {
            // 获取命令行参数
            let args: Vec<String> = env::args().collect();
            
            let mut startup_file_path: Option<String> = None;
            
            // 如果有文件路径参数，保存起来
            if args.len() > 1 {
                let file_path = args[1].clone();
                println!("检测到启动参数: {}", file_path);
                
                // 检查是否是支持的文件类型
                if file_path.ends_with(".md") || file_path.ends_with(".markdown") || file_path.ends_with(".txt") {
                    startup_file_path = Some(file_path);
                    println!("文件路径已保存，等待前端调用 frontend_ready");
                }
            }
            
            // 将启动文件保存到应用状态
            app.manage(StartupFile(Mutex::new(startup_file_path)));
            
            Ok(())
        })

        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
