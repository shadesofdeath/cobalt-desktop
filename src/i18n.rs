//! translations, loaded from the same json files the cobalt web app uses.
//! keys are namespaced like `save.paste`, `settings.video.quality`,
//! `error.api.unreachable`, `error.queue.fetch.crashed` (same as the web app).

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::collections::HashMap;

macro_rules! locale_files {
    ($lang:literal) => {
        &[
            ("", include_str!(concat!("../assets/i18n/", $lang, "/general.json"))),
            ("general", include_str!(concat!("../assets/i18n/", $lang, "/general.json"))),
            ("save", include_str!(concat!("../assets/i18n/", $lang, "/save.json"))),
            ("settings", include_str!(concat!("../assets/i18n/", $lang, "/settings.json"))),
            ("error", include_str!(concat!("../assets/i18n/", $lang, "/error.json"))),
            ("error.api", include_str!(concat!("../assets/i18n/", $lang, "/error/api.json"))),
            ("error.queue", include_str!(concat!("../assets/i18n/", $lang, "/error/queue.json"))),
            ("button", include_str!(concat!("../assets/i18n/", $lang, "/button.json"))),
            ("tabs", include_str!(concat!("../assets/i18n/", $lang, "/tabs.json"))),
            ("queue", include_str!(concat!("../assets/i18n/", $lang, "/queue.json"))),
            ("dialog", include_str!(concat!("../assets/i18n/", $lang, "/dialog.json"))),
            ("remux", include_str!(concat!("../assets/i18n/", $lang, "/remux.json"))),
            ("about", include_str!(concat!("../assets/i18n/", $lang, "/about.json"))),
            ("donate", include_str!(concat!("../assets/i18n/", $lang, "/donate.json"))),
            ("updates", include_str!(concat!("../assets/i18n/", $lang, "/updates.json"))),
            ("receiver", include_str!(concat!("../assets/i18n/", $lang, "/receiver.json"))),
            ("notification", include_str!(concat!("../assets/i18n/", $lang, "/notification.json"))),
        ]
    };
}

const EN: &[(&str, &str)] = locale_files!("en");
const RU: &[(&str, &str)] = locale_files!("ru");

pub const LOCALES: &[(&str, &str)] = &[("en", "english"), ("ru", "русский")];
pub const DEFAULT_LOCALE: &str = "en";

/// strings that exist only in the desktop client (not in the web app).
const DESKTOP_EXTRA_EN: &[(&str, &str)] = &[
    ("error.api.turnstile_desktop", "this processing instance requires a cloudflare turnstile (captcha) check, which can only be done in a web browser. the desktop client can't pass it.\n\nuse a self-hosted or community instance (settings > instances), or add an instance access key if the owner gave you one."),
    ("error.ffmpeg.not_found", "ffmpeg wasn't found. install ffmpeg and make sure it's in PATH, or place ffmpeg.exe next to the app. local processing and remuxing need it."),
    ("error.save.failed", "couldn't save the file:\n{{ value }}"),
    ("settings.page.desktop", "desktop"),
    ("settings.desktop.downloads", "download folder"),
    ("settings.desktop.downloads.title", "save files to"),
    ("settings.desktop.downloads.description", "files downloaded with the \"download\" saving method will be saved to this folder. the \"ask\" method will show a file dialog every time instead."),
    ("settings.desktop.downloads.choose", "choose folder"),
    ("settings.desktop.downloads.open", "open folder"),
    ("settings.desktop.ffmpeg", "ffmpeg"),
    ("settings.desktop.ffmpeg.title", "ffmpeg path"),
    ("settings.desktop.ffmpeg.description", "leave empty to use ffmpeg from PATH or from the app folder. ffmpeg is used for on-device remuxing and transcoding, exactly like libav.js is in the web app."),
    ("settings.desktop.ffmpeg.found", "found: {{ value }}"),
    ("settings.desktop.ffmpeg.missing", "ffmpeg not found"),
    ("settings.desktop.window", "window"),
    ("settings.desktop.window.close_to_tray.title", "keep the app compact"),
    ("dialog.saving.open_folder", "show in folder"),
    ("dialog.saving.open_browser", "open in browser"),
    ("dialog.saving.copy_link", "copy link"),
    ("dialog.saving.download", "download"),
    ("dialog.saving.saved", "saved to {{ value }}"),
    ("queue.state.running.proxy", "downloading"),
    ("about.page.desktop", "desktop client"),
    ("about.heading.desktop", "about this desktop client"),
    ("save.label.instance", "instance: {{ value }}"),
    ("save.label.local_instance", "local instance"),
    ("save.label.official_instance", "official instance"),
    ("remux.select_files", "select files"),
    ("dialog.error.title", "something went wrong"),
    ("dialog.notice.title", "heads up"),
    ("toast.copied", "copied to clipboard"),
    ("toast.saved", "saved to {{ value }}"),
    ("toast.queued", "added to the queue"),
    ("home.title", "save anything from the web"),
    ("home.subtitle", "paste a link from youtube, tiktok, instagram, twitter and more. pick a mode, hit download."),
    ("home.download", "download"),
    ("home.services.show", "show all supported services"),
    ("home.services.hide", "hide services"),
    ("home.activity", "activity"),
    ("home.activity.empty.title", "nothing here yet"),
    ("home.activity.empty.description", "downloads and remuxes will show up here with live progress."),
    ("home.presets", "current preset"),
    ("nav.download", "download"),
    ("nav.support", "support"),
    ("nav.section.tools", "tools"),
    ("nav.section.more", "more"),
    ("instance.connected", "connected"),
    ("instance.turnstile", "captcha required"),
    ("instance.offline", "unreachable"),
    ("instance.connecting", "connecting…"),
    ("instance.official", "official instance"),
    ("theme.toggle", "toggle theme"),
    ("activity.open_folder", "show in folder"),
    ("activity.save", "save file"),
    ("settings.section.general", "general"),
    ("settings.section.media", "media"),
    ("settings.section.processing", "processing"),
    ("settings.section.app", "app"),
    ("updates.versions", "versions"),
    ("settings.desktop.ffmpeg.browse", "browse"),
    ("settings.processing.status", "instance status"),
    ("settings.processing.status.turnstile", "turnstile required"),
    ("settings.processing.status.no_captcha", "no captcha"),
    ("settings.processing.status.services", "services"),
];

