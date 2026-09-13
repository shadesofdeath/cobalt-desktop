//! settings: category rail on the left, cards with rows on the right

use crate::app::{App, SettingsPage};
use crate::i18n::{t, tf};
use crate::settings::{self as cfg, CobaltSettings};
use crate::theme::{self, cap, PAGE_PADDING};
use crate::ui::widgets::{self, cr, ButtonKind, ButtonOpts, Ctx, InputOpts, SegOption};
use egui::{Align, Align2, Color32, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};

struct Cat {
    page: SettingsPage,
    icon: &'static str,
    label: String,
    color: Color32,
}

pub fn show(app: &mut App, c: &Ctx, ui: &mut Ui) {
    let th = c.t();
    let debug = app.settings.advanced.debug;
    let sections: Vec<(String, Vec<Cat>)> = vec![
        (
            t("settings.section.general"),
            vec![
                Cat { page: SettingsPage::Appearance, icon: "sun-high", label: t("settings.page.appearance"), color: th.accent },
                Cat { page: SettingsPage::Accessibility, icon: "accessible", label: t("settings.page.accessibility"), color: th.accent_2 },
            ],
        ),
        (
            t("settings.section.media"),
            vec![
                Cat { page: SettingsPage::Video, icon: "movie", label: t("settings.page.video"), color: th.danger },
                Cat { page: SettingsPage::Audio, icon: "music", label: t("settings.page.audio"), color: th.warning },
                Cat { page: SettingsPage::Metadata, icon: "file-download", label: t("settings.page.metadata"), color: th.success },
            ],
        ),
        (
            t("settings.section.processing"),
            vec![
                Cat { page: SettingsPage::Local, icon: "cpu", label: t("settings.page.local"), color: th.accent },
                Cat { page: SettingsPage::Instances, icon: "world", label: t("settings.page.instances"), color: th.accent_2 },
                Cat { page: SettingsPage::Privacy, icon: "lock", label: t("settings.page.privacy"), color: th.text_muted },
            ],
        ),
        (
            t("settings.section.app"),
            {
                let mut v = vec![
                    Cat { page: SettingsPage::Desktop, icon: "device-laptop", label: t("settings.page.desktop"), color: th.warning },
                    Cat { page: SettingsPage::Advanced, icon: "adjustments-star", label: t("settings.page.advanced"), color: th.danger },
                ];
                if debug {
                    v.push(Cat { page: SettingsPage::Debug, icon: "bug", label: t("settings.page.debug"), color: th.text_muted });
                }
                v
            },
        ),
    ];

    let full = ui.max_rect();
    let rail_w = 224.0;
    let rail = Rect::from_min_max(Pos2::new(full.left() + PAGE_PADDING, full.top()), Pos2::new(full.left() + PAGE_PADDING + rail_w, full.bottom()));
    let content = Rect::from_min_max(Pos2::new(rail.right() + 20.0, full.top()), full.max);

    let mut clicked = None;
    let mut rui = ui.new_child(egui::UiBuilder::new().max_rect(rail).layout(egui::Layout::top_down(Align::Min)));
    rui.set_clip_rect(rail);
    egui::ScrollArea::vertical().id_salt("settings-rail").scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(&mut rui, |ui| {
        ui.set_width(rail_w);
        ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);
        ui.add_space(8.0);
        for (si, (label, cats)) in sections.iter().enumerate() {
            if si > 0 {
                ui.add_space(12.0);
            }
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                widgets::text(ui, label.to_uppercase(), theme::semibold(10.5), th.text_faint);
            });
            ui.add_space(4.0);
            for cat in cats {
                if rail_item(ui, c, cat, app.settings_page == cat.page) {
                    clicked = Some(cat.page);
                }
            }
        }
    });
    if let Some(p) = clicked {
        app.settings_page = p;
    }

    let mut cui = ui.new_child(egui::UiBuilder::new().max_rect(content).layout(egui::Layout::top_down(Align::Min)));
    cui.set_clip_rect(content);
    egui::ScrollArea::vertical().id_salt(("settings-content", app.settings_page as u8)).auto_shrink([false, false]).show(&mut cui, |ui| {
        let w = (ui.available_width() - PAGE_PADDING - 8.0).min(720.0).max(320.0);
        ui.allocate_ui_with_layout(Vec2::new(w, 0.0), egui::Layout::top_down(Align::Min), |ui| {
            ui.add_space(8.0);
            ui.spacing_mut().item_spacing = Vec2::new(12.0, 14.0);
            match app.settings_page {
                SettingsPage::Appearance => appearance(app, c, ui, w),
                SettingsPage::Accessibility => accessibility(app, c, ui, w),
                SettingsPage::Video => video(app, c, ui, w),
                SettingsPage::Audio => audio(app, c, ui, w),
                SettingsPage::Metadata => metadata(app, c, ui, w),
                SettingsPage::Local => local(app, c, ui, w),
                SettingsPage::Instances => instances(app, c, ui, w),
                SettingsPage::Privacy => privacy(app, c, ui, w),
                SettingsPage::Desktop => desktop(app, c, ui, w),
                SettingsPage::Advanced => advanced(app, c, ui, w),
                SettingsPage::Debug => debug_page(app, c, ui, w),
            }
            ui.add_space(40.0);
        });
    });
    app.settings_changed();
}

