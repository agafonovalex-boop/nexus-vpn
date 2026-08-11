--- README.md (原始)
# NEXUS-VPN 1.0 — Твой кастомный VPN-протокол

Полностью новый протокол (не WireGuard, не VLESS, не IKEv2).

## 📁 Скачиваемые файлы
- **Admin_Guide.md** — инструкция для администратора
- **User_Guide.md** — инструкция для пользователей

## 🚀 Быстрый старт

### Требования
- **Windows 10/11** (64-bit)
- **Rust 1.80+** — установи с [rustup.rs](https://rustup.rs/)
- **Node.js 20+** — установи с [nodejs.org](https://nodejs.org/)
- **Tauri CLI** — установи командой: `cargo install tauri-cli`
- **(Опционально) Ollama** — для AI-функций, установи с [ollama.com](https://ollama.com/) и модель: `ollama pull llama3.2:3b`

### Сборка приложения для Windows

1. **Открой PowerShell или Command Prompt** от имени администратора

2. **Перейди в папку проекта:**
   ```bash
   cd путь\к\workspace\nexus-vpn
   ```

3. **Установи Tauri CLI** (если ещё не установлен):
   ```bash
   cargo install tauri-cli
   ```

4. **Запусти сборку:**
   ```bash
   cargo tauri build --release
   ```

5. **Найди готовые файлы:**
   После сборки твои файлы будут в папке:
   ```
   src-tauri\target\release\bundle\msi\
   ```
   или
   ```
   src-tauri\target\release\bundle\nsis\
   ```

   Там ты найдёшь:
   - `NEXUS-VPN_1.0.0_x64.msi` — установщик MSI
   - `NEXUS-VPN_1.0.0_x64-setup.exe` — установщик NSIS (если включён)

### Запуск в режиме разработки

Для тестирования без сборки:
```bash
cargo tauri dev
```

## 📋 Структура проекта

```
nexus-vpn/
├── src-tauri/
│   ├── src/
│   │   └── main.rs          # Rust бэкенд
│   ├── assets/
│   │   └── index.html       # HTML/CSS/JS фронтенд
│   ├── icons/               # Иконки приложения
│   ├── Cargo.toml           # Rust зависимости
│   ├── build.rs             # Build скрипт
│   └── tauri.conf.json      # Конфигурация Tauri
├── Admin_Guide.md           # Инструкция администратора
├── User_Guide.md            # Инструкция пользователя
├── README.md                # Этот файл
└── tauri.conf.json          # Корневая конфигурация
```

## 🔧 Функции приложения

- ✅ Подключение к серверу в Нидерландах по SSH
- ✅ Автоматическая установка VPN-сервера
- ✅ Раздельное туннелирование (выбор приложений)
- ✅ AI-ассистент NexusBrain (требует Ollama)
- ✅ Системный трей с быстрым доступом
- ✅ Минималистичный современный UI

## ⚠️ Важно

- Для работы AI-функций необходим локально установленный **Ollama** с моделью `llama3.2:3b`
- Приложение создаёт окно консоли только в режиме отладки. В релизе окно скрыто
- Для установки VPN-сервера требуются права администратора на удалённой машине

## 🛠️ Решение проблем

### Ошибка при сборке "cannot find tauri.conf.json"
Убедись, что запускаешь команду из корневой папки проекта.

### Ошибка "package not found"
Выполни: `cargo update` в папке `src-tauri`

### AI не работает
1. Установи Ollama: https://ollama.com/download/windows
2. Выполни в терминале: `ollama pull llama3.2:3b`
3. Убедись, что Ollama запущен: `ollama serve`

---

**Собрано с помощью Tauri v2 | Rust | HTML/CSS/JS**
