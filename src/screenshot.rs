//! `--screenshot <dir>`: renders a set of scenes and saves them as png files.

use crate::app::{AboutPage, App, ButtonState, Dialog, DialogButton, DialogAction, DialogKind, Page, SettingsPage};
use crate::queue::{ItemState, MediaType, PipelineItem, QueueItem, WorkerArgs, WorkerKind, WorkerProgress};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct Scene {
    pub name: &'static str,
    pub setup: fn(&mut App),
    /// seconds to wait after setup before capturing (animations, network)
    pub wait: f32,
}

impl Scene {
    fn new(name: &'static str, setup: fn(&mut App)) -> Self {
        Self { name, setup, wait: 0.7 }
    }
    fn waiting(name: &'static str, wait: f32, setup: fn(&mut App)) -> Self {
        Self { name, setup, wait }
    }
}

pub struct Plan {
    pub out_dir: PathBuf,
    pub scenes: Vec<Scene>,
    pub index: usize,
    pub frames: u32,
    pub requested: bool,
    pub started: Option<std::time::Instant>,
}

const TEST_LINK: &str = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";

fn fake_queue(app: &mut App) {
    app.tm.clear_queue();
    let mk = |id: &str, filename: &str, mime: &str, media: MediaType, state: ItemState, workers: Vec<(WorkerKind, Option<WorkerProgress>, bool)>| {
        let mut pipeline = Vec::new();
        let mut current = HashMap::new();
        let mut results = HashMap::new();
        for (i, (kind, progress, done)) in workers.into_iter().enumerate() {
            let wid = format!("{id}-w{i}");
            pipeline.push(PipelineItem {
                worker: kind,
                worker_id: wid.clone(),
                parent_id: id.to_string(),
                depends_on: vec![],
                args: match kind {
                    WorkerKind::Fetch => WorkerArgs::Fetch { url: String::new(), header_filename: false },
                    _ => WorkerArgs::Ffmpeg { ffargs: vec![], format: "mp4".into(), mime: mime.into(), files: vec![] },
                },
            });
            if done {
                results.insert(wid, (PathBuf::from("x"), 12_400_000));
            } else {
                current.insert(wid, progress);
            }
        }
        QueueItem {
            id: id.into(),
            pipeline,
            can_retry: true,
            original_request: None,
            filename: filename.into(),
            mime: mime.into(),
            media_type: media,
            state,
            pipeline_results: results,
            current_tasks: current,
            retrying: false,
        }
    };
    app.tm.add_item(mk(
        "a",
        "Rick Astley - Never Gonna Give You Up (Official Video) (4K Remaster) - Rick Astley (1080p, h264).mp4",
        "video/mp4",
        MediaType::Video,
        ItemState::Running,
        vec![
            (WorkerKind::Fetch, None, true),
            (WorkerKind::Fetch, Some(WorkerProgress { percentage: 62.0, size: 41_300_000 }), false),
            (WorkerKind::Remux, None, false),
        ],
    ));
    app.tm.add_item(mk(
        "b",
        "Daft Punk - Harder, Better, Faster, Stronger - Daft Punk.mp3",
        "audio/mpeg",
        MediaType::Audio,
        ItemState::Done { file: PathBuf::from("x"), size: 5_400_000, saved_to: Some(PathBuf::from("C:\\Users\\shades\\Downloads\\x.mp3")) },
        vec![(WorkerKind::Fetch, None, true), (WorkerKind::Encode, None, true)],
    ));
    app.tm.add_item(mk(
        "c",
        "twitter_1234567890.gif",
        "image/gif",
        MediaType::Image,
        ItemState::Error { code: "queue.fetch.empty_tunnel".into() },
        vec![(WorkerKind::Fetch, None, false), (WorkerKind::Encode, None, false)],
    ));
    app.tm.add_item(mk(
        "d",
        "instagram_reel.mp4",
        "video/mp4",
        MediaType::Video,
        ItemState::Waiting,
        vec![(WorkerKind::Fetch, None, false)],
    ));
    // stop the scheduler from touching fakes
    for item in app.tm.state.lock().items.iter_mut() {
        if matches!(item.state, ItemState::Waiting) {
            item.pipeline.clear();
        }
    }
}