fn rail_item(ui: &mut Ui, c: &Ctx, cat: &Cat, active: bool) -> bool {
    let th = c.t();
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 38.0), Sense::click());
    let bg = if active { th.surface } else if resp.hovered() { th.surface_hover } else { Color32::TRANSPARENT };
    ui.painter().rect_filled(rect, cr(10.0), bg);
    if active {
        ui.painter().rect_stroke(rect, cr(10.0), Stroke::new(1.0, th.border), StrokeKind::Inside);
    }
    let tile = Rect::from_center_size(Pos2::new(rect.left() + 12.0 + 12.0, rect.center().y), Vec2::splat(24.0));
    widgets::icon_tile_at(ui, c, tile, cat.icon, cat.color);
    ui.painter().text(Pos2::new(tile.right() + 10.0, rect.center().y), Align2::LEFT_CENTER, cap(&cat.label), theme::medium(13.5), if active { th.text } else { th.text_muted });
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

// ---------- helpers ----------

fn section(ui: &mut Ui, c: &Ctx, w: f32, title: &str, beta: bool, rows: impl FnOnce(&mut Ui)) {
    let th = c.t();
    widgets::card(ui, c, w, |ui| {
        ui.spacing_mut().item_spacing = Vec2::new(8.0, 14.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            widgets::text(ui, cap(title), theme::semibold(15.0), th.text);
            if beta {
                widgets::badge(ui, c, "BETA", th.accent);
            }
        });
        rows(ui);
    });
}

