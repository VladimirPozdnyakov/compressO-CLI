use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    English,
    Russian,
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Russian => "ru",
        }
    }
}

pub struct Localizer {
    current_language: Language,
    translations: HashMap<String, HashMap<String, String>>,
}

impl Localizer {
    pub fn new(language: Language) -> Self {
        let mut localizer = Localizer {
            current_language: language,
            translations: HashMap::new(),
        };

        // Initialize with English translations
        localizer.add_translations(Language::English, english_translations());

        // Initialize with Russian translations
        localizer.add_translations(Language::Russian, russian_translations());

        localizer
    }

    pub fn set_language(&mut self, language: Language) {
        self.current_language = language;
    }

    fn add_translations(&mut self, language: Language, translations: HashMap<String, String>) {
        self.translations.insert(language.code().to_string(), translations);
    }

    pub fn t(&self, key: &str) -> String {
        if let Some(lang_map) = self.translations.get(self.current_language.code()) {
            if let Some(value) = lang_map.get(key) {
                return value.clone();
            }
        }

        // Fallback to English if key not found in current language
        if self.current_language != Language::English {
            if let Some(lang_map) = self.translations.get("en") {
                if let Some(value) = lang_map.get(key) {
                    return value.clone();
                }
            }
        }

        // Return the key itself if no translation is found
        key.to_string()
    }
}

