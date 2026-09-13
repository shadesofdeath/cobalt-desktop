//! application state, navigation, dialogs and the saving handler
//! (`web/src/lib/api/saving-handler.ts`, `web/src/lib/download.ts`, `web/src/lib/state/*`).

use crate::api::{ApiError, ApiResponse, CachedServerInfo, Client, PickerItem, SaveRequest, SharedClient};
use crate::i18n::{t, tf};
use crate::queue::{ItemState, QueueEvent, TaskManager};
use crate::settings::CobaltSettings;
use crate::theme::{Icons, Theme};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    Save,
    Remux,
    Settings,
    Donate,
    Updates,
    About,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsPage {
    Appearance,
    Accessibility,
    Video,
    Audio,
    Metadata,
    Local,
    Instances,
    Privacy,
    Desktop,
    Advanced,
    Debug,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AboutPage {
    General,
    Community,
    Privacy,
    Terms,
    Credits,
    Desktop,
}

/// `CobaltDownloadButtonState`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ButtonState {
    Idle,
    Think,
    Check,
    Done,
    Error,
}

impl ButtonState {
    pub fn text(&self) -> &'static str {
        match self {
            ButtonState::Idle => ">>",
            ButtonState::Think => "...",
            ButtonState::Check => "..?",
            ButtonState::Done => ">>>",
            ButtonState::Error => "!!",
        }
    }
}

pub enum AppEvent {
    SaveResponse {
        result: Result<ApiResponse, ApiError>,
        request: SaveRequest,
        old_task_id: Option<String>,
    },
    TunnelProbe {
        ok: bool,
        url: String,
        filename: String,
        request: SaveRequest,
    },
    ServerInfo(Result<CachedServerInfo, ApiError>),
    Thumb {
        url: String,
        bytes: Option<Arc<[u8]>>,
    },
    FileSaved {
        id: String,
        result: Result<PathBuf, String>,
    },
    FilesPicked(Vec<PathBuf>),
    SaveAsPicked {
        id: String,
        path: Option<PathBuf>,
    },
    ImportPicked(Option<PathBuf>),
    ExportPicked(Option<PathBuf>),
    FolderPicked(Option<PathBuf>),
    FfmpegPicked(Option<PathBuf>),
}

#[derive(Clone, Debug)]
pub enum DialogAction {
    Close,
    ResetSettings,
    OpenUrl(String),
    CopyText(String),
    DownloadUrl { url: String, filename: String },
    SaveFileToDownloads(String),
    SaveFileAs(String),
    ShowInFolder(PathBuf),
    ClearCache,
    ConfirmCustomInstance,
    ImportSettings(String),
}

#[derive(Clone, Debug)]
pub struct DialogButton {
    pub text: String,
    pub main: bool,
    pub red: bool,
    pub action: DialogAction,
}

impl DialogButton {
    pub fn new(text: impl Into<String>, action: DialogAction) -> Self {
        Self { text: text.into(), main: false, red: false, action }
    }
    pub fn main(mut self) -> Self {
        self.main = true;
        self
    }
    pub fn red(mut self) -> Self {
        self.red = true;
        self
    }
}

#[derive(Clone, Debug)]
pub enum DialogKind {
    Small {
        icon: Option<&'static str>,
        icon_color: Option<&'static str>,
        meowbalt: Option<&'static str>,
        title: String,
        body: String,
        body_sub: String,
        left_aligned: bool,
    },
    Picker {
        items: Vec<PickerItem>,
        audio: Option<String>,
    },
    Saving {
        url: Option<(String, String)>,
        file: Option<(String, PathBuf, String)>,
        body: String,
    },
}

#[derive(Clone, Debug)]
pub struct Dialog {
    pub id: String,
    pub kind: DialogKind,
    pub buttons: Vec<DialogButton>,
    pub dismissable: bool,
}

pub struct DonateState {
    pub recurring: bool,
    pub custom_amount: String,
    pub copied: Option<(String, Instant)>,
}