fn seg_opts<'a>(values: &'a [&'a str], label: impl Fn(&str) -> String) -> Vec<SegOption<'a>> {
    values.iter().map(|v| SegOption { value: v, label: label(v), icon: None }).collect()
}

fn seg_row(ui: &mut Ui, c: &Ctx, title: &str, desc: &str, current: &mut String, options: &[SegOption]) {
    if options.len() <= 3 {
        widgets::setting_row(ui, c, title, desc, 280.0, |ui| {
            if let Some(v) = widgets::segmented(ui, c, current, options, true, 36.0) {
                *current = v;
            }
        });
    } else {
        widgets::setting_block(ui, c, title, desc, |ui| {
            if let Some(v) = widgets::segmented(ui, c, current, options, true, 36.0) {
                *current = v;
            }
        });
    }
}

fn lang_items(first: (&str, String)) -> Vec<(String, String)> {
    let mut items = vec![(first.0.to_string(), first.1)];
    for code in cfg::LANGUAGES {
        items.push((code.to_string(), format!("{} ({code})", cfg::language_name(code))));
    }
    items
}

fn dropdown_row(ui: &mut Ui, c: &Ctx, id: &str, title: &str, desc: &str, items: &[(String, String)], current: &mut String, disabled: bool) {
    widgets::setting_row(ui, c, title, desc, 260.0, |ui| {
        widgets::dropdown(ui, c, id, items, current, 260.0, disabled);
    });
}

// ---------- pages ----------

fn appearance(app: &mut App, c: &Ctx, ui: &mut Ui, w: f32) {
    let s = &mut app.settings;
    section(ui, c, w, &t("settings.page.appearance"), false, |ui| {
        let options = seg_opts(cfg::THEME_OPTIONS, |v| cap(&t(&format!("settings.theme.{v}"))));
        seg_row(ui, c, &cap(&t("settings.theme")), &t("settings.theme.description"), &mut s.appearance.theme, &options);
        widgets::divider(ui, c);
        widgets::toggle_row(ui, c, &cap(&t("settings.language.auto.title")), &t("settings.language.auto.description"), &mut s.appearance.auto_language, false);
        widgets::divider(ui, c);
        let items: Vec<(String, String)> = crate::i18n::LOCALES.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        dropdown_row(ui, c, "language", &cap(&t("settings.language.preferred.title")), &t("settings.language.preferred.description"), &items, &mut s.appearance.language, s.appearance.auto_language);
    });
    section(ui, c, w, &t("settings.tabs"), false, |ui| {
        widgets::toggle_row(ui, c, &cap(&t("settings.tabs.hide_remux")), &t("settings.tabs.hide_remux.description"), &mut s.appearance.hide_remux_tab, false);
    });
}

fn accessibility(app: &mut App, c: &Ctx, ui: &mut Ui, w: f32) {
    let s = &mut app.settings;
    section(ui, c, w, &t("settings.accessibility.visual"), false, |ui| {
        widgets::toggle_row(ui, c, &cap(&t("settings.accessibility.motion.title")), &t("settings.accessibility.motion.description"), &mut s.accessibility.reduce_motion, false);
        widgets::divider(ui, c);
        widgets::toggle_row(ui, c, &cap(&t("settings.accessibility.transparency.title")), &t("settings.accessibility.transparency.description"), &mut s.accessibility.reduce_transparency, false);
    });
    section(ui, c, w, &t("settings.accessibility.behavior"), false, |ui| {
        widgets::toggle_row(ui, c, &cap(&t("settings.accessibility.auto_queue.title")), &t("settings.accessibility.auto_queue.description"), &mut s.accessibility.dont_auto_open_queue, false);
    });
}

fn video(app: &mut App, c: &Ctx, ui: &mut Ui, w: f32) {
    let s = &mut app.settings;
    section(ui, c, w, &t("settings.page.video"), false, |ui| {
        let options = seg_opts(cfg::VIDEO_QUALITY_OPTIONS, |v| t(&format!("settings.video.quality.{v}")));
        seg_row(ui, c, &cap(&t("settings.video.quality")), &t("settings.video.quality.description"), &mut s.save.video_quality, &options);
        widgets::divider(ui, c);
        widgets::toggle_row(ui, c, &cap(&t("settings.video.h265.title")), &t("settings.video.h265.description"), &mut s.save.allow_h265, false);
        widgets::divider(ui, c);
        widgets::toggle_row(ui, c, &cap(&t("settings.video.twitter.gif.title")), &t("settings.video.twitter.gif.description"), &mut s.save.convert_gif, false);
    });
    section(ui, c, w, "youtube", false, |ui| {
        let options = seg_opts(cfg::YOUTUBE_VIDEO_CODEC_OPTIONS, |v| match v { "h264" => "h264 + aac", "av1" => "av1 + opus", _ => "vp9 + opus" }.to_string());
        seg_row(ui, c, &cap(&t("settings.video.youtube.codec")), &t("settings.video.youtube.codec.description"), &mut s.save.youtube_video_codec, &options);
        widgets::divider(ui, c);
        let options = seg_opts(cfg::YOUTUBE_VIDEO_CONTAINER_OPTIONS, |v| v.to_string());
        seg_row(ui, c, &cap(&t("settings.video.youtube.container")), &t("settings.video.youtube.container.description"), &mut s.save.youtube_video_container, &options);
    });
}

fn audio(app: &mut App, c: &Ctx, ui: &mut Ui, w: f32) {
    let s = &mut app.settings;
    section(ui, c, w, &t("settings.page.audio"), false, |ui| {
        let options = seg_opts(cfg::AUDIO_FORMAT_OPTIONS, |v| t(&format!("settings.audio.format.{v}")));
        seg_row(ui, c, &cap(&t("settings.audio.format")), &t("settings.audio.format.description"), &mut s.save.audio_format, &options);
        widgets::divider(ui, c);
        let bitrate_disabled = matches!(s.save.audio_format.as_str(), "wav" | "best");
        ui.scope(|ui| {
            if bitrate_disabled {
                ui.disable();
                ui.set_opacity(0.45);
            }
            let kbps = t("settings.audio.bitrate.kbps");
            let options = seg_opts(cfg::AUDIO_BITRATE_OPTIONS, |v| format!("{v}{kbps}"));
            seg_row(ui, c, &cap(&t("settings.audio.bitrate")), &t("settings.audio.bitrate.description"), &mut s.save.audio_bitrate, &options);
        });
    });
    section(ui, c, w, "youtube", false, |ui| {
        widgets::toggle_row(ui, c, &cap(&t("settings.audio.youtube.better_audio.title")), &t("settings.audio.youtube.better_audio.description"), &mut s.save.youtube_better_audio, false);
        widgets::divider(ui, c);
        let items = lang_items(("original", cap(&t("settings.youtube.dub.original"))));
        dropdown_row(ui, c, "dub", &cap(&t("settings.audio.youtube.dub.title")), &t("settings.audio.youtube.dub.description"), &items, &mut s.save.youtube_dub_lang, false);
    });
    section(ui, c, w, &t("settings.audio.tiktok.original"), false, |ui| {
        widgets::toggle_row(ui, c, &cap(&t("settings.audio.tiktok.original.title")), &t("settings.audio.tiktok.original.description"), &mut s.save.tiktok_full_audio, false);
    });
}

fn metadata(app: &mut App, c: &Ctx, ui: &mut Ui, w: f32) {
    let s = &mut app.settings;
    section(ui, c, w, &t("settings.metadata.filename"), false, |ui| {
        let options = seg_opts(cfg::FILENAME_STYLE_OPTIONS, |v| cap(&t(&format!("settings.metadata.filename.{v}"))));
        widgets::setting_block(ui, c, &cap(&t("settings.metadata.filename")), &t("settings.metadata.filename.description"), |ui| {
            if let Some(v) = widgets::segmented(ui, c, &s.save.filename_style, &options, true, 36.0) {
                s.save.filename_style = v;
            }
        });
        filename_preview(ui, c, s);
    });
    section(ui, c, w, &t("settings.saving.title"), false, |ui| {
        let options = seg_opts(cfg::SAVING_METHOD_OPTIONS, |v| cap(&t(&format!("settings.saving.{v}"))));
        seg_row(ui, c, &cap(&t("settings.saving.title")), &t("settings.saving.description"), &mut s.save.saving_method, &options);
    });
    section(ui, c, w, &t("settings.metadata.file"), false, |ui| {
        let items = lang_items(("none", cap(&t("settings.subtitles.none"))));
        dropdown_row(ui, c, "subs", &cap(&t("settings.subtitles.title")), &t("settings.subtitles.description"), &items, &mut s.save.subtitle_lang, false);
        widgets::divider(ui, c);
        widgets::toggle_row(ui, c, &cap(&t("settings.metadata.disable.title")), &t("settings.metadata.disable.description"), &mut s.save.disable_metadata, false);
    });
}

/// port of FilenamePreview.svelte
fn filename_preview(ui: &mut Ui, c: &Ctx, s: &CobaltSettings) {
    let th = c.t();
    let video_title = t("settings.metadata.filename.preview.video");
    let audio_title = t("settings.metadata.filename.preview.audio");
    let info_base = ["youtube", "dQw4w9WgXcQ"];
    let full_resolution = |q: &str| match q {
        "2160" => "3840x2160",
        "1440" => "2560x1440",
        "1080" => "1920x1080",
        "720" => "1280x720",
        "480" => "854x480",
        "360" => "640x360",
        "240" => "426x240",
        _ => "256x144",
    };
    let codec = s.save.youtube_video_codec.as_str();
    let youtube_ext = if codec == "vp9" { "webm" } else { "mp4" };
    let audio_format = if s.save.audio_format != "best" { s.save.audio_format.clone() } else { "opus".into() };
    let mut quality = if s.save.video_quality != "max" { s.save.video_quality.clone() } else { "2160".into() };
    if codec == "h264" && quality.parse::<u32>().unwrap_or(0) > 1080 {
        quality = "1080".into();
    }
    let mut classic_tags: Vec<String> = info_base.iter().map(|x| x.to_string()).collect();
    classic_tags.push(full_resolution(&quality).into());
    classic_tags.push(codec.into());
    let mut basic_tags = vec![format!("{quality}p"), codec.to_string()];
    if s.save.download_mode == "mute" {
        classic_tags.push("mute".into());
        basic_tags.push("mute".into());
    }
    let (video, audio) = match s.save.filename_style.as_str() {
        "classic" => (classic_tags.join("_"), "youtube_dQw4w9WgXcQ_audio".to_string()),
        "pretty" => {
            let mut tags = basic_tags.clone();
            tags.push(info_base[0].into());
            (format!("{video_title} ({})", tags.join(", ")), format!("{audio_title} ({})", info_base[0]))
        }
        "nerdy" => {
            let mut tags = basic_tags.clone();
            tags.extend(info_base.iter().map(|x| x.to_string()));
            (format!("{video_title} ({})", tags.join(", ")), format!("{audio_title} ({})", info_base.join(", ")))
        }
        _ => (format!("{video_title} ({})", basic_tags.join(", ")), audio_title.clone()),
    };
    let w = ui.available_width();
    egui::Frame::new().fill(th.inset).corner_radius(cr(12.0)).inner_margin(widgets::margin(14.0, 12.0)).show(ui, |ui| {
        ui.set_width(w - 28.0);
        ui.spacing_mut().item_spacing.y = 10.0;
        for (icon, color, name, desc) in [
            ("movie", th.accent, format!("{video}.{youtube_ext}"), t("settings.filename.preview_desc.video")),
            ("music", th.accent_2, format!("{audio}.{audio_format}"), t("settings.filename.preview_desc.audio")),
        ] {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                widgets::icon_tile(ui, c, icon, 30.0, color);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    let mut job = widgets::wrapped_job(&name, theme::mono_medium(12.5), th.text, w - 28.0 - 40.0, 1.35);
                    job.wrap.break_anywhere = true;
                    widgets::galley_label(ui, job, th.text);
                    widgets::text(ui, cap(&desc), theme::regular(12.0), th.text_muted);
                });
            });
        }
    });
}