// English translations
fn english_translations() -> HashMap<String, String> {
    let mut translations = HashMap::new();

    // General terms
    translations.insert("app_name".to_string(), "CompressO CLI".to_string());
    translations.insert("app_version".to_string(), "v1.1.0".to_string());
    translations.insert("header_separator".to_string(), "━".repeat(50).to_string());
    translations.insert("compression_complete".to_string(), "Compression complete!".to_string());
    translations.insert("batch_compression_complete".to_string(), "Batch compression complete!".to_string());
    translations.insert("cancelled_by_user".to_string(), "Compression cancelled by user.".to_string());
    translations.insert("cancelled".to_string(), "Compression cancelled.".to_string());
    translations.insert("press_enter_to_exit".to_string(), "Press Enter to exit...".to_string());

    // Video information
    translations.insert("video_information".to_string(), "Video Information".to_string());
    translations.insert("file".to_string(), "File:".to_string());
    translations.insert("size".to_string(), "Size:".to_string());
    translations.insert("duration".to_string(), "Duration:".to_string());
    translations.insert("resolution".to_string(), "Resolution:".to_string());
    translations.insert("frame_rate".to_string(), "Frame rate:".to_string());

    // Compression settings
    translations.insert("compression_settings".to_string(), "Compression Settings".to_string());
    translations.insert("input".to_string(), "Input:".to_string());
    translations.insert("output".to_string(), "Output:".to_string());
    translations.insert("preset".to_string(), "Preset:".to_string());
    translations.insert("quality".to_string(), "Quality:".to_string());
    translations.insert("dimensions".to_string(), "Dimensions:".to_string());
    translations.insert("fps".to_string(), "FPS:".to_string());
    translations.insert("audio".to_string(), "Audio:".to_string());
    translations.insert("muted".to_string(), "muted".to_string());
    translations.insert("format".to_string(), "Format:".to_string());
    translations.insert("rotate".to_string(), "Rotate:".to_string());
    translations.insert("flip".to_string(), "Flip:".to_string());
    translations.insert("crop".to_string(), "Crop:".to_string());

    // Preset names
    translations.insert("thunderbolt_preset".to_string(), "thunderbolt (fast)".to_string());
    translations.insert("ironclad_preset".to_string(), "ironclad (quality)".to_string());

    // Progress and results
    translations.insert("original".to_string(), "Original:".to_string());
    translations.insert("compressed".to_string(), "Compressed:".to_string());
    translations.insert("saved".to_string(), "Saved:".to_string());
    translations.insert("time".to_string(), "Time:".to_string());
    translations.insert("processing".to_string(), "Processing".to_string());

    // Batch processing
    translations.insert("summary".to_string(), "Summary".to_string());
    translations.insert("total_files".to_string(), "Total files:".to_string());
    translations.insert("successful".to_string(), "Successful:".to_string());
    translations.insert("failed".to_string(), "Failed:".to_string());
    translations.insert("total_original".to_string(), "Total original:".to_string());
    translations.insert("total_compressed".to_string(), "Total compressed:".to_string());
    translations.insert("total_saved".to_string(), "Total saved:".to_string());
    translations.insert("total_time".to_string(), "Total time:".to_string());
    translations.insert("individual_results".to_string(), "Individual Results".to_string());

    // Interactive mode
    translations.insert("interactive_mode".to_string(), "Interactive Mode".to_string());
    translations.insert("drag_drop_video".to_string(), "Drag & drop video file here or enter path:".to_string());
    translations.insert("press_enter_without_input".to_string(), "(Press Enter without input to exit)".to_string());
    translations.insert("selected".to_string(), "Selected:".to_string());
    translations.insert("start_compression".to_string(), "Start compression?".to_string());
    translations.insert("no".to_string(), "No".to_string());
    translations.insert("yes".to_string(), "Yes".to_string());
    translations.insert("compression_cancelled".to_string(), "Compression cancelled.".to_string());

    // Advanced settings
    translations.insert("advanced_settings".to_string(), "Advanced Settings".to_string());
    translations.insert("transform_options".to_string(), "Transform Options".to_string());
    translations.insert("leave_empty_keep_original".to_string(), "(Leave empty to keep original)".to_string());
    translations.insert("remove_audio".to_string(), "Remove audio?".to_string());
    translations.insert("rotate_video".to_string(), "Rotate video".to_string());
    translations.insert("flip_horizontally".to_string(), "Flip horizontally (mirror)?".to_string());
    translations.insert("flip_vertically".to_string(), "Flip vertically?".to_string());
    translations.insert("crop_video".to_string(), "Crop video (format: WIDTHxHEIGHT:X:Y)".to_string());
    translations.insert("crop_example".to_string(), "Example: 1920x1080:0:0 (crop to 1920x1080 from top-left corner)".to_string());

    // Rotation options
    translations.insert("none_keep_original".to_string(), "None (keep original)".to_string());
    translations.insert("ninety_clockwise".to_string(), "90° clockwise".to_string());
    translations.insert("one_eighty".to_string(), "180°".to_string());
    translations.insert("two_seventy_clockwise".to_string(), "270° clockwise (90° counter-clockwise)".to_string());

    // Format options
    translations.insert("keep_original_format".to_string(), "Keep original format [default]".to_string());
    translations.insert("mp4_format".to_string(), "MP4".to_string());
    translations.insert("webm_format".to_string(), "WebM".to_string());
    translations.insert("mkv_format".to_string(), "MKV".to_string());
    translations.insert("avi_format".to_string(), "AVI".to_string());
    translations.insert("mov_format".to_string(), "MOV".to_string());

    // Preset options
    translations.insert("ironclad_slow_best_quality".to_string(), "Ironclad (slow, best quality) [default]".to_string());
    translations.insert("thunderbolt_fast_good_quality".to_string(), "Thunderbolt (fast, good quality)".to_string());

    // Size estimates
    translations.insert("original_size".to_string(), "Original size:".to_string());
    translations.insert("est_output".to_string(), "Est. output:".to_string());
    translations.insert("est_savings".to_string(), "Est. savings:".to_string());

    // Batch mode
    translations.insert("batch_compression_mode".to_string(), "Batch Compression Mode".to_string());
    translations.insert("video_files_found".to_string(), "video files found:".to_string());
    translations.insert("files_will_be_skipped".to_string(), "files will be skipped:".to_string());
    translations.insert("no_valid_video_files".to_string(), "No valid video files to process!".to_string());
    translations.insert("configure_advanced_settings".to_string(), "Configure advanced settings?".to_string());
    translations.insert("select_preset".to_string(), "Select preset".to_string());
    translations.insert("quality_prompt".to_string(), "Quality (0-100, higher = better)".to_string());
    translations.insert("output_format".to_string(), "Output format".to_string());
    translations.insert("width_prompt".to_string(), "Width (e.g., 1920)".to_string());
    translations.insert("height_prompt".to_string(), "Height (e.g., 1080)".to_string());
    translations.insert("fps_prompt".to_string(), "FPS (e.g., 30)".to_string());

    // Error messages
    translations.insert("file_not_found".to_string(), "File not found".to_string());
    translations.insert("not_a_valid_video_file".to_string(), "This is not a valid video file!".to_string());
    translations.insert("video_path".to_string(), "Video path".to_string());
    translations.insert("invalid_input_file".to_string(), "Invalid input file".to_string());
    translations.insert("invalid_output_path".to_string(), "Invalid output path".to_string());
    translations.insert("ffmpeg_not_found".to_string(), "FFmpeg not found. Please install FFmpeg or use bundled version.".to_string());
    translations.insert("ffmpeg_error".to_string(), "FFmpeg error".to_string());
    translations.insert("compression_cancelled_by_user".to_string(), "Compression cancelled by user".to_string());
    translations.insert("video_corrupted_or_unsupported".to_string(), "Video is corrupted or unsupported".to_string());
    translations.insert("io_error".to_string(), "IO error".to_string());

    translations
}