pub struct App {
    pub settings: CobaltSettings,
    pub client: SharedClient,
    pub tm: Arc<TaskManager>,
    pub icons: Arc<Icons>,
    pub theme: Theme,
    pub events_tx: crossbeam_channel::Sender<AppEvent>,
    pub events_rx: crossbeam_channel::Receiver<AppEvent>,
    pub queue_rx: crossbeam_channel::Receiver<QueueEvent>,

    pub page: Page,
    pub settings_page: SettingsPage,
    pub about_page: AboutPage,

    // save page
    pub link: String,
    pub button_state: ButtonState,
    pub button_state_since: Instant,
    pub focus_input: bool,
    pub services_expanded: bool,
    pub server_info: Option<CachedServerInfo>,
    pub server_info_loading: bool,
    pub server_info_error: Option<String>,

    // queue
    pub queue_visible: bool,
    pub saving_dialog_for_done: bool,

    pub dialogs: Vec<Dialog>,
    pub thumbs: HashMap<String, Option<Arc<[u8]>>>,
    pub updates_index: usize,
    pub donate: DonateState,
    pub remux_dragging: bool,
    pub copied_at: Option<Instant>,
    pub ffmpeg_path: Option<PathBuf>,
    pub last_settings: CobaltSettings,
    /// (dark, reduce_motion) the egui style was last configured for
    pub style_applied: Option<(bool, bool)>,
    pub screenshot: Option<crate::screenshot::Plan>,
    pub ctx: egui::Context,
    /// screenshot / test mode: never write settings to disk
    pub no_persist: bool,
    pub toasts: Vec<Toast>,
    pub activity_open: bool,
    /// queue items that were already re-requested automatically after a tunnel failure
    pub auto_retried: std::collections::HashSet<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ToastKind {
    Info,
    Success,
    Error,
}

#[derive(Clone, Debug)]
pub struct Toast {
    pub text: String,
    pub kind: ToastKind,
    pub at: Instant,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, screenshot: Option<crate::screenshot::Plan>) -> Self {
        crate::theme::install_fonts(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut settings = CobaltSettings::load();
        settings.validate();
        crate::i18n::set_locale(&settings.effective_locale());

        let client = Client::new();
        let (events_tx, events_rx) = crossbeam_channel::unbounded();
        let (queue_tx, queue_rx) = crossbeam_channel::unbounded();
        let tm = TaskManager::new(client.http().clone(), queue_tx);
        tm.set_repaint_context(cc.egui_ctx.clone());
        let ffmpeg_path = crate::ffmpeg::find_ffmpeg(&settings.desktop.ffmpeg_path);
        tm.set_ffmpeg(ffmpeg_path.clone());

        let system_dark = cc.egui_ctx.system_theme().map(|t| t == egui::Theme::Dark).unwrap_or(true);
        let theme = Theme::for_setting(&settings.appearance.theme, system_dark);
        let last_settings = settings.clone();

        let mut app = Self {
            settings,
            client,
            tm,
            icons: Arc::new(Icons::new()),
            theme,
            events_tx,
            events_rx,
            queue_rx,
            page: Page::Save,
            settings_page: SettingsPage::Appearance,
            about_page: AboutPage::General,
            link: String::new(),
            button_state: ButtonState::Idle,
            button_state_since: Instant::now(),
            focus_input: true,
            services_expanded: false,
            server_info: None,
            server_info_loading: false,
            server_info_error: None,
            queue_visible: false,
            saving_dialog_for_done: false,
            dialogs: Vec::new(),
            thumbs: HashMap::new(),
            updates_index: 0,
            donate: DonateState { recurring: false, custom_amount: String::new(), copied: None },
            remux_dragging: false,
            copied_at: None,
            ffmpeg_path,
            last_settings,
            style_applied: None,
            no_persist: screenshot.is_some(),
            screenshot,
            ctx: cc.egui_ctx.clone(),
            toasts: Vec::new(),
            activity_open: false,
            auto_retried: std::collections::HashSet::new(),
        };
        app.load_server_info();
        app
    }

    // --- helpers ---

    pub fn set_button_state(&mut self, state: ButtonState) {
        self.button_state = state;
        self.button_state_since = Instant::now();
    }

    pub fn tick_button_state(&mut self) {
        if matches!(self.button_state, ButtonState::Done | ButtonState::Error)
            && self.button_state_since.elapsed().as_millis() > 1500
        {
            self.button_state = ButtonState::Idle;
        }
    }

    pub fn open_dialog(&mut self, dialog: Dialog) {
        self.dialogs.retain(|d| d.id != dialog.id);
        self.dialogs.push(dialog);
    }

    pub fn close_dialog(&mut self) {
        self.dialogs.pop();
    }

    pub fn error_dialog(&mut self, text: String) {
        self.open_dialog(Dialog {
            id: "save-error".into(),
            kind: DialogKind::Small {
                icon: None,
                icon_color: None,
                meowbalt: Some("error"),
                title: String::new(),
                body: text,
                body_sub: String::new(),
                left_aligned: false,
            },
            buttons: vec![DialogButton::new(t("button.gotit"), DialogAction::Close).main()],
            dismissable: true,
        });
    }

    /// called after any ui interaction that may have changed settings.
    /// cheap when nothing changed; only does the expensive bits for what actually changed.
    pub fn settings_changed(&mut self) {
        if self.settings == self.last_settings {
            return;
        }
        self.settings.validate();
        let prev = std::mem::replace(&mut self.last_settings, self.settings.clone());
        if !self.no_persist {
            self.settings.save();
        }
        if prev.appearance.language != self.settings.appearance.language || prev.appearance.auto_language != self.settings.appearance.auto_language {
            crate::i18n::set_locale(&self.settings.effective_locale());
        }
        if prev.desktop.ffmpeg_path != self.settings.desktop.ffmpeg_path {
            let ffmpeg = crate::ffmpeg::find_ffmpeg(&self.settings.desktop.ffmpeg_path);
            self.ffmpeg_path = ffmpeg.clone();
            self.tm.set_ffmpeg(ffmpeg);
        }
        // api url may have changed: forget the cached instance info
        if prev.api_url() != self.settings.api_url() {
            self.client.clear_server_info();
            self.server_info = None;
            self.server_info_error = None;
            self.load_server_info();
        }
    }

    pub fn load_server_info(&mut self) {
        if self.server_info_loading {
            return;
        }
        self.server_info_loading = true;
        let client = self.client.clone();
        let api = self.settings.api_url();
        let tx = self.events_tx.clone();
        tokio::spawn(async move {
            let result = client.server_info(&api).await;
            let _ = tx.send(AppEvent::ServerInfo(result));
        });
    }

    pub fn copy_text(&mut self, text: &str) {
        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_text(text.to_string());
        }
        self.copied_at = Some(Instant::now());
        self.toast(t("toast.copied"), ToastKind::Success);
    }

