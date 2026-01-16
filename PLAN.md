# План преобразования compressO в CLI

## Текущая архитектура

- **Фронтенд**: React 18 + TypeScript + Vite + TailwindCSS
- **Бекенд**: Rust (Tauri 2.0)
- **Основная логика**: `src-tauri/src/lib/ffmpeg.rs` — сжатие видео через FFmpeg
- **Бинарники**: Предкомпилированные FFmpeg для Windows/macOS/Linux

## Целевая архитектура

Чистое Rust CLI-приложение без Tauri и React.

---

## Этапы реализации

### Этап 1: Подготовка проекта

1. **Создать новый Cargo проект** или адаптировать существующий `src-tauri`
2. **Удалить зависимости Tauri** из `Cargo.toml`
3. **Добавить CLI-библиотеки**:
   - `clap` — парсинг аргументов командной строки
   - `indicatif` — прогресс-бар в терминале
   - `colored` / `console` — цветной вывод

### Этап 2: Рефакторинг бизнес-логики

**Файлы для сохранения и адаптации:**
- `src-tauri/src/lib/ffmpeg.rs` — основная логика сжатия
- `src-tauri/src/lib/fs.rs` — файловые операции
- `src-tauri/src/lib/domain.rs` — структуры данных
- `src-tauri/bin/` — FFmpeg бинарники

**Изменения в `ffmpeg.rs`:**
1. Убрать `tauri::AppHandle` из всех функций
2. Заменить `app.shell().sidecar()` на прямой запуск FFmpeg
3. Убрать `app.emit()` — заменить на callback или канал для прогресса
4. Адаптировать `compress_video()` для работы без Tauri

### Этап 3: Реализация CLI интерфейса

**Структура команд:**

```
compresso <INPUT> [OUTPUT] [OPTIONS]

Arguments:
  <INPUT>   Путь к исходному видео файлу
  [OUTPUT]  Путь для сохранения (по умолчанию: input_compressed.ext)

Options:
  -q, --quality <0-100>     Качество сжатия (по умолчанию: 70)
  -p, --preset <PRESET>     Предустановка: thunderbolt (быстро) | ironclad (качество)
  -f, --format <FORMAT>     Выходной формат: mp4, mov, webm, avi, mkv
  -w, --width <WIDTH>       Ширина видео
  -h, --height <HEIGHT>     Высота видео
  --fps <FPS>               Частота кадров
  --mute                    Убрать звук
  --rotate <DEGREES>        Повернуть: 90, 180, 270
  --flip <DIRECTION>        Отразить: horizontal, vertical
  --crop <W:H:X:Y>          Обрезать видео
  -y, --yes                 Перезаписать без подтверждения
  -v, --verbose             Подробный вывод
  --help                    Показать справку
  --version                 Показать версию
```

**Примеры использования:**

```bash
# Базовое сжатие
compresso video.mp4

# С настройками качества
compresso video.mp4 -q 80 -p ironclad

# Конвертация формата
compresso video.mp4 output.webm -f webm

# Изменение размера и FPS
compresso video.mp4 -w 1280 -h 720 --fps 30

# Трансформации
compresso video.mp4 --rotate 90 --flip horizontal --mute
```

### Этап 4: Прогресс и вывод в терминал

```
compresso video.mp4 -q 75

CompressO CLI v1.0.0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Input:   video.mp4 (245.6 MB)
Output:  video_compressed.mp4
Preset:  thunderbolt
Quality: 75%

Compressing... ████████████████░░░░░░░░░░░░░░ 53% | 01:23 / 02:35

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✓ Compression complete!

Original:   245.6 MB
Compressed: 89.2 MB
Saved:      156.4 MB (63.7%)
Time:       2m 35s
```

### Этап 5: Структура файлов CLI-проекта

```
compressO-CLI/
├── src/
│   ├── main.rs           # Точка входа, CLI парсинг
│   ├── lib.rs            # Экспорт модулей
│   ├── cli.rs            # Определение CLI (clap)
│   ├── ffmpeg.rs         # Логика сжатия (из Tauri)
│   ├── fs.rs             # Файловые операции
│   ├── domain.rs         # Структуры данных
│   ├── progress.rs       # Прогресс-бар
│   └── error.rs          # Обработка ошибок
├── bin/                  # FFmpeg бинарники (копируем из src-tauri)
│   ├── ffmpeg-windows.exe
│   ├── ffmpeg-macos
│   └── ffmpeg-linux
├── Cargo.toml
├── build.rs              # Копирование FFmpeg при сборке
└── README.md
```

### Этап 6: Cargo.toml

```toml
[package]
name = "compresso"
version = "1.0.0"
edition = "2021"
description = "Fast video compression CLI tool"

[dependencies]
clap = { version = "4", features = ["derive"] }
indicatif = "0.17"
console = "0.15"
colored = "2"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
regex = "1"
image = "0.25"
infer = "0.15"
thiserror = "1"
which = "6"            # Поиск FFmpeg в PATH
directories = "5"      # Стандартные директории ОС

[build-dependencies]
embed-resource = "2"   # Для Windows иконки (опционально)
```

### Этап 7: Распространение FFmpeg

**Варианты:**

1. **Встроенный FFmpeg** (текущий подход)
   - Плюсы: Работает "из коробки"
   - Минусы: Большой размер бинарника (~80 MB)

2. **Системный FFmpeg**
   - Плюсы: Маленький размер CLI
   - Минусы: Требует установки FFmpeg пользователем

3. **Гибридный подход** (рекомендуется)
   - Сначала проверить системный FFmpeg (`which ffmpeg`)
   - Если нет — использовать встроенный или предложить установить

---

## Порядок реализации (по приоритету)

| # | Задача | Сложность |
|---|--------|-----------|
| 1 | Создать новый Cargo проект с CLI структурой | Низкая |
| 2 | Портировать `domain.rs` (структуры данных) | Низкая |
| 3 | Портировать `fs.rs` (файловые операции) | Низкая |
| 4 | Адаптировать `ffmpeg.rs` (убрать Tauri) | Средняя |
| 5 | Реализовать CLI парсинг с clap | Низкая |
| 6 | Добавить прогресс-бар (indicatif) | Низкая |
| 7 | Настроить сборку и FFmpeg бинарники | Средняя |
| 8 | Тестирование на всех платформах | Средняя |
| 9 | Документация и README | Низкая |

---

## Что удаляем

- Весь `src/` (React фронтенд)
- `package.json`, `pnpm-lock.yaml`
- `vite.config.ts`, `tailwind.config.ts`, `tsconfig.json`
- Tauri конфигурации (`tauri.conf.json`, `capabilities/`)
- Зависимости Tauri в Cargo.toml

## Что сохраняем

- Rust логику из `src-tauri/src/lib/`
- FFmpeg бинарники из `src-tauri/bin/`
- Алгоритмы сжатия и FFmpeg параметры
