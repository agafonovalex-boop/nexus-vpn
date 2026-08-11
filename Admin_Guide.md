# NEXUS-VPN 1.0 — Инструкция для Администратора (Windows)

## Что это?
Полноценное Windows-приложение с GUI для:
- Подключения к серверу в Нидерландах по SSH
- Автоматической установки твоего Rust-сервера (с TUN + NAT + forwarding)
- Раздельного туннелирования
- Подключения/отключения одним кликом

## Требования
- Windows 10/11 (64-bit)
- Rust 1.80+ (установи с https://rustup.rs/)
- Node.js 20+ (установи с https://nodejs.org/)
- Tauri CLI (установи один раз: `cargo install tauri-cli`)

## Шаг 1: Создай проект
```bash
mkdir nexus-vpn && cd nexus-vpn
cargo init --bin src-tauri