const DESKTOP_EXTRA_RU: &[(&str, &str)] = &[
    ("settings.page.desktop", "десктоп"),
];

struct Table {
    map: HashMap<String, String>,
}

fn parse_into(map: &mut HashMap<String, String>, prefix: &str, json: &str) {
    let value: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return,
    };
    if let serde_json::Value::Object(obj) = value {
        for (k, v) in obj {
            if let serde_json::Value::String(s) = v {
                let key = if prefix.is_empty() { k } else { format!("{prefix}.{k}") };
                map.insert(key, s);
            }
        }
    }
}

fn build(files: &[(&str, &str)], extra: &[(&str, &str)]) -> Table {
    let mut map = HashMap::new();
    for (prefix, json) in files {
        parse_into(&mut map, prefix, json);
    }
    for (k, v) in extra {
        map.insert((*k).to_string(), (*v).to_string());
    }
    Table { map }
}

static TABLES: Lazy<HashMap<&'static str, Table>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("en", build(EN, DESKTOP_EXTRA_EN));
    m.insert("ru", build(RU, DESKTOP_EXTRA_RU));
    m
});

static CURRENT: Lazy<RwLock<String>> = Lazy::new(|| RwLock::new(DEFAULT_LOCALE.to_string()));

pub fn set_locale(locale: &str) {
    let locale = if TABLES.contains_key(locale) { locale } else { DEFAULT_LOCALE };
    *CURRENT.write() = locale.to_string();
}

pub fn current_locale() -> String {
    CURRENT.read().clone()
}

/// detects the system language (like the browser's `navigator.language`).
pub fn system_locale() -> String {
    let lang = std::env::var("LANG")
        .ok()
        .or_else(|| std::env::var("LC_ALL").ok())
        .unwrap_or_default();
    let mut code = lang.split(['_', '.', '-']).next().unwrap_or("").to_lowercase();
    if code.is_empty() {
        #[cfg(windows)]
        {
            code = windows_ui_language();
        }
    }
    if TABLES.contains_key(code.as_str()) {
        code
    } else {
        DEFAULT_LOCALE.to_string()
    }
}

#[cfg(windows)]
fn windows_ui_language() -> String {
    // PowerShell-free approach: read the user default UI language from the registry.
    let out = std::process::Command::new("reg")
        .args(["query", "HKCU\\Control Panel\\International", "/v", "LocaleName"])
        .output();
    if let Ok(out) = out {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            if line.contains("LocaleName") {
                if let Some(v) = line.split_whitespace().last() {
                    return v.split('-').next().unwrap_or("").to_lowercase();
                }
            }
        }
    }
    String::new()
}

fn lookup(key: &str) -> Option<String> {
    let cur = CURRENT.read();
    if let Some(t) = TABLES.get(cur.as_str()) {
        if let Some(v) = t.map.get(key) {
            return Some(v.clone());
        }
    }
    TABLES.get(DEFAULT_LOCALE).and_then(|t| t.map.get(key).cloned())
}

/// translate a key. unknown keys are returned as-is (same as the web app).
pub fn t(key: &str) -> String {
    lookup(key).unwrap_or_else(|| key.to_string())
}

/// translate with `{{ name }}` replacements.
pub fn tf(key: &str, params: &[(&str, &str)]) -> String {
    let mut s = t(key);
    for (name, value) in params {
        // the web app accepts both `{{ value }}` and `{{value}}`
        s = s.replace(&format!("{{{{ {name} }}}}"), value);
        s = s.replace(&format!("{{{{{name}}}}}"), value);
    }
    s
}

/// translate an api error code with its optional context
pub fn t_error(code: &str, service: Option<&str>, limit: Option<f64>) -> String {
    let limit_s = limit.map(|l| {
        if l.fract() == 0.0 { format!("{}", l as i64) } else { format!("{l}") }
    });
    let mut params: Vec<(&str, &str)> = Vec::new();
    if let Some(s) = service {
        params.push(("service", s));
    }
    if let Some(l) = &limit_s {
        params.push(("limit", l));
    }
    let text = tf(code, &params);
    if text == code {
        // unknown error code: fall back to the generic message and show the code
        format!("{}\n\n({code})", t("error.api.generic"))
    } else {
        text
    }
}

pub fn has_key(key: &str) -> bool {
    lookup(key).is_some()
}