fn local(app: &mut App, c: &Ctx, ui: &mut Ui, w: f32) {
    let s = &mut app.settings;
    section(ui, c, w, &t("settings.local.saving"), true, |ui| {
        let options = seg_opts(cfg::LOCAL_PROCESSING_OPTIONS, |v| cap(&t(&format!("settings.local.saving.{v}"))));
        seg_row(ui, c, &cap(&t("settings.local.saving")), &t("settings.local.saving.description"), &mut s.save.local_processing, &options);
    });
}

fn instances(app: &mut App, c: &Ctx, ui: &mut Ui, w: f32) {
    let th = c.t();
    let seen = app.settings.processing.seen_custom_warning;
    let mut show_warning = false;
    {
        let s = &mut app.settings;
        section(ui, c, w, &t("settings.processing.community"), false, |ui| {
            let before = s.processing.enable_custom_instances;
            widgets::toggle_row(ui, c, &cap(&t("settings.processing.enable_custom.title")), &t("settings.processing.enable_custom.description"), &mut s.processing.enable_custom_instances, false);
            if s.processing.enable_custom_instances && !before && !seen {
                show_warning = true;
            }
            if s.processing.enable_custom_instances {
                let iw = ui.available_width();
                let r = widgets::text_input(ui, c, "custom-instance", &mut s.processing.custom_instance_url, iw, InputOpts { placeholder: "https://instance.url.example/", icon: Some("world"), mono: true, ..Default::default() });
                if r.changed() {
                    s.processing.custom_instance_url = s.processing.custom_instance_url.trim().to_string();
                }
            }
        });
        section(ui, c, w, &t("settings.processing.access_key"), false, |ui| {
            widgets::toggle_row(ui, c, &cap(&t("settings.processing.access_key.title")), &t("settings.processing.access_key.description"), &mut s.processing.enable_custom_api_key, false);
            if s.processing.enable_custom_api_key {
                let iw = ui.available_width();
                widgets::text_input(ui, c, "api-key", &mut s.processing.custom_api_key, iw, InputOpts { placeholder: "00000000-0000-0000-0000-000000000000", icon: Some("lock"), mono: true, sensitive: true, ..Default::default() });
            }
        });
    }
    section(ui, c, w, &t("settings.processing.status"), false, |ui| {
        let (color, label) = crate::ui::instance_status(app, c);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let (r, _) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
            ui.painter().circle_filled(r.center(), 5.0, color);
            widgets::text(ui, cap(&label), theme::medium(14.0), th.text);
            if app.server_info_loading {
                widgets::spinner(ui, c, 14.0, th.text_faint);
            }
        });
        match (&app.server_info, &app.server_info_error) {
            (Some(info), _) => {
                let lines = [
                    (t("settings.processing.custom_instance.input.alt_text"), info.origin.clone()),
                    ("version".to_string(), format!("v{} · {} {}", info.info.cobalt.version, info.info.cobalt.services.len(), t("settings.processing.status.services"))),
                    ("git".to_string(), format!("{}@{} ({})", info.info.git.remote, info.info.git.branch, &info.info.git.commit[..7.min(info.info.git.commit.len())])),
                ];
                for (k, v) in lines {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 12.0;
                        ui.allocate_ui_with_layout(Vec2::new(120.0, 20.0), egui::Layout::left_to_right(Align::Center), |ui| {
                            widgets::text(ui, cap(&k), theme::regular(12.5), th.text_faint);
                        });
                        widgets::text(ui, v, theme::mono(12.5), th.text_muted);
                    });
                }
            }
            (None, Some(err)) => {
                widgets::wrapped(ui, &cap(err), theme::regular(13.0), th.danger, ui.available_width(), 1.45);
            }
            _ => {}
        }
        if widgets::button(ui, c, &cap(&t("button.retry")), Some("reload"), ButtonOpts::secondary().small()).clicked() {
            app.client.clear_server_info();
            app.server_info = None;
            app.server_info_error = None;
            app.load_server_info();
        }
    });
    if show_warning {
        app.custom_instance_warning();
    }
}