    pub fn toast(&mut self, text: String, kind: ToastKind) {
        self.toasts.retain(|t| t.text != text);
        self.toasts.push(Toast { text, kind, at: Instant::now() });
    }

    pub fn tick_toasts(&mut self) {
        self.toasts.retain(|t| t.at.elapsed().as_secs_f32() < 3.2);
    }

    pub fn paste_from_clipboard(&mut self) {
        if !self.dialogs.is_empty() || self.is_loading() {
            return;
        }
        let text = arboard::Clipboard::new().ok().and_then(|mut cb| cb.get_text().ok());
        if let Some(text) = text {
            if let Some(link) = crate::api::extract_link(&text) {
                self.link = link.clone();
                self.saving_handler(Some(link), None, None);
            }
        }
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.button_state, ButtonState::Think | ButtonState::Check)
    }

    // --- saving handler (saving-handler.ts) ---

    pub fn saving_handler(&mut self, url: Option<String>, request: Option<SaveRequest>, old_task_id: Option<String>) {
        self.set_button_state(ButtonState::Think);
        let request = match request {
            Some(r) => r,
            None => match url {
                Some(u) => SaveRequest::from_settings(u.trim(), &self.settings),
                None => return,
            },
        };
        let client = self.client.clone();
        let settings = self.settings.clone();
        let tx = self.events_tx.clone();
        tokio::spawn(async move {
            let result = client.request(&settings, &request).await;
            let _ = tx.send(AppEvent::SaveResponse { result, request, old_task_id });
        });
    }

    fn handle_save_response(&mut self, result: Result<ApiResponse, ApiError>, request: SaveRequest, old_task_id: Option<String>) {
        if let Some(id) = &old_task_id {
            self.tm.set_retrying(id, false);
        }
        let response = match result {
            Ok(r) => r,
            Err(e) => {
                self.set_button_state(ButtonState::Error);
                self.error_dialog(e.message());
                return;
            }
        };
        match response {
            ApiResponse::Error { error } => {
                self.set_button_state(ButtonState::Error);
                self.error_dialog(error.message());
            }
            ApiResponse::Redirect { url, filename } => {
                self.set_button_state(ButtonState::Done);
                self.download_url(&url, &filename, Some(request));
            }
            ApiResponse::Tunnel { url, filename } => {
                self.set_button_state(ButtonState::Check);
                let client = self.client.clone();
                let tx = self.events_tx.clone();
                tokio::spawn(async move {
                    let ok = client.probe_tunnel(&url).await;
                    let _ = tx.send(AppEvent::TunnelProbe { ok, url, filename, request });
                });
            }
            ApiResponse::LocalProcessing(info) => {
                self.set_button_state(ButtonState::Done);
                if self.ffmpeg_path.is_none() && info.kind != "proxy" {
                    self.error_dialog(t("error.ffmpeg.not_found"));
                    return;
                }
                match self.tm.create_save_pipeline(&info, request, old_task_id) {
                    Ok(_) => self.open_queue_popover(),
                    Err(code) => self.error_dialog(t(&format!("error.{code}"))),
                }
            }
            ApiResponse::Picker { picker, audio, audio_filename: _ } => {
                self.set_button_state(ButtonState::Done);
                let mut buttons = vec![DialogButton::new(t("button.done"), DialogAction::Close).main()];
                if let Some(a) = &audio {
                    buttons.insert(
                        0,
                        DialogButton::new(
                            t("button.download.audio"),
                            DialogAction::DownloadUrl { url: a.clone(), filename: String::new() },
                        ),
                    );
                }
                for item in &picker {
                    if let Some(thumb) = &item.thumb {
                        self.load_thumb(thumb);
                    } else if item.kind == "photo" {
                        self.load_thumb(&item.url);
                    }
                }
                self.open_dialog(Dialog {
                    id: "download-picker".into(),
                    kind: DialogKind::Picker { items: picker, audio },
                    buttons,
                    dismissable: true,
                });
            }
        }
    }

    pub fn load_thumb(&mut self, url: &str) {
        if self.thumbs.contains_key(url) {
            return;
        }
        self.thumbs.insert(url.to_string(), None);
        let http = self.client.http().clone();
        let tx = self.events_tx.clone();
        let url = url.to_string();
        tokio::spawn(async move {
            let client = reqwest::Client::builder().redirect(reqwest::redirect::Policy::limited(5)).build().unwrap_or(http);
            let bytes = match client.get(&url).send().await {
                Ok(r) if r.status().is_success() => r.bytes().await.ok().map(|b| Arc::from(b.to_vec().into_boxed_slice())),
                _ => None,
            };
            let _ = tx.send(AppEvent::Thumb { url, bytes });
        });
    }

    pub fn open_queue_popover(&mut self) {
        if !self.settings.accessibility.dont_auto_open_queue {
            // on the download page the activity list is inline; elsewhere open the side panel
            self.queue_visible = true;
            if self.page != Page::Save {
                self.activity_open = true;
            }
        }
        self.toast(t("toast.queued"), ToastKind::Info);
    }

    /// `downloadFile({ url })` for tunnel / redirect urls
    pub fn download_url(&mut self, url: &str, filename: &str, request: Option<SaveRequest>) {
        let filename = if filename.is_empty() { filename_from_url(url) } else { filename.to_string() };
        match self.settings.save.saving_method.as_str() {
            "download" => {
                self.tm.create_proxy_pipeline(url, &filename, request);
                self.open_queue_popover();
            }
            "copy" => {
                self.copy_text(url);
            }
            _ => {
                self.open_saving_dialog(Some((url.to_string(), filename)), None, String::new());
            }
        }
    }

    pub fn open_saving_dialog(&mut self, url: Option<(String, String)>, file: Option<(String, PathBuf, String)>, body: String) {
        let mut buttons = Vec::new();
        if let Some((id, _, _)) = &file {
            buttons.push(DialogButton::new(t("button.download"), DialogAction::SaveFileToDownloads(id.clone())).main());
            buttons.push(DialogButton::new(format!("{}...", t("button.save")), DialogAction::SaveFileAs(id.clone())));
        }
        if let Some((u, name)) = &url {
            buttons.push(
                DialogButton::new(t("button.download"), DialogAction::DownloadUrl { url: u.clone(), filename: name.clone() }).main(),
            );
            buttons.push(DialogButton::new(t("dialog.saving.open_browser"), DialogAction::OpenUrl(u.clone())));
            buttons.push(DialogButton::new(t("button.copy"), DialogAction::CopyText(u.clone())));
        }
        buttons.push(DialogButton::new(t("button.cancel"), DialogAction::Close));
        self.open_dialog(Dialog {
            id: "saving".into(),
            kind: DialogKind::Saving { url, file, body },
            buttons,
            dismissable: true,
        });
    }

    pub fn save_to_downloads(&mut self, id: &str) {
        let Some(item) = self.tm.state.lock().get(id).cloned() else { return };
        let ItemState::Done { file, .. } = &item.state else { return };
        let dir = self.settings.download_dir();
        let target = unique_path(&dir.join(sanitize_filename(&item.filename)));
        self.move_file(id.to_string(), file.clone(), target);
    }

    pub fn save_as(&mut self, id: &str) {
        let Some(item) = self.tm.state.lock().get(id).cloned() else { return };
        let tx = self.events_tx.clone();
        let id = id.to_string();
        let dir = self.settings.download_dir();
        let name = sanitize_filename(&item.filename);
        std::thread::spawn(move || {
            let path = rfd::FileDialog::new().set_directory(dir).set_file_name(name).save_file();
            let _ = tx.send(AppEvent::SaveAsPicked { id, path });
        });
    }

    fn move_file(&mut self, id: String, from: PathBuf, to: PathBuf) {
        let tx = self.events_tx.clone();
        std::thread::spawn(move || {
            let result = (|| -> Result<PathBuf, String> {
                if let Some(parent) = to.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                // keep the cache copy so the queue can save it again
                std::fs::copy(&from, &to).map_err(|e| e.to_string())?;
                Ok(to)
            })();
            let _ = tx.send(AppEvent::FileSaved { id, result });
        });
    }

    pub fn pick_remux_files(&mut self) {
        let tx = self.events_tx.clone();
        std::thread::spawn(move || {
            let files = rfd::FileDialog::new()
                .add_filter("media", &["mp4", "webm", "mkv", "mov", "mp3", "ogg", "opus", "wav", "m4a", "flac"])
                .pick_files()
                .unwrap_or_default();
            let _ = tx.send(AppEvent::FilesPicked(files));
        });
    }

    pub fn remux_files(&mut self, files: Vec<PathBuf>) {
        if files.is_empty() {
            return;
        }
        if self.ffmpeg_path.is_none() {
            self.error_dialog(t("error.ffmpeg.not_found"));
            return;
        }
        let mut added = false;
        for f in files {
            if self.tm.create_remux_pipeline(&f).is_some() {
                added = true;
            }
        }
        if added {
            self.open_queue_popover();
        }
    }

    pub fn export_settings(&mut self) {
        let tx = self.events_tx.clone();
        std::thread::spawn(move || {
            let path = rfd::FileDialog::new().set_file_name("cobalt-settings.json").add_filter("json", &["json"]).save_file();
            let _ = tx.send(AppEvent::ExportPicked(path));
        });
    }

    pub fn import_settings(&mut self) {
        let tx = self.events_tx.clone();
        std::thread::spawn(move || {
            let path = rfd::FileDialog::new().add_filter("json", &["json"]).pick_file();
            let _ = tx.send(AppEvent::ImportPicked(path));
        });
    }

    pub fn pick_download_folder(&mut self) {
        let tx = self.events_tx.clone();
        let dir = self.settings.download_dir();
        std::thread::spawn(move || {
            let path = rfd::FileDialog::new().set_directory(dir).pick_folder();
            let _ = tx.send(AppEvent::FolderPicked(path));
        });
    }

    pub fn pick_ffmpeg(&mut self) {
        let tx = self.events_tx.clone();
        std::thread::spawn(move || {
            let path = rfd::FileDialog::new().add_filter("ffmpeg", &["exe", ""]).pick_file();
            let _ = tx.send(AppEvent::FfmpegPicked(path));
        });
    }

    pub fn reset_settings_dialog(&mut self) {
        self.open_dialog(Dialog {
            id: "reset-settings".into(),
            kind: DialogKind::Small {
                icon: Some("alert-triangle"),
                icon_color: Some("red"),
                meowbalt: None,
                title: t("dialog.reset_settings.title"),
                body: t("dialog.reset_settings.body"),
                body_sub: String::new(),
                left_aligned: false,
            },
            buttons: vec![
                DialogButton::new(t("button.cancel"), DialogAction::Close),
                DialogButton::new(t("button.reset"), DialogAction::ResetSettings).main().red(),
            ],
            dismissable: true,
        });
    }

    pub fn clear_cache_dialog(&mut self) {
        self.open_dialog(Dialog {
            id: "clear-cache".into(),
            kind: DialogKind::Small {
                icon: Some("alert-triangle"),
                icon_color: Some("red"),
                meowbalt: None,
                title: t("dialog.clear_cache.title"),
                body: t("dialog.clear_cache.body"),
                body_sub: String::new(),
                left_aligned: false,
            },
            buttons: vec![
                DialogButton::new(t("button.cancel"), DialogAction::Close),
                DialogButton::new(t("button.clear_cache"), DialogAction::ClearCache).main().red(),
            ],
            dismissable: true,
        });
    }

    pub fn custom_instance_warning(&mut self) {
        self.open_dialog(Dialog {
            id: "safety-warning".into(),
            kind: DialogKind::Small {
                icon: Some("alert-triangle"),
                icon_color: Some("red"),
                meowbalt: None,
                title: t("dialog.safety.title"),
                body: t("dialog.safety.custom_instance.body"),
                body_sub: String::new(),
                left_aligned: true,
            },
            buttons: vec![
                DialogButton::new(t("button.cancel"), DialogAction::Close),
                DialogButton::new(t("button.continue"), DialogAction::ConfirmCustomInstance).main(),
            ],
            dismissable: false,
        });
    }

    pub fn retry_item(&mut self, id: &str) {
        let req = self.tm.state.lock().get(id).and_then(|i| i.original_request.clone());
        if let Some(req) = req {
            self.tm.set_retrying(id, true);
            self.saving_handler(None, Some(req), Some(id.to_string()));
        }
    }

    pub fn run_action(&mut self, action: DialogAction) {
        match action {
            DialogAction::Close => self.close_dialog(),
            DialogAction::ResetSettings => {
                self.close_dialog();
                let desktop = self.settings.desktop.clone();
                self.settings = CobaltSettings::default();
                self.settings.desktop = desktop;
                self.settings_changed();
            }
            DialogAction::OpenUrl(url) => {
                self.close_dialog();
                let _ = open::that_detached(url);
            }
            DialogAction::CopyText(text) => {
                self.close_dialog();
                self.copy_text(&text);
            }
            DialogAction::DownloadUrl { url, filename } => {
                self.close_dialog();
                let filename = if filename.is_empty() { filename_from_url(&url) } else { filename };
                self.tm.create_proxy_pipeline(&url, &filename, None);
                self.open_queue_popover();
            }
            DialogAction::SaveFileToDownloads(id) => {
                self.close_dialog();
                self.save_to_downloads(&id);
            }
            DialogAction::SaveFileAs(id) => {
                self.close_dialog();
                self.save_as(&id);
            }
            DialogAction::ShowInFolder(path) => {
                self.close_dialog();
                show_in_folder(&path);
            }
            DialogAction::ClearCache => {
                self.close_dialog();
                self.tm.clear_queue();
            }
            DialogAction::ConfirmCustomInstance => {
                self.close_dialog();
                self.settings.processing.seen_custom_warning = true;
                self.settings_changed();
            }
            DialogAction::ImportSettings(json) => {
                self.close_dialog();
                match CobaltSettings::from_json(&json) {
                    Ok(mut s) => {
                        if s.desktop.download_dir.is_empty() {
                            s.desktop = self.settings.desktop.clone();
                        }
                        self.settings = s;
                        self.settings_changed();
                    }
                    Err(e) => self.error_dialog(tf("error.import.unknown", &[("value", &e.to_string())])),
                }
            }
        }
    }

    // --- event pump, called every frame ---

    pub fn pump_events(&mut self) {
        while let Ok(ev) = self.events_rx.try_recv() {
            match ev {
                AppEvent::SaveResponse { result, request, old_task_id } => {
                    self.handle_save_response(result, request, old_task_id)
                }
                AppEvent::TunnelProbe { ok, url, filename, request } => {
                    if ok {
                        self.set_button_state(ButtonState::Done);
                        self.download_url(&url, &filename, Some(request));
                    } else {
                        self.set_button_state(ButtonState::Error);
                        self.error_dialog(t("error.tunnel.probe"));
                    }
                }
                AppEvent::ServerInfo(result) => {
                    self.server_info_loading = false;
                    match result {
                        Ok(info) => {
                            self.server_info = Some(info);
                            self.server_info_error = None;
                        }
                        Err(e) => self.server_info_error = Some(e.message()),
                    }
                }
                AppEvent::Thumb { url, bytes } => {
                    self.thumbs.insert(url, bytes);
                }
                AppEvent::FileSaved { id, result } => match result {
                    Ok(path) => {
                        let folder = path.parent().map(|p| p.display().to_string()).unwrap_or_default();
                        self.tm.mark_saved(&id, path);
                        self.toast(tf("toast.saved", &[("value", &folder)]), ToastKind::Success);
                    }
                    Err(e) => self.error_dialog(tf("error.save.failed", &[("value", &e)])),
                },
                AppEvent::FilesPicked(files) => self.remux_files(files),
                AppEvent::SaveAsPicked { id, path } => {
                    if let Some(path) = path {
                        let file = self.tm.state.lock().get(&id).and_then(|i| match &i.state {
                            ItemState::Done { file, .. } => Some(file.clone()),
                            _ => None,
                        });
                        if let Some(file) = file {
                            self.move_file(id, file, path);
                        }
                    }
                }
                AppEvent::ImportPicked(path) => {
                    if let Some(path) = path {
                        match std::fs::read_to_string(&path) {
                            Ok(json) => {
                                let valid = serde_json::from_str::<serde_json::Value>(&json)
                                    .map(|v| v.get("schemaVersion").is_some())
                                    .unwrap_or(false);
                                if !valid {
                                    self.error_dialog(t("error.import.invalid"));
                                } else {
                                    self.open_dialog(Dialog {
                                        id: "import-settings".into(),
                                        kind: DialogKind::Small {
                                            icon: Some("alert-triangle"),
                                            icon_color: Some("red"),
                                            meowbalt: None,
                                            title: t("dialog.safety.title"),
                                            body: t("dialog.import.body"),
                                            body_sub: String::new(),
                                            left_aligned: true,
                                        },
                                        buttons: vec![
                                            DialogButton::new(t("button.cancel"), DialogAction::Close),
                                            DialogButton::new(t("button.import"), DialogAction::ImportSettings(json)).main(),
                                        ],
                                        dismissable: true,
                                    });
                                }
                            }
                            Err(e) => self.error_dialog(tf("error.import.unknown", &[("value", &e.to_string())])),
                        }
                    }
                }
                AppEvent::ExportPicked(path) => {
                    if let Some(path) = path {
                        let _ = std::fs::write(path, self.settings.to_json());
                    }
                }
                AppEvent::FolderPicked(path) => {
                    if let Some(path) = path {
                        self.settings.desktop.download_dir = path.to_string_lossy().to_string();
                        self.settings_changed();
                    }
                }
                AppEvent::FfmpegPicked(path) => {
                    if let Some(path) = path {
                        self.settings.desktop.ffmpeg_path = path.to_string_lossy().to_string();
                        self.settings_changed();
                    }
                }
            }
        }
        while let Ok(ev) = self.queue_rx.try_recv() {
            match ev {
                QueueEvent::ItemDone(id) => {
                    let item = self.tm.state.lock().get(&id).cloned();
                    if let Some(item) = item {
                        if let ItemState::Done { file, .. } = &item.state {
                            match self.settings.save.saving_method.as_str() {
                                "download" => self.save_to_downloads(&id),
                                _ => self.open_saving_dialog(
                                    None,
                                    Some((id.clone(), file.clone(), item.filename.clone())),
                                    String::new(),
                                ),
                            }
                        }
                    }
                }
                QueueEvent::ItemError(id, code) => {
                    // tunnels expire ~90s after the api response; when an item had to wait in the
                    // queue its tunnels may be stale. re-request once automatically before giving up.
                    let transient = matches!(
                        code.as_str(),
                        "queue.fetch.bad_response" | "queue.fetch.empty_tunnel" | "queue.fetch.network_error"
                    );
                    let can_retry = self.tm.state.lock().get(&id).map(|i| i.can_retry && i.original_request.is_some()).unwrap_or(false);
                    if transient && can_retry && !self.auto_retried.contains(&id) {
                        self.auto_retried.insert(id.clone());
                        self.retry_item(&id);
                    }
                }
            }
        }
    }
}

pub fn filename_from_url(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.path_segments().and_then(|s| s.last().map(|x| x.to_string())))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "file".to_string())
}

pub fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || (c as u32) < 32 { '_' } else { c })
        .collect();
    let trimmed = cleaned.trim().trim_end_matches('.').to_string();
    if trimmed.is_empty() { "file".into() } else { trimmed }
}

pub fn unique_path(path: &PathBuf) -> PathBuf {
    if !path.exists() {
        return path.clone();
    }
    let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "file".into());
    let ext = path.extension().map(|e| e.to_string_lossy().to_string());
    let dir = path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
    for i in 1..1000 {
        let name = match &ext {
            Some(e) => format!("{stem} ({i}).{e}"),
            None => format!("{stem} ({i})"),
        };
        let candidate = dir.join(name);
        if !candidate.exists() {
            return candidate;
        }
    }
    path.clone()
}

pub fn show_in_folder(path: &std::path::Path) {
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("explorer").arg(format!("/select,{}", path.display())).spawn();
    }
    #[cfg(not(windows))]
    {
        if let Some(parent) = path.parent() {
            let _ = open::that_detached(parent);
        }
    }
}
