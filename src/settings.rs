//! settings, 1:1 with `web/src/lib/settings/defaults.ts` (schema version 6),
//! plus a small `desktop` section for things a browser handles by itself.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const THEME_OPTIONS: &[&str] = &["auto", "light", "dark"];
pub const AUDIO_BITRATE_OPTIONS: &[&str] = &["320", "256", "128", "96", "64", "8"];
pub const AUDIO_FORMAT_OPTIONS: &[&str] = &["best", "mp3", "ogg", "wav", "opus"];
pub const DOWNLOAD_MODE_OPTIONS: &[&str] = &["auto", "audio", "mute"];
pub const FILENAME_STYLE_OPTIONS: &[&str] = &["classic", "basic", "pretty", "nerdy"];
pub const VIDEO_QUALITY_OPTIONS: &[&str] =
    &["max", "2160", "1440", "1080", "720", "480", "360", "240", "144"];
pub const YOUTUBE_VIDEO_CODEC_OPTIONS: &[&str] = &["h264", "av1", "vp9"];
pub const YOUTUBE_VIDEO_CONTAINER_OPTIONS: &[&str] = &["auto", "mp4", "webm", "mkv"];
pub const LOCAL_PROCESSING_OPTIONS: &[&str] = &["disabled", "preferred", "forced"];
/// "share" exists in the web app for mobile share sheets; desktops don't have one.
pub const SAVING_METHOD_OPTIONS: &[&str] = &["ask", "download", "copy"];

/// same list as `web/src/lib/settings/audio-sub-language.ts`
pub const LANGUAGES: &[&str] = &[
    "en", "es", "pt", "fr", "ru", "zh", "vi", "hi", "bn", "ja", "af", "am", "ar", "as", "az", "be",
    "bg", "bs", "ca", "cs", "da", "de", "el", "et", "eu", "fa", "fi", "fil", "gl", "gu", "hr", "hu",
    "hy", "id", "is", "it", "iw", "ka", "kk", "ko", "km", "kn", "ky", "lo", "lt", "lv", "mk", "ml",
    "mn", "mr", "ms", "my", "no", "ne", "nl", "or", "pa", "pl", "ro", "si", "sk", "sl", "sq", "sr",
    "sv", "sw", "ta", "te", "th", "tr", "uk", "ur", "uz", "zh-Hans", "zh-Hant", "zh-CN", "zh-HK",
    "zh-TW", "zu",
];

