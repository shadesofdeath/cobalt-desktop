//! design system: our own palette, fonts (Inter + IBM Plex Mono) and tabler icons.

use egui::{Color32, FontData, FontDefinitions, FontFamily, FontId};
use std::collections::HashMap;
use std::sync::Arc;

pub const SIDEBAR_WIDTH: f32 = 232.0;
pub const TOPBAR_HEIGHT: f32 = 60.0;
pub const PAGE_PADDING: f32 = 28.0;
pub const CARD_RADIUS: f32 = 16.0;
pub const CONTROL_RADIUS: f32 = 10.0;
pub const ACTIVITY_WIDTH: f32 = 400.0;

fn hex(s: &str) -> Color32 {
    let s = s.trim_start_matches('#');
    let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
    Color32::from_rgb(r, g, b)
}

fn rgba(r: u8, g: u8, b: u8, a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(r, g, b, (a * 255.0).round() as u8)
}

#[derive(Clone, Debug)]
pub struct Theme {
    pub dark: bool,
    /// window background
    pub bg: Color32,
    /// sidebar / rails
    pub bg_elevated: Color32,
    /// cards
    pub surface: Color32,
    pub surface_hover: Color32,
    pub surface_active: Color32,
    /// inset controls (inputs, segmented track)
    pub inset: Color32,
    pub border: Color32,
    pub border_strong: Color32,
    pub text: Color32,
    pub text_muted: Color32,
    pub text_faint: Color32,
    pub accent: Color32,
    pub accent_2: Color32,
    pub accent_soft: Color32,
    pub accent_text: Color32,
    pub success: Color32,
    pub success_soft: Color32,
    pub warning: Color32,
    pub warning_soft: Color32,
    pub danger: Color32,
    pub danger_soft: Color32,
    pub backdrop: Color32,
    pub shadow: Color32,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            dark: true,
            bg: hex("#0B0E14"),
            bg_elevated: hex("#0E121A"),
            surface: hex("#141924"),
            surface_hover: hex("#1A2030"),
            surface_active: hex("#212A3D"),
            inset: hex("#0F131C"),
            border: rgba(255, 255, 255, 0.07),
            border_strong: rgba(255, 255, 255, 0.14),
            text: hex("#EEF1F7"),
            text_muted: hex("#98A2B8"),
            text_faint: hex("#5F6B82"),
            accent: hex("#7C8CFF"),
            accent_2: hex("#22D3EE"),
            accent_soft: rgba(124, 140, 255, 0.16),
            accent_text: Color32::WHITE,
            success: hex("#34D399"),
            success_soft: rgba(52, 211, 153, 0.14),
            warning: hex("#FBBF24"),
            warning_soft: rgba(251, 191, 36, 0.14),
            danger: hex("#F87171"),
            danger_soft: rgba(248, 113, 113, 0.14),
            backdrop: rgba(4, 6, 12, 0.6),
            shadow: rgba(0, 0, 0, 0.55),
        }
    }

    pub fn light() -> Self {
        Self {
            dark: false,
            bg: hex("#F3F5FA"),
            bg_elevated: hex("#FFFFFF"),
            surface: hex("#FFFFFF"),
            surface_hover: hex("#F3F5FA"),
            surface_active: hex("#E9EDF6"),
            inset: hex("#F1F3F8"),
            border: rgba(15, 23, 42, 0.08),
            border_strong: rgba(15, 23, 42, 0.16),
            text: hex("#0F172A"),
            text_muted: hex("#5B6478"),
            text_faint: hex("#94A0B8"),
            accent: hex("#5B6CFF"),
            accent_2: hex("#06B6D4"),
            accent_soft: rgba(91, 108, 255, 0.12),
            accent_text: Color32::WHITE,
            success: hex("#059669"),
            success_soft: rgba(5, 150, 105, 0.12),
            warning: hex("#D97706"),
            warning_soft: rgba(217, 119, 6, 0.12),
            danger: hex("#DC2626"),
            danger_soft: rgba(220, 38, 38, 0.12),
            backdrop: rgba(15, 23, 42, 0.35),
            shadow: rgba(15, 23, 42, 0.18),
        }
    }

    pub fn for_setting(theme: &str, system_dark: bool) -> Self {
        match theme {
            "light" => Self::light(),
            "dark" => Self::dark(),
            _ => {
                if system_dark {
                    Self::dark()
                } else {
                    Self::light()
                }
            }
        }
    }
}

// --- fonts ---

pub const FONT_REGULAR: &str = "inter";
pub const FONT_MEDIUM: &str = "inter-medium";
pub const FONT_SEMIBOLD: &str = "inter-semibold";
pub const FONT_BOLD: &str = "inter-bold";
pub const FONT_MONO: &str = "plex-mono";
pub const FONT_MONO_MEDIUM: &str = "plex-mono-medium";

pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let faces: [(&str, &'static [u8]); 6] = [
        (FONT_REGULAR, include_bytes!("../assets/fonts/Inter-Regular.ttf")),
        (FONT_MEDIUM, include_bytes!("../assets/fonts/Inter-Medium.ttf")),
        (FONT_SEMIBOLD, include_bytes!("../assets/fonts/Inter-SemiBold.ttf")),
        (FONT_BOLD, include_bytes!("../assets/fonts/Inter-Bold.ttf")),
        (FONT_MONO, include_bytes!("../assets/fonts/IBMPlexMono-Regular.ttf")),
        (FONT_MONO_MEDIUM, include_bytes!("../assets/fonts/IBMPlexMono-Medium.ttf")),
    ];
    for (name, bytes) in faces {
        fonts.font_data.insert(name.into(), Arc::new(FontData::from_static(bytes)));
    }
    let fallbacks: Vec<String> = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();

    let with_fallbacks = |primary: &str| {
        let mut v = vec![primary.to_owned()];
        v.extend(fallbacks.iter().cloned());
        v
    };
    fonts.families.insert(FontFamily::Proportional, with_fallbacks(FONT_REGULAR));
    fonts.families.insert(FontFamily::Monospace, with_fallbacks(FONT_MONO));
    for name in [FONT_MEDIUM, FONT_SEMIBOLD, FONT_BOLD, FONT_MONO_MEDIUM] {
        fonts.families.insert(FontFamily::Name(name.into()), with_fallbacks(name));
    }
    ctx.set_fonts(fonts);
}

pub fn regular(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}
pub fn medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(FONT_MEDIUM.into()))
}
pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(FONT_SEMIBOLD.into()))
}
pub fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(FONT_BOLD.into()))
}
pub fn mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}
pub fn mono_medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(FONT_MONO_MEDIUM.into()))
}

/// sentence-case: cobalt's strings are all lowercase, our ui capitalises the first letter
pub fn cap(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

// --- icons ---

macro_rules! icons {
    ($($name:literal),* $(,)?) => {
        &[$(($name, include_str!(concat!("../assets/icons/", $name, ".svg")))),*]
    };
}

const ICON_SOURCES: &[(&str, &str)] = icons![
    "accessible", "adjustments-star", "alert-triangle", "arrow-back", "arrow-down", "arrow-left",
    "arrow-right", "box-multiple", "brand-bluesky", "brand-discord", "brand-github", "brand-telegram",
    "brand-x", "bug", "calendar-repeat", "check", "checklist", "chevron-right", "coin", "comet", "copy",
    "cpu", "cup", "device-laptop", "devices", "diamond", "download", "exclamation-circle",
    "external-link", "eye", "eye-off", "file", "file-download", "file-export", "file-import",
    "file-shredder", "folder", "gif", "heart", "heart-handshake", "info-circle", "link", "loader-2",
    "lock", "mood-smile-beam", "movie", "music", "photo", "plus", "reload", "repeat", "restore",
    "selector", "settings", "share-2", "sun-high", "upload", "users-group", "world", "world-www", "x",
    "sparkles", "clipboard", "volume-off", "volume", "trash", "player-play", "folder-open", "moon", "moon-stars", "bolt",
];

pub struct Icons {
    data: HashMap<String, Arc<[u8]>>,
}

impl Icons {
    pub fn new() -> Self {
        let mut data: HashMap<String, Arc<[u8]>> = HashMap::new();
        for (name, svg) in ICON_SOURCES {
            for (variant, width) in [("", "1.75"), ("thin", "1.5"), ("bold", "2.25")] {
                let processed = svg
                    .replace("currentColor", "#ffffff")
                    .replace("stroke-width=\"2\"", &format!("stroke-width=\"{width}\""));
                let key = if variant.is_empty() { name.to_string() } else { format!("{name}@{variant}") };
                data.insert(key, Arc::from(processed.into_bytes().into_boxed_slice()));
            }
        }
        Self { data }
    }

    pub fn source(&self, name: &str) -> egui::ImageSource<'static> {
        let bytes = self.data.get(name).or_else(|| self.data.get("x")).cloned().expect("icon");
        egui::ImageSource::Bytes {
            uri: std::borrow::Cow::Owned(format!("bytes://icons/{name}.svg")),
            bytes: egui::load::Bytes::Shared(bytes),
        }
    }

    pub fn image(&self, name: &str, size: f32, color: Color32) -> egui::Image<'static> {
        egui::Image::new(self.source(name)).fit_to_exact_size(egui::vec2(size, size)).tint(color)
    }
}