fn privacy(app: &mut App, c: &Ctx, ui: &mut Ui, w: f32) {
    let s = &mut app.settings;
    section(ui, c, w, &t("settings.privacy.tunnel"), false, |ui| {
        widgets::toggle_row(ui, c, &cap(&t("settings.privacy.tunnel.title")), &t("settings.privacy.tunnel.description"), &mut s.save.always_proxy, false);
    });
}

fn desktop(app: &mut App, c: &Ctx, ui: &mut Ui, w: f32) {
    let th = c.t();
    let dir = app.settings.download_dir();
    let mut pick_folder = false;
    let mut open_folder = false;
    let mut pick_ffmpeg = false;
    let mut clear_ffmpeg = false;
    section(ui, c, w, &t("settings.desktop.downloads"), false, |ui| {
        widgets::setting_block(ui, c, &cap(&t("settings.desktop.downloads.title")), &t("settings.desktop.downloads.description"), |ui| {
            let iw = ui.available_width();
            egui::Frame::new().fill(th.inset).corner_radius(cr(10.0)).inner_margin(widgets::margin(12.0, 10.0)).show(ui, |ui| {
                ui.set_width(iw - 24.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    let (r, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                    c.icons.image("folder", 16.0, th.text_muted).paint_at(ui, r);
                    let mut job = widgets::wrapped_job(&dir.display().to_string(), theme::mono(12.5), th.text, iw - 60.0, 1.35);
                    job.wrap.break_anywhere = true;
                    widgets::galley_label(ui, job, th.text);
                });
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                if widgets::button(ui, c, &cap(&t("settings.desktop.downloads.choose")), Some("folder"), ButtonOpts::secondary().small()).clicked() {
                    pick_folder = true;
                }
                if widgets::button(ui, c, &cap(&t("settings.desktop.downloads.open")), Some("external-link"), ButtonOpts::ghost().small()).clicked() {
                    open_folder = true;
                }
            });
        });
    });
    let ffmpeg_status = match &app.ffmpeg_path {
        Some(p) => tf("settings.desktop.ffmpeg.found", &[("value", &p.display().to_string())]),
        None => t("settings.desktop.ffmpeg.missing"),
    };
    let ok = app.ffmpeg_path.is_some();
    section(ui, c, w, &t("settings.desktop.ffmpeg"), false, |ui| {
        widgets::setting_block(ui, c, &cap(&t("settings.desktop.ffmpeg.title")), &t("settings.desktop.ffmpeg.description"), |ui| {
            let iw = ui.available_width();
            widgets::text_input(ui, c, "ffmpeg-path", &mut app.settings.desktop.ffmpeg_path, iw, InputOpts { placeholder: "ffmpeg", icon: Some("cpu"), mono: true, ..Default::default() });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                if widgets::button(ui, c, &cap(&t("settings.desktop.ffmpeg.browse")), Some("file"), ButtonOpts::secondary().small()).clicked() {
                    pick_ffmpeg = true;
                }
                if !app.settings.desktop.ffmpeg_path.is_empty() && widgets::button(ui, c, &cap(&t("button.reset")), Some("restore"), ButtonOpts::ghost().small()).clicked() {
                    clear_ffmpeg = true;
                }
            });
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                let (r, _) = ui.allocate_exact_size(Vec2::splat(14.0), Sense::hover());
                c.icons.image(if ok { "check" } else { "exclamation-circle" }, 14.0, if ok { th.success } else { th.danger }).paint_at(ui, r);
                let mut job = widgets::wrapped_job(&cap(&ffmpeg_status), theme::regular(12.5), if ok { th.success } else { th.danger }, iw - 24.0, 1.4);
                job.wrap.break_anywhere = true;
                widgets::galley_label(ui, job, if ok { th.success } else { th.danger });
            });
        });
    });
    if pick_folder {
        app.pick_download_folder();
    }
    if open_folder {
        let _ = std::fs::create_dir_all(&dir);
        let _ = open::that_detached(dir);
    }
    if pick_ffmpeg {
        app.pick_ffmpeg();
    }
    if clear_ffmpeg {
        app.settings.desktop.ffmpeg_path.clear();
    }
}

fn advanced(app: &mut App, c: &Ctx, ui: &mut Ui, w: f32) {
    let (mut export, mut import, mut reset, mut clear) = (false, false, false, false);
    section(ui, c, w, &t("settings.advanced.debug"), false, |ui| {
        widgets::toggle_row(ui, c, &cap(&t("settings.advanced.debug.title")), &t("settings.advanced.debug.description"), &mut app.settings.advanced.debug, false);
    });
    section(ui, c, w, &t("settings.advanced.settings_data"), false, |ui| {
        widgets::wrap_row(ui, 8.0, |ui| {
            if widgets::button(ui, c, &cap(&t("button.import")), Some("file-import"), ButtonOpts::secondary()).clicked() {
                import = true;
            }
            if widgets::button(ui, c, &cap(&t("button.export")), Some("file-export"), ButtonOpts::secondary()).clicked() {
                export = true;
            }
            if widgets::button(ui, c, &cap(&t("button.reset")), Some("file-shredder"), ButtonOpts::new(ButtonKind::Danger)).clicked() {
                reset = true;
            }
        });
    });
    section(ui, c, w, &t("settings.advanced.local_storage"), false, |ui| {
        if widgets::button(ui, c, &cap(&t("button.clear_cache")), Some("trash"), ButtonOpts::secondary()).clicked() {
            clear = true;
        }
    });
    if export {
        app.export_settings();
    }
    if import {
        app.import_settings();
    }
    if reset {
        app.reset_settings_dialog();
    }
    if clear {
        app.clear_cache_dialog();
    }
}

fn debug_page(app: &mut App, c: &Ctx, ui: &mut Ui, w: f32) {
    let th = c.t();
    let device = serde_json::json!({
        "os": std::env::consts::OS, "arch": std::env::consts::ARCH,
        "ffmpeg": app.ffmpeg_path.as_ref().map(|p| p.display().to_string()),
    });
    let version = serde_json::json!({ "version": env!("CARGO_PKG_VERSION"), "web_reference": "11.7", "api_reference": "11.7.1" });
    let settings: serde_json::Value = serde_json::from_str(&app.settings.to_json()).unwrap_or_default();
    let server = app.server_info.as_ref().map(|i| serde_json::json!({ "origin": i.origin, "version": i.info.cobalt.version, "services": i.info.cobalt.services, "turnstile": i.info.cobalt.turnstile_sitekey.is_some() })).unwrap_or(serde_json::Value::Null);
    let queue: Vec<serde_json::Value> = app.tm.snapshot().iter().map(|i| serde_json::json!({ "id": i.id, "filename": i.filename, "state": format!("{:?}", i.state) })).collect();
    for (title, data) in [("device", device), ("app", version), ("settings", settings), ("server", server), ("queue", serde_json::Value::Array(queue))] {
        let text = serde_json::to_string_pretty(&data).unwrap_or_default();
        section(ui, c, w, title, false, |ui| {
            let iw = ui.available_width();
            egui::Frame::new().fill(th.inset).corner_radius(cr(10.0)).inner_margin(widgets::margin(12.0, 10.0)).show(ui, |ui| {
                ui.set_width(iw - 24.0);
                let mut job = widgets::wrapped_job(&text, theme::mono(12.0), th.text_muted, iw - 24.0, 1.4);
                job.wrap.break_anywhere = true;
                widgets::galley_label(ui, job, th.text_muted);
            });
            if widgets::button(ui, c, &cap(&t("button.copy")), Some("copy"), ButtonOpts::ghost().small()).clicked() {
                app.copy_text(&text);
            }
        });
    }
}