// Russian translations
fn russian_translations() -> HashMap<String, String> {
    let mut translations = HashMap::new();

    // General terms
    translations.insert("app_name".to_string(), "CompressO CLI".to_string());
    translations.insert("app_version".to_string(), "v1.1.0".to_string());
    translations.insert("header_separator".to_string(), "━".repeat(50).to_string());
    translations.insert("compression_complete".to_string(), "Сжатие завершено!".to_string());
    translations.insert("batch_compression_complete".to_string(), "Пакетное сжатие завершено!".to_string());
    translations.insert("cancelled_by_user".to_string(), "Сжатие отменено пользователем.".to_string());
    translations.insert("cancelled".to_string(), "Сжатие отменено.".to_string());
    translations.insert("press_enter_to_exit".to_string(), "Нажмите Enter для выхода...".to_string());

    // Video information
    translations.insert("video_information".to_string(), "Информация о видео".to_string());
    translations.insert("file".to_string(), "Файл:".to_string());
    translations.insert("size".to_string(), "Размер:".to_string());
    translations.insert("duration".to_string(), "Длительность:".to_string());
    translations.insert("resolution".to_string(), "Разрешение:".to_string());
    translations.insert("frame_rate".to_string(), "Частота кадров:".to_string());

    // Compression settings
    translations.insert("compression_settings".to_string(), "Настройки сжатия".to_string());
    translations.insert("input".to_string(), "Входной файл:".to_string());
    translations.insert("output".to_string(), "Выходной файл:".to_string());
    translations.insert("preset".to_string(), "Пресет:".to_string());
    translations.insert("quality".to_string(), "Качество:".to_string());
    translations.insert("dimensions".to_string(), "Размеры:".to_string());
    translations.insert("fps".to_string(), "FPS:".to_string());
    translations.insert("audio".to_string(), "Аудио:".to_string());
    translations.insert("muted".to_string(), "без звука".to_string());
    translations.insert("format".to_string(), "Формат:".to_string());
    translations.insert("rotate".to_string(), "Поворот:".to_string());
    translations.insert("flip".to_string(), "Отражение:".to_string());
    translations.insert("crop".to_string(), "Обрезка:".to_string());

    // Preset names
    translations.insert("thunderbolt_preset".to_string(), "thunderbolt (быстро)".to_string());
    translations.insert("ironclad_preset".to_string(), "ironclad (качество)".to_string());

    // Progress and results
    translations.insert("original".to_string(), "Оригинал:".to_string());
    translations.insert("compressed".to_string(), "Сжатый:".to_string());
    translations.insert("saved".to_string(), "Сэкономлено:".to_string());
    translations.insert("time".to_string(), "Время:".to_string());
    translations.insert("processing".to_string(), "Обработка".to_string());

    // Batch processing
    translations.insert("summary".to_string(), "Сводка".to_string());
    translations.insert("total_files".to_string(), "Всего файлов:".to_string());
    translations.insert("successful".to_string(), "Успешно:".to_string());
    translations.insert("failed".to_string(), "Ошибка:".to_string());
    translations.insert("total_original".to_string(), "Всего оригинальных:".to_string());
    translations.insert("total_compressed".to_string(), "Всего сжатых:".to_string());
    translations.insert("total_saved".to_string(), "Всего сэкономлено:".to_string());
    translations.insert("total_time".to_string(), "Общее время:".to_string());
    translations.insert("individual_results".to_string(), "Индивидуальные результаты".to_string());

    // Interactive mode
    translations.insert("interactive_mode".to_string(), "Интерактивный режим".to_string());
    translations.insert("drag_drop_video".to_string(), "Перетащите видеофайл сюда или введите путь:".to_string());
    translations.insert("press_enter_without_input".to_string(), "(Нажмите Enter без ввода для выхода)".to_string());
    translations.insert("selected".to_string(), "Выбрано:".to_string());
    translations.insert("start_compression".to_string(), "Начать сжатие?".to_string());
    translations.insert("no".to_string(), "Нет".to_string());
    translations.insert("yes".to_string(), "Да".to_string());
    translations.insert("compression_cancelled".to_string(), "Сжатие отменено.".to_string());

    // Advanced settings
    translations.insert("advanced_settings".to_string(), "Дополнительные настройки".to_string());
    translations.insert("transform_options".to_string(), "Параметры преобразования".to_string());
    translations.insert("leave_empty_keep_original".to_string(), "(Оставьте пустым, чтобы сохранить оригинал)".to_string());
    translations.insert("remove_audio".to_string(), "Удалить аудио?".to_string());
    translations.insert("rotate_video".to_string(), "Повернуть видео".to_string());
    translations.insert("flip_horizontally".to_string(), "Отразить по горизонтали (зеркало)?".to_string());
    translations.insert("flip_vertically".to_string(), "Отразить по вертикали?".to_string());
    translations.insert("crop_video".to_string(), "Обрезать видео (формат: ШИРИНАxВЫСОТА:X:Y)".to_string());
    translations.insert("crop_example".to_string(), "Пример: 1920x1080:0:0 (обрезать до 1920x1080 от левого верхнего угла)".to_string());

    // Rotation options
    translations.insert("none_keep_original".to_string(), "Без изменений (сохранить оригинал)".to_string());
    translations.insert("ninety_clockwise".to_string(), "90° по часовой стрелке".to_string());
    translations.insert("one_eighty".to_string(), "180°".to_string());
    translations.insert("two_seventy_clockwise".to_string(), "270° по часовой стрелке (90° против часовой стрелки)".to_string());

    // Format options
    translations.insert("keep_original_format".to_string(), "Сохранить исходный формат [по умолчанию]".to_string());
    translations.insert("mp4_format".to_string(), "MP4".to_string());
    translations.insert("webm_format".to_string(), "WebM".to_string());
    translations.insert("mkv_format".to_string(), "MKV".to_string());
    translations.insert("avi_format".to_string(), "AVI".to_string());
    translations.insert("mov_format".to_string(), "MOV".to_string());

    // Preset options
    translations.insert("ironclad_slow_best_quality".to_string(), "Ironclad (медленно, лучшее качество) [по умолчанию]".to_string());
    translations.insert("thunderbolt_fast_good_quality".to_string(), "Thunderbolt (быстро, хорошее качество)".to_string());

    // Size estimates
    translations.insert("original_size".to_string(), "Оригинальный размер:".to_string());
    translations.insert("est_output".to_string(), "Расч. вывод:".to_string());
    translations.insert("est_savings".to_string(), "Расч. экономия:".to_string());

    // Batch mode
    translations.insert("batch_compression_mode".to_string(), "Режим пакетного сжатия".to_string());
    translations.insert("video_files_found".to_string(), "видеофайлов найдено:".to_string());
    translations.insert("files_will_be_skipped".to_string(), "файлов будет пропущено:".to_string());
    translations.insert("no_valid_video_files".to_string(), "Нет допустимых видеофайлов для обработки!".to_string());
    translations.insert("configure_advanced_settings".to_string(), "Настроить дополнительные параметры?".to_string());
    translations.insert("select_preset".to_string(), "Выбрать пресет".to_string());
    translations.insert("quality_prompt".to_string(), "Качество (0-100, выше = лучше)".to_string());
    translations.insert("output_format".to_string(), "Формат вывода".to_string());
    translations.insert("width_prompt".to_string(), "Ширина (например, 1920)".to_string());
    translations.insert("height_prompt".to_string(), "Высота (например, 1080)".to_string());
    translations.insert("fps_prompt".to_string(), "FPS (например, 30)".to_string());

    // Error messages
    translations.insert("file_not_found".to_string(), "Файл не найден".to_string());
    translations.insert("not_a_valid_video_file".to_string(), "Это недействительный видеофайл!".to_string());
    translations.insert("video_path".to_string(), "Путь к видео".to_string());
    translations.insert("invalid_input_file".to_string(), "Недействительный входной файл".to_string());
    translations.insert("invalid_output_path".to_string(), "Недействительный путь вывода".to_string());
    translations.insert("ffmpeg_not_found".to_string(), "FFmpeg не найден. Пожалуйста, установите FFmpeg или используйте встроенную версию.".to_string());
    translations.insert("ffmpeg_error".to_string(), "Ошибка FFmpeg".to_string());
    translations.insert("compression_cancelled_by_user".to_string(), "Сжатие отменено пользователем".to_string());
    translations.insert("video_corrupted_or_unsupported".to_string(), "Видео повреждено или не поддерживается".to_string());
    translations.insert("io_error".to_string(), "Ошибка ввода-вывода".to_string());

    translations
}

// Global static instance of the localizer
use std::sync::Mutex;
use once_cell::sync::Lazy;

pub static LOCALIZER: Lazy<Mutex<Localizer>> = Lazy::new(|| Mutex::new(Localizer::new(Language::English)));

// Helper functions to access the global localizer
pub fn set_language(language: Language) {
    if let Ok(mut localizer) = LOCALIZER.lock() {
        localizer.set_language(language);
    }
}

pub fn t(key: &str) -> String {
    if let Ok(localizer) = LOCALIZER.lock() {
        localizer.t(key)
    } else {
        key.to_string()
    }
}