pub fn default_plan(out_dir: PathBuf, live: bool) -> Plan {
    let mut scenes: Vec<Scene> = vec![
        Scene::waiting("01-save", 2.5, |app| {
            app.settings.appearance.theme = "dark".into();
            app.page = Page::Save;
            app.link.clear();
            app.dialogs.clear();
            app.queue_visible = false;
            app.services_expanded = false;
        }),
        Scene::new("02-save-services", |app| {
            app.page = Page::Save;
            app.services_expanded = true;
        }),
        Scene::new("03-save-link", |app| {
            app.services_expanded = false;
            app.link = TEST_LINK.into();
            app.settings.save.download_mode = "audio".into();
            app.set_button_state(ButtonState::Idle);
        }),
        Scene::new("04-save-light", |app| {
            app.settings.appearance.theme = "light".into();
            app.link.clear();
            app.settings.save.download_mode = "auto".into();
        }),
        Scene::new("05-settings-appearance", |app| {
            app.settings.appearance.theme = "dark".into();
            app.page = Page::Settings;
            app.settings_page = SettingsPage::Appearance;
        }),
        Scene::new("06-settings-video", |app| app.settings_page = SettingsPage::Video),
        Scene::new("07-settings-audio", |app| app.settings_page = SettingsPage::Audio),
        Scene::new("07b-settings-audio-dropdown", |app| {
            egui::Popup::open_id(&app.ctx, crate::ui::widgets::dropdown_popup_id("dub"));
        }),
        Scene::new("08-settings-metadata", |app| {
            egui::Popup::close_id(&app.ctx, crate::ui::widgets::dropdown_popup_id("dub"));
            app.settings_page = SettingsPage::Metadata;
        }),
        Scene::waiting("09-settings-instances", 2.0, |app| {
            app.settings_page = SettingsPage::Instances;
            app.settings.processing.enable_custom_instances = true;
            app.settings.processing.seen_custom_warning = true;
            if app.settings.processing.custom_instance_url.is_empty() {
                app.settings.processing.custom_instance_url = "http://localhost:9000".into();
            }
        }),
        Scene::new("10-settings-local", |app| app.settings_page = SettingsPage::Local),
        Scene::new("11-settings-desktop", |app| app.settings_page = SettingsPage::Desktop),
        Scene::new("12-remux", |app| app.page = Page::Remux),
        Scene::new("13-updates", |app| {
            app.page = Page::Updates;
            app.updates_index = 0;
        }),
        Scene::new("14-about", |app| {
            app.page = Page::About;
            app.about_page = AboutPage::General;
        }),
        Scene::new("15-about-community", |app| app.about_page = AboutPage::Community),
        Scene::new("16-donate", |app| app.page = Page::Donate),
        Scene::new("17-queue", |app| {
            app.page = Page::Save;
            fake_queue(app);
            app.queue_visible = true; app.activity_open = false;
        }),
        Scene::new("17b-activity-panel", |app| {
            app.page = Page::Settings;
            app.settings_page = SettingsPage::Video;
            app.activity_open = true;
        }),
        Scene::new("18-queue-light", |app| {
            app.settings.appearance.theme = "light".into();
            app.page = Page::Save;
            app.activity_open = false;
        }),
        Scene::new("19-dialog-error", |app| {
            app.settings.appearance.theme = "dark".into();
            app.queue_visible = false; app.activity_open = false;
            app.error_dialog(crate::i18n::t("error.api.content.video.live"));
        }),
        Scene::new("20-picker", |app| {
            app.dialogs.clear();
            let items: Vec<crate::api::PickerItem> = (0..6)
                .map(|i| crate::api::PickerItem {
                    kind: if i == 1 { "video".into() } else { "photo".into() },
                    url: format!("https://example.invalid/{i}.jpg"),
                    thumb: None,
                })
                .collect();
            for it in &items {
                app.thumbs.insert(it.url.clone(), None);
            }
            let dialog = Dialog {
                id: "download-picker".into(),
                kind: DialogKind::Picker { items, audio: Some("x".into()) },
                buttons: vec![
                    DialogButton::new(crate::i18n::t("button.download.audio"), DialogAction::Close),
                    DialogButton::new(crate::i18n::t("button.done"), DialogAction::Close).main(),
                ],
                dismissable: true,
            };
            app.open_dialog(dialog);
        }),
        Scene::new("21-reset-dialog", |app| {
            app.dialogs.clear();
            app.page = Page::Settings;
            app.settings_page = SettingsPage::Advanced;
            app.reset_settings_dialog();
        }),
    ];
    if live {
        // a real end-to-end run against the local instance: request -> tunnels -> ffmpeg -> saved file
        scenes.push(Scene::waiting("22-live-request", 1.5, |app| {
            app.dialogs.clear();
            app.tm.clear_queue();
            app.page = Page::Save;
            app.settings.processing.enable_custom_instances = true;
            app.settings.processing.seen_custom_warning = true;
            app.settings.processing.custom_instance_url = "http://localhost:9000".into();
            app.settings.save.download_mode = "auto".into();
            app.settings.save.video_quality = "720".into();
            app.settings.desktop.download_dir = std::env::temp_dir().join("cobalt-desktop-test").to_string_lossy().to_string();
            app.settings_changed();
            app.link = TEST_LINK.into();
            app.saving_handler(Some(TEST_LINK.into()), None, None);
        }));
        scenes.push(Scene::waiting("23-live-downloading", 6.0, |app| {
            app.queue_visible = true; app.activity_open = false;
        }));
        scenes.push(Scene::waiting("24-live-done", 60.0, |app| {
            app.queue_visible = true; app.activity_open = false;
        }));
    }
    Plan { out_dir, scenes, index: 0, frames: 0, requested: false, started: None }
}

pub fn save_png(image: &egui::ColorImage, path: &std::path::Path) -> anyhow::Result<()> {
    let [w, h] = image.size;
    let mut buf: Vec<u8> = Vec::with_capacity(w * h * 4);
    for px in &image.pixels {
        buf.extend_from_slice(&px.to_array());
    }
    let img = image::RgbaImage::from_raw(w as u32, h as u32, buf).ok_or_else(|| anyhow::anyhow!("bad image"))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    img.save(path)?;
    Ok(())
}