/// english display names for the language codes (the web app uses `Intl.DisplayNames`).
/// english is used for every entry so the list renders with the bundled fonts.
pub fn language_name(code: &str) -> &'static str {
    match code {
        "en" => "English", "es" => "Spanish", "pt" => "Portuguese", "fr" => "French", "ru" => "Russian",
        "zh" => "Chinese", "vi" => "Vietnamese", "hi" => "Hindi", "bn" => "Bengali", "ja" => "Japanese",
        "af" => "Afrikaans", "am" => "Amharic", "ar" => "Arabic", "as" => "Assamese", "az" => "Azerbaijani",
        "be" => "Belarusian", "bg" => "Bulgarian", "bs" => "Bosnian", "ca" => "Catalan", "cs" => "Czech",
        "da" => "Danish", "de" => "German", "el" => "Greek", "et" => "Estonian", "eu" => "Basque",
        "fa" => "Persian", "fi" => "Finnish", "fil" => "Filipino", "gl" => "Galician", "gu" => "Gujarati",
        "hr" => "Croatian", "hu" => "Hungarian", "hy" => "Armenian", "id" => "Indonesian", "is" => "Icelandic",
        "it" => "Italian", "iw" => "Hebrew", "ka" => "Georgian", "kk" => "Kazakh", "ko" => "Korean",
        "km" => "Khmer", "kn" => "Kannada", "ky" => "Kyrgyz", "lo" => "Lao", "lt" => "Lithuanian",
        "lv" => "Latvian", "mk" => "Macedonian", "ml" => "Malayalam", "mn" => "Mongolian", "mr" => "Marathi",
        "ms" => "Malay", "my" => "Burmese", "no" => "Norwegian", "ne" => "Nepali", "nl" => "Dutch",
        "or" => "Odia", "pa" => "Punjabi", "pl" => "Polish", "ro" => "Romanian", "si" => "Sinhala",
        "sk" => "Slovak", "sl" => "Slovenian", "sq" => "Albanian", "sr" => "Serbian", "sv" => "Swedish",
        "sw" => "Swahili", "ta" => "Tamil", "te" => "Telugu", "th" => "Thai", "tr" => "Turkish",
        "uk" => "Ukrainian", "ur" => "Urdu", "uz" => "Uzbek", "zh-Hans" => "Chinese (Simplified)",
        "zh-Hant" => "Chinese (Traditional)", "zh-CN" => "Chinese (China)", "zh-HK" => "Chinese (Hong Kong)",
        "zh-TW" => "Chinese (Taiwan)", "zu" => "Zulu",
        _ => "Unknown",
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Advanced {
    pub debug: bool,
    pub use_web_codecs: bool,
}
impl Default for Advanced {
    fn default() -> Self {
        Self { debug: false, use_web_codecs: false }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Appearance {
    pub theme: String,
    pub language: String,
    pub auto_language: bool,
    pub hide_remux_tab: bool,
}
impl Default for Appearance {
    fn default() -> Self {
        Self {
            theme: "auto".into(),
            language: crate::i18n::DEFAULT_LOCALE.into(),
            auto_language: true,
            hide_remux_tab: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Accessibility {
    pub reduce_motion: bool,
    pub reduce_transparency: bool,
    pub disable_haptics: bool,
    pub dont_auto_open_queue: bool,
}
impl Default for Accessibility {
    fn default() -> Self {
        Self {
            reduce_motion: false,
            reduce_transparency: false,
            disable_haptics: false,
            dont_auto_open_queue: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Save {
    pub always_proxy: bool,
    pub local_processing: String,
    pub audio_bitrate: String,
    pub audio_format: String,
    pub disable_metadata: bool,
    pub download_mode: String,
    pub filename_style: String,
    pub saving_method: String,
    pub allow_h265: bool,
    pub tiktok_full_audio: bool,
    pub convert_gif: bool,
    pub video_quality: String,
    pub subtitle_lang: String,
    pub youtube_video_codec: String,
    pub youtube_video_container: String,
    pub youtube_dub_lang: String,
    #[serde(rename = "youtubeHLS")]
    pub youtube_hls: bool,
    pub youtube_better_audio: bool,
}
impl Default for Save {
    fn default() -> Self {
        Self {
            always_proxy: false,
            // the desktop always supports local processing (ffmpeg), like a capable browser
            local_processing: "preferred".into(),
            audio_bitrate: "128".into(),
            audio_format: "mp3".into(),
            disable_metadata: false,
            download_mode: "auto".into(),
            filename_style: "basic".into(),
            saving_method: "download".into(),
            allow_h265: false,
            tiktok_full_audio: false,
            convert_gif: true,
            video_quality: "1080".into(),
            subtitle_lang: "none".into(),
            youtube_video_codec: "h264".into(),
            youtube_video_container: "auto".into(),
            youtube_dub_lang: "original".into(),
            youtube_hls: false,
            youtube_better_audio: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Privacy {
    pub disable_analytics: bool,
}
impl Default for Privacy {
    fn default() -> Self {
        Self { disable_analytics: false }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Processing {
    #[serde(rename = "customInstanceURL")]
    pub custom_instance_url: String,
    pub custom_api_key: String,
    pub enable_custom_instances: bool,
    pub enable_custom_api_key: bool,
    pub seen_custom_warning: bool,
}
impl Default for Processing {
    fn default() -> Self {
        Self {
            custom_instance_url: String::new(),
            custom_api_key: String::new(),
            enable_custom_instances: false,
            enable_custom_api_key: false,
            seen_custom_warning: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Desktop {
    pub download_dir: String,
    pub ffmpeg_path: String,
}
impl Default for Desktop {
    fn default() -> Self {
        Self { download_dir: String::new(), ffmpeg_path: String::new() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct CobaltSettings {
    pub schema_version: u32,
    pub advanced: Advanced,
    pub appearance: Appearance,
    pub accessibility: Accessibility,
    pub save: Save,
    pub privacy: Privacy,
    pub processing: Processing,
    pub desktop: Desktop,
}

impl Default for CobaltSettings {
    fn default() -> Self {
        Self {
            schema_version: 6,
            advanced: Default::default(),
            appearance: Default::default(),
            accessibility: Default::default(),
            save: Default::default(),
            privacy: Default::default(),
            processing: Default::default(),
            desktop: Default::default(),
        }
    }
}

pub const OFFICIAL_API_URL: &str = "https://api.cobalt.tools";

impl CobaltSettings {
    pub fn config_dir() -> PathBuf {
        directories::ProjectDirs::from("net", "imput", "cobalt-desktop")
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    }

    pub fn cache_dir() -> PathBuf {
        directories::ProjectDirs::from("net", "imput", "cobalt-desktop")
            .map(|d| d.cache_dir().to_path_buf())
            .unwrap_or_else(|| std::env::temp_dir().join("cobalt-desktop"))
    }

    pub fn path() -> PathBuf {
        Self::config_dir().join("settings.json")
    }

    pub fn load() -> Self {
        let path = Self::path();
        match std::fs::read_to_string(&path) {
            Ok(text) => Self::from_json(&text).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn from_json(text: &str) -> anyhow::Result<Self> {
        let value: serde_json::Value = serde_json::from_str(text)?;
        if !value.is_object() {
            anyhow::bail!("not an object");
        }
        let mut s: CobaltSettings = serde_json::from_value(value)?;
        s.validate();
        Ok(s)
    }

    pub fn save(&self) {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(text) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, text);
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// make sure every enum-like value is one of the accepted options
    pub fn validate(&mut self) {
        fn fix(v: &mut String, options: &[&str], default: &str) {
            if !options.contains(&v.as_str()) {
                *v = default.to_string();
            }
        }
        let d = CobaltSettings::default();
        fix(&mut self.appearance.theme, THEME_OPTIONS, &d.appearance.theme);
        fix(&mut self.save.audio_bitrate, AUDIO_BITRATE_OPTIONS, &d.save.audio_bitrate);
        fix(&mut self.save.audio_format, AUDIO_FORMAT_OPTIONS, &d.save.audio_format);
        fix(&mut self.save.download_mode, DOWNLOAD_MODE_OPTIONS, &d.save.download_mode);
        fix(&mut self.save.filename_style, FILENAME_STYLE_OPTIONS, &d.save.filename_style);
        fix(&mut self.save.video_quality, VIDEO_QUALITY_OPTIONS, &d.save.video_quality);
        fix(&mut self.save.youtube_video_codec, YOUTUBE_VIDEO_CODEC_OPTIONS, &d.save.youtube_video_codec);
        fix(&mut self.save.youtube_video_container, YOUTUBE_VIDEO_CONTAINER_OPTIONS, &d.save.youtube_video_container);
        fix(&mut self.save.local_processing, LOCAL_PROCESSING_OPTIONS, &d.save.local_processing);
        fix(&mut self.save.saving_method, SAVING_METHOD_OPTIONS, &d.save.saving_method);
        if self.save.subtitle_lang != "none" && !LANGUAGES.contains(&self.save.subtitle_lang.as_str()) {
            self.save.subtitle_lang = "none".into();
        }
        if self.save.youtube_dub_lang != "original" && !LANGUAGES.contains(&self.save.youtube_dub_lang.as_str()) {
            self.save.youtube_dub_lang = "original".into();
        }
        self.schema_version = 6;
    }

    /// `currentApiURL()` from `web/src/lib/api/api-url.ts`
    pub fn api_url(&self) -> String {
        let p = &self.processing;
        if p.enable_custom_instances && !p.custom_instance_url.is_empty() {
            if let Ok(u) = url::Url::parse(&p.custom_instance_url) {
                return u.origin().ascii_serialization();
            }
        }
        OFFICIAL_API_URL.to_string()
    }

    pub fn is_custom_instance(&self) -> bool {
        self.api_url() != OFFICIAL_API_URL
    }

    pub fn download_dir(&self) -> PathBuf {
        if !self.desktop.download_dir.is_empty() {
            return PathBuf::from(&self.desktop.download_dir);
        }
        directories::UserDirs::new()
            .and_then(|u| u.download_dir().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."))
    }

    pub fn effective_locale(&self) -> String {
        if self.appearance.auto_language {
            crate::i18n::system_locale()
        } else {
            self.appearance.language.clone()
        }
    }
}
