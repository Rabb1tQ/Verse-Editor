use std::env;
use std::process::Command;
use std::sync::Mutex;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{Emitter, Manager};
use serde::{Deserialize, Serialize};
use notify::{Watcher, RecursiveMode, Result as NotifyResult};
use notify_debouncer_full::{new_debouncer, DebounceEventResult};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
struct FileChangeEvent {
    path: String,
    event_type: String, // "modified", "deleted", "created"
}

// 存储启动时要打开的文件路径
struct StartupFile(Mutex<Option<String>>);

// 存储文件监控器（需要保持活动状态）
struct FileWatcherState {
    watchers: Mutex<Vec<Box<dyn std::any::Any + Send>>>,
}

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

// 监控目录中的文件变化
#[tauri::command]
fn watch_directory(app: tauri::AppHandle, path: String) -> Result<(), String> {
    println!("开始监控目录: {}", path);
    
    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() {
        return Err(format!("路径不存在: {}", path));
    }
    
    let app_handle = app.clone();
    
    // 创建防抖动的文件监控器
    let mut debouncer = new_debouncer(
        Duration::from_millis(500),
        None,
        move |result: DebounceEventResult| {
            match result {
                Ok(events) => {
                    for event in events {
                        for path in &event.paths {
                            let path_str = path.to_string_lossy().to_string();
                            
                            // 只处理图片文件
                            let is_image = path_str.ends_with(".png") 
                                || path_str.ends_with(".jpg") 
                                || path_str.ends_with(".jpeg")
                                || path_str.ends_with(".gif")
                                || path_str.ends_with(".svg")
                                || path_str.ends_with(".webp")
                                || path_str.ends_with(".bmp");
                            
                            if !is_image {
                                continue;
                            }
                            
                            let event_type = match event.kind {
                                notify::EventKind::Create(_) => "created",
                                notify::EventKind::Modify(_) => "modified",
                                notify::EventKind::Remove(_) => "deleted",
                                _ => continue,
                            };
                            
                            println!("文件变化: {} - {}", event_type, path_str);
                            
                            // 发送文件变化事件到前端
                            let _ = app_handle.emit("file-changed", FileChangeEvent {
                                path: path_str,
                                event_type: event_type.to_string(),
                            });
                        }
                    }
                }
                Err(errors) => {
                    for error in errors {
                        eprintln!("文件监控错误: {:?}", error);
                    }
                }
            }
        },
    ).map_err(|e| format!("创建文件监控器失败: {}", e))?;
    
    // 开始监控目录
    debouncer
        .watcher()
        .watch(&path_buf, RecursiveMode::Recursive)
        .map_err(|e| format!("监控目录失败: {}", e))?;
    
    // 将监控器存储到状态中，防止被销毁
    let watcher_state = app.state::<FileWatcherState>();
    watcher_state.watchers.lock().unwrap().push(Box::new(debouncer));
    
    println!("目录监控已启动: {}", path);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![greet, open_file_location, frontend_ready, watch_directory])
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
            
            // 初始化文件监控器状态
            app.manage(FileWatcherState {
                watchers: Mutex::new(Vec::new()),
            });
            
            Ok(())
        })

        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
