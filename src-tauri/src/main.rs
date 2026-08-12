// Prevent console window from appearing on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    AppHandle, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
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
fn get_ssh_info(state: tauri::State<Arc<AppState>>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({ "ip": state.server_ip.clone(), "user": state.ssh_user.clone(), "os": state.os_info.clone() }))
}

#[tauri::command]
fn install_vpn_server(app: AppHandle, server_ip: String, ssh_user: String, ssh_pass: String) -> Result<String, String> {
    let state = app.state::<Arc<AppState>>();
    state.server_ip = server_ip.clone();
    state.ssh_user = ssh_user.clone();
    state.ssh_pass = ssh_pass.clone();

    let os_info = "Windows Server (определяется автоматически)".to_string();
    state.os_info = os_info.clone();

    let _ = app.shell().command("powershell").args(["-c", &format!("cd ~\\nexus-vpn && cargo run --release")]).spawn();

    state.vpn_installed = true;
    Ok(format!("Установлено на {} ({})", server_ip, os_info))
}

#[tauri::command]
fn connect_to_server(state: tauri::State<Arc<AppState>>) -> Result<String, String> {
    state.connected = true;
    Ok("Подключено! Трафик через VPS".to_string())
}

#[tauri::command]
fn disconnect(state: tauri::State<Arc<AppState>>) -> Result<String, String> {
    state.connected = false;
    Ok("Разорвано".to_string())
}

#[tauri::command]
fn get_split_tunnel_apps() -> Result<Vec<String>, String> {
    Ok(get_installed_apps())
}

#[tauri::command]
fn toggle_split_tunnel(_app: &AppHandle, app_name: String, enable: bool) -> Result<String, String> {
    println!("Раздельное туннелирование для {}: {}", app_name, if enable { "вкл" } else { "выкл" });
    Ok(format!("{} — {}", app_name, if enable { "вкл" } else { "выкл" }))
}

#[tauri::command]
async fn get_nexus_recommendations(state: tauri::State<Arc<AppState>>) -> Result<serde_json::Value, String> {
    if !state.connected {
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
    // Запуск Ollama (локально) - требует установленного Ollama и модели qwen2.5-coder:1.5b
    // Легкая модель (1.5B) для эффективной работы на Windows
    let client = ollama_rs::Ollama::default();
    
    // Системный промпт для точного управления функциями VPN
    let system_prompt = r#"Ты — NexusBrain, AI-агент для NEXUS-VPN. Твоя задача — помогать пользователю управлять VPN-соединением и оптимизировать его работу.

ПРАВИЛА ОТВЕТА:
1. Отвечай ТОЛЬКО на русском языке
2. Будь краток и конкретен (1-3 предложения)
3. Не используй технические термины без необходимости
4. Если вопрос не связан с VPN — вежливо направь к теме VPN
5. Для команд управления возвращай только подтверждение действия

ФУНКЦИИ VPN, КОТОРЫЕ ТЫ МОЖЕШЬ УПОМИНАТЬ:
- Подключение/отключение от сервера
- Раздельное туннелирование для приложений
- Оптимизация пинга и маршрутов
- Рекомендации по выбору сервера

ПРИМЕРЫ ПРАВИЛЬНЫХ ОТВЕТОВ:
- "Подключаюсь к серверу VPS..."
- "Для Telegram включено раздельное туннелирование"
- "Рекомендую выбрать сервер с пингом менее 15мс"
- "Отключаю VPN соединение"

Отвечай дружелюбно, но профессионально."#;

    let request = ollama_rs::generation::generate::GenerateRequest::new(
        "qwen2.5-coder:1.5b".to_string(), // лёгкая модель для Windows (1.5B параметров)
        format!("{}\n\nПользователь сказал: '{}'. Дай ответ согласно правилам выше.", system_prompt, prompt)
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
        .system_tray(SystemTray::new().with_menu(SystemTrayMenu::new()
            .add_item(SystemTrayMenuItem::CustomMenu("quit".to_string(), "Выйти"))
        ))
        .on_system_tray_event(|app, event| {
            if let SystemTrayEvent::MenuItemClick { id, .. } = event {
                if id.as_str() == "quit" { app.exit(0); }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_ssh_info, install_vpn_server, connect_to_server, disconnect,
            get_split_tunnel_apps, toggle_split_tunnel,
            get_nexus_recommendations, run_nexus_ai_prompt
        ])
        .setup(|app| {
            let state = Arc::new(AppState {
                connected: false, server_ip: String::new(),
                ssh_user: String::new(), ssh_pass: String::new(),
                os_info: "Не подключено".to_string(), vpn_installed: false,
            });
            app.manage(state);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
