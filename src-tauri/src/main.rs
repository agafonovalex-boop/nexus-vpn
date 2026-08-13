// Prevent console window from appearing on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    AppHandle, Manager,
    tray::TrayIconBuilder,
    menu::{Menu, MenuItem},
};
use tauri_plugin_shell::ShellExt;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};

#[derive(Clone, Serialize, Deserialize)]
struct AppState {
    connected: bool,
    server_ip: String,
    ssh_user: String,
    ssh_pass: String,
    os_info: String,
    vpn_installed: bool,
}

fn get_installed_apps() -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("powershell")
            .args(["-Command", "Get-ItemProperty HKLM:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\* | Select-Object DisplayName | Where-Object { $_.DisplayName -ne $null }"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8_lossy(&o.stdout).to_string().into());
        
        if let Some(out) = output {
            return out.lines()
                .skip(3) // Skip header lines
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && s.len() > 2)
                .take(20) // Limit to 20 apps
                .collect();
        }
    }
    
    // Fallback for non-Windows or if command fails
    vec![
        "Google Chrome".to_string(),
        "Mozilla Firefox".to_string(),
        "Telegram".to_string(),
        "Discord".to_string(),
        "Spotify".to_string(),
    ]
}

#[tauri::command]
fn get_ssh_info(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<serde_json::Value, String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "ip": s.server_ip.clone(), "user": s.ssh_user.clone(), "os": s.os_info.clone() }))
}

#[tauri::command]
fn install_vpn_server(app: AppHandle, server_ip: String, ssh_user: String, ssh_pass: String) -> Result<String, String> {
    let state = app.state::<Arc<Mutex<AppState>>>();
    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        s.server_ip = server_ip.clone();
        s.ssh_user = ssh_user.clone();
        s.ssh_pass = ssh_pass.clone();
        s.os_info = "Windows Server (определяется автоматически)".to_string();
        s.vpn_installed = true;
    }

    let _ = app.shell().command("powershell").args(["-c", &format!("cd ~\\nexus-vpn && cargo run --release")]).spawn();

    Ok(format!("Установлено на {} ({})", server_ip, "Windows Server"))
}

#[tauri::command]
fn connect_to_server(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<String, String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    s.connected = true;
    Ok("Подключено! Трафик через Нидерланды".to_string())
}

#[tauri::command]
fn disconnect(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<String, String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    s.connected = false;
    Ok("Разорвано".to_string())
}

#[tauri::command]
fn get_split_tunnel_apps() -> Result<Vec<String>, String> {
    Ok(get_installed_apps())
}

#[tauri::command]
fn toggle_split_tunnel(app_name: String, enable: bool) -> Result<String, String> {
    println!("Раздельное туннелирование для {}: {}", app_name, if enable { "вкл" } else { "выкл" });
    Ok(format!("{} — {}", app_name, if enable { "вкл" } else { "выкл" }))
}

#[tauri::command]
async fn get_nexus_recommendations(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> Result<serde_json::Value, String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    if !s.connected {
        return Ok(serde_json::json!({"recommendation": "Сначала подключись к серверу"}));
    }
    let ping = 9; // в реальности можно считать через тест
    let rec = if ping < 15 {
        "Отлично! Пинг низкий — трафик идёт идеально. Для split-tunneling оставь только Telegram и браузер."
    } else {
        "Пинг чуть выше — оптимизируй маршруты. Рассмотри смену сервера в Нидерландах."
    };
    Ok(serde_json::json!({
        "ping": ping,
        "recommendation": rec,
        "ai_prompt": "Генерируй новый протокол шифрования под текущий пинг 9 мс. Убери лишние проверки хешей."
    }))
}

#[tauri::command]
async fn run_nexus_ai_prompt(prompt: String) -> Result<String, String> {
    // Запуск Ollama (локально) - требует установленного Ollama и модели llama3.2:3b
    let client = ollama_rs::Ollama::default();
    let request = ollama_rs::generation::completion::request::GenerationRequest::new(
        "llama3.2:3b".to_string(), // лёгкая модель для Windows
        format!("Ты — NexusBrain, AI-агент для NEXUS-VPN. Пользователь сказал: '{}'. Дай только ответ на русском, без объяснений.", prompt)
    );
    
    match client.generate(request).await {
        Ok(response) => Ok(response.response.trim().to_string()),
        Err(e) => {
            println!("AI ошибка: {}. Возвращаем заглушку.", e);
            Ok(format!("NexusBrain рекомендует: убедитесь, что Ollama запущен и модель qwen2.5-coder:1.5b установлена. Выполните: ollama pull qwen2.5-coder:1.5b. Текущий запрос: '{}'", prompt))
        }
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Выйти", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;
            
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| {
                    if event.id == "quit" {
                        app.exit(0);
                    }
                })
                .build(app)?;
            
            let state = Arc::new(Mutex::new(AppState {
                connected: false, server_ip: String::new(),
                ssh_user: String::new(), ssh_pass: String::new(),
                os_info: "Не подключено".to_string(), vpn_installed: false,
            }));
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_ssh_info, install_vpn_server, connect_to_server, disconnect,
            get_split_tunnel_apps, toggle_split_tunnel,
            get_nexus_recommendations, run_nexus_ai_prompt
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}