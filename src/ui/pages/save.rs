//! download page: hero, link card, supported services, inline activity

use crate::app::{App, ButtonState, Page, SettingsPage};
use crate::i18n::t;
use crate::theme::{self, cap};
use crate::ui::queue_ui;
use crate::ui::widgets::{self, ButtonKind, ButtonOpts, Ctx, InputOpts, SegOption};
use crate::ui::{instance_status, page_body};
use egui::{Align, Pos2, Rect, Sense, Ui, Vec2};

pub fn show(app: &mut App, c: &Ctx, ui: &mut Ui) {
    let th = c.t();
    app.tick_button_state();

    let full = ui.max_rect();
    widgets::glow(ui.painter(), Pos2::new(full.center().x, full.top() - 30.0), full.width() * 0.55, widgets::with_alpha(th.accent, if th.dark { 0.22 } else { 0.14 }));

    page_body(ui, "download", 860.0, |ui, w| {
        ui.add_space(26.0);
        ui.vertical_centered(|ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            widgets::text(ui, cap(&t("home.title")), theme::bold(32.0), th.text);
            let mut job = widgets::wrapped_job(&cap(&t("home.subtitle")), theme::regular(15.0), th.text_muted, (w - 120.0).max(200.0), 1.5);
            job.halign = Align::Center;
            ui.label(job);
        });
        ui.add_space(10.0);

        link_card(app, c, ui, w);

        if let Some(info) = &app.server_info {
            if info.info.cobalt.turnstile_sitekey.is_some() && !app.settings.processing.enable_custom_api_key {
                widgets::banner(ui, c, "alert-triangle", th.warning, th.warning_soft, &cap(&t("error.api.turnstile_desktop")));
            }
        }
        if app.ffmpeg_path.is_none() {
            widgets::banner(ui, c, "alert-triangle", th.danger, th.danger_soft, &cap(&t("error.ffmpeg.not_found")));
        }

        services(app, c, ui, w);
        ui.add_space(6.0);
        activity(app, c, ui, w);
    });

    keyboard(app, ui);
}

fn link_card(app: &mut App, c: &Ctx, ui: &mut Ui, w: f32) {
    let th = c.t();
    widgets::card(ui, c, w, |ui| {
        ui.spacing_mut().item_spacing = Vec2::new(10.0, 14.0);
        let downloadable = crate::api::valid_link(&app.link);
        let loading = app.is_loading();
        let disabled = app.button_state != ButtonState::Idle;
        let inner_w = ui.available_width();

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            let btn_w = 150.0;
            let show_clear = !app.link.is_empty() && !loading;
            let clear_w = if show_clear { 44.0 + 10.0 } else { 0.0 };
            let input_w = inner_w - btn_w - clear_w - 10.0;
            let resp = widgets::text_input(
                ui,
                c,
                "link-area",
                &mut app.link,
                input_w,
                InputOpts { placeholder: &cap(&t("save.input.placeholder")), icon: Some("link"), mono: true, height: 48.0, font_size: 14.5, ..Default::default() },
            );
            if app.focus_input && app.dialogs.is_empty() {
                resp.request_focus();
                app.focus_input = false;
            }
            if resp.changed() {
                app.link = app.link.replace(['\n', '\r'], "");
            }
            if show_clear && widgets::icon_button(ui, c, "x", 44.0, ButtonKind::Ghost).clicked() {
                app.link.clear();
                app.focus_input = true;
            }
            let (label, icon) = match app.button_state {
                ButtonState::Idle => (cap(&t("home.download")), Some("arrow-down")),
                ButtonState::Think | ButtonState::Check => (String::new(), None),
                ButtonState::Done => (String::new(), Some("check")),
                ButtonState::Error => (String::new(), Some("exclamation-circle")),
            };
            let mut opts = ButtonOpts::primary().height(48.0).disabled(disabled || !downloadable).mask(th.surface);
            opts.font_size = 14.5;
            opts.full_width = true;
            // loading state keeps the primary look
            if loading {
                opts.disabled = false;
            }
            let start = ui.cursor().min;
            let r = ui
                .allocate_ui_with_layout(Vec2::new(btn_w, 48.0), egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                    widgets::button(ui, c, &label, icon, opts)
                })
                .inner;
            if loading {
                let br = Rect::from_min_size(start, Vec2::new(btn_w, 48.0));
                widgets::spinner_at(ui, c, Rect::from_center_size(br.center(), Vec2::splat(20.0)), th.accent_text);
            }
            if r.clicked() && !disabled && downloadable {
                let link = app.link.clone();
                app.saving_handler(Some(link), None, None);
            }
        });

        let mode = app.settings.save.download_mode.clone();
        let summary = {
            let s = &app.settings.save;
            let quality = t(&format!("settings.video.quality.{}", s.video_quality));
            if s.download_mode == "audio" {
                if s.audio_format == "best" { "best audio".to_string() } else { format!("{} · {}kb/s", s.audio_format, s.audio_bitrate) }
            } else {
                format!("{quality} · {}", s.youtube_video_codec)
            }
        };
        let mut new_mode: Option<String> = None;
        let mut paste = false;
        let mut open_settings = false;
        egui::Sides::new().height(36.0).show(
            ui,
            |ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                let options = [
                    SegOption { value: "auto", label: cap(&t("save.auto")), icon: Some("sparkles") },
                    SegOption { value: "audio", label: cap(&t("save.audio")), icon: Some("music") },
                    SegOption { value: "mute", label: cap(&t("save.mute")), icon: Some("volume-off") },
                ];
                new_mode = widgets::segmented(ui, c, &mode, &options, false, 36.0);
                if widgets::button(ui, c, &cap(&t("save.paste")), Some("clipboard"), ButtonOpts::secondary().height(36.0)).clicked() {
                    paste = true;
                }
            },
            |ui| {
                let r = widgets::button(ui, c, &summary, Some("adjustments-star"), ButtonOpts::ghost().height(36.0)).on_hover_text(cap(&t("home.presets")));
                open_settings = r.clicked();
            },
        );
        if let Some(v) = new_mode {
            app.settings.save.download_mode = v;
            app.settings_changed();
        }
        if paste {
            app.paste_from_clipboard();
        }
        if open_settings {
            app.page = Page::Settings;
            app.settings_page = if app.settings.save.download_mode == "audio" { SettingsPage::Audio } else { SettingsPage::Video };
        }
    });
}

fn services(app: &mut App, c: &Ctx, ui: &mut Ui, w: f32) {
    let th = c.t();
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
        let (color, label) = instance_status(app, c);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            let (r, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
            ui.painter().circle_filled(r.center(), 4.0, color);
            widgets::text(ui, cap(&t("save.services.title")), theme::medium(12.5), th.text_muted);
            widgets::text(ui, format!("· {}", label), theme::regular(12.5), th.text_faint);
        });
        ui.add_space(4.0);
        match &app.server_info {
            Some(info) => {
                let services = &info.info.cobalt.services;
                let limit = if app.services_expanded { services.len() } else { 8.min(services.len()) };
                for s in services.iter().take(limit) {
                    let _ = widgets::chip(ui, c, s, false);
                }
                if services.len() > 8 {
                    let label = if app.services_expanded { cap(&t("home.services.hide")) } else { format!("+{} more", services.len() - 8) };
                    if widgets::chip(ui, c, &label, true).clicked() {
                        app.services_expanded = !app.services_expanded;
                    }
                }
            }
            None => {
                if let Some(err) = &app.server_info_error {
                    widgets::wrapped(ui, &cap(err), theme::regular(12.5), th.danger, w - 40.0, 1.4);
                } else {
                    widgets::spinner(ui, c, 14.0, th.text_faint);
                }
            }
        }
    });
}

fn activity(app: &mut App, c: &Ctx, ui: &mut Ui, w: f32) {
    let th = c.t();
    let items = app.tm.snapshot();
    egui::Sides::new().height(32.0).show(
        ui,
        |ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            widgets::text(ui, cap(&t("home.activity")), theme::semibold(16.0), th.text);
            if !items.is_empty() {
                widgets::badge(ui, c, &items.len().to_string(), th.accent);
            }
        },
        |ui| {
            if !items.is_empty() && widgets::button(ui, c, &cap(&t("button.clear")), Some("trash"), ButtonOpts::ghost().small()).clicked() {
                app.tm.clear_queue();
            }
        },
    );
    if items.is_empty() {
        widgets::card(ui, c, w, |ui| {
            widgets::empty_state(ui, c, "arrow-down", &cap(&t("home.activity.empty.title")), &cap(&t("home.activity.empty.description")));
        });
    } else {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 10.0;
            for item in &items {
                queue_ui::activity_card(app, c, ui, item, w);
            }
        });
    }
}

/// same shortcuts as cobalt's omnibox
fn keyboard(app: &mut App, ui: &mut Ui) {
    if !app.dialogs.is_empty() || app.is_loading() {
        return;
    }
    let focused_id = ui.make_persistent_id("link-area");
    let input_focused = ui.ctx().memory(|m| m.has_focus(focused_id));
    let (enter, escape, slash, d, j, k, l) = ui.input(|i| {
        (
            i.key_pressed(egui::Key::Enter),
            i.key_pressed(egui::Key::Escape),
            i.key_pressed(egui::Key::Slash),
            i.key_pressed(egui::Key::D) && i.modifiers.shift,
            i.key_pressed(egui::Key::J) && i.modifiers.shift,
            i.key_pressed(egui::Key::K) && i.modifiers.shift,
            i.key_pressed(egui::Key::L) && i.modifiers.shift,
        )
    });
    if slash && !input_focused {
        app.focus_input = true;
    }
    if input_focused {
        if enter && crate::api::valid_link(&app.link) {
            let link = app.link.clone();
            app.saving_handler(Some(link), None, None);
        }
        if escape {
            app.link.clear();
        }
        return;
    }
    if d {
        app.paste_from_clipboard();
    } else if j || k || l {
        app.settings.save.download_mode = if j { "auto" } else if k { "audio" } else { "mute" }.into();
        app.settings_changed();
    }
}
