//! modal dialogs: message, picker, saving

use super::widgets::{self, cr, ButtonKind, ButtonOpts, Ctx};
use crate::app::{App, DialogAction, DialogButton, DialogKind};
use crate::i18n::t;
use crate::theme::{self, cap};
use egui::{Align, Color32, Frame, Rect, Sense, Stroke, Ui, Vec2};

pub fn show(app: &mut App, c: &Ctx, ctx: &egui::Context) {
    let Some(dialog) = app.dialogs.last().cloned() else { return };
    let th = c.t();
    let mut action: Option<DialogAction> = None;
    let frame = Frame::new()
        .fill(th.surface)
        .stroke(Stroke::new(1.0, th.border_strong))
        .corner_radius(cr(18.0))
        .inner_margin(widgets::margin(24.0, 22.0))
        .shadow(egui::epaint::Shadow { offset: [0, 16], blur: 48, spread: 0, color: th.shadow });

    let modal = egui::Modal::new(egui::Id::new(("dialog", &dialog.id))).backdrop_color(th.backdrop).frame(frame).show(ctx, |ui| match &dialog.kind {
        DialogKind::Small { icon, icon_color, meowbalt, title, body, body_sub, left_aligned: _ } => {
            ui.set_width(420.0);
            let (icon, color) = match (icon, meowbalt) {
                (Some(i), _) => (*i, icon_color.map(|n| color_by_name(th, n)).unwrap_or(th.accent)),
                (None, Some("error")) => ("exclamation-circle", th.danger),
                _ => ("info-circle", th.accent),
            };
            let title = if title.is_empty() {
                if matches!(meowbalt, Some("error")) { cap(&t("dialog.error.title")) } else { cap(&t("dialog.notice.title")) }
            } else {
                cap(title)
            };
            message_dialog(ui, c, icon, color, &title, body, body_sub);
            action = buttons_row(ui, c, &dialog.buttons);
        }
        DialogKind::Picker { items, audio: _ } => {
            let w = (ctx.content_rect().width() - 120.0).min(560.0).max(320.0);
            ui.set_width(w);
            action = picker_dialog(app, ui, c, items, &dialog.buttons, w);
        }
        DialogKind::Saving { url, file, body } => {
            ui.set_width(400.0);
            let name = file.as_ref().map(|f| f.2.clone()).or_else(|| url.as_ref().map(|u| u.1.clone())).unwrap_or_default();
            message_dialog(ui, c, "download", th.accent, &cap(&t("dialog.saving.title")), &name, body);
            action = buttons_column(ui, c, &dialog.buttons);
        }
    });

    if modal.should_close() && dialog.dismissable && action.is_none() {
        action = Some(DialogAction::Close);
    }
    if let Some(a) = action {
        app.run_action(a);
    }
}

fn color_by_name(th: &crate::theme::Theme, name: &str) -> Color32 {
    match name {
        "red" => th.danger,
        "green" => th.success,
        "orange" | "yellow" => th.warning,
        _ => th.accent,
    }
}

fn message_dialog(ui: &mut Ui, c: &Ctx, icon: &str, color: Color32, title: &str, body: &str, body_sub: &str) {
    let th = c.t();
    ui.spacing_mut().item_spacing = Vec2::new(0.0, 10.0);
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 14.0;
        widgets::icon_tile(ui, c, icon, 44.0, color);
        let w = ui.available_width();
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            ui.add_space(2.0);
            widgets::wrapped(ui, title, theme::semibold(17.0), th.text, w, 1.25);
            if !body.is_empty() {
                widgets::wrapped(ui, body, theme::regular(13.5), th.text_muted, w, 1.55);
            }
            if !body_sub.is_empty() {
                widgets::wrapped(ui, body_sub, theme::regular(12.5), th.text_faint, w, 1.45);
            }
        });
    });
    ui.add_space(10.0);
}

fn button_opts(b: &DialogButton) -> ButtonOpts {
    if b.red {
        ButtonOpts::danger()
    } else if b.main {
        ButtonOpts::primary()
    } else {
        ButtonOpts::secondary()
    }
}

fn buttons_row(ui: &mut Ui, c: &Ctx, buttons: &[DialogButton]) -> Option<DialogAction> {
    let mut action = None;
    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        for b in buttons.iter().rev() {
            let opts = button_opts(b).mask(c.t().surface);
            if widgets::button(ui, c, &cap(&b.text), None, opts).clicked() {
                action = Some(b.action.clone());
            }
        }
    });
    action
}

fn buttons_column(ui: &mut Ui, c: &Ctx, buttons: &[DialogButton]) -> Option<DialogAction> {
    let mut action = None;
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 8.0;
        for b in buttons {
            let opts = button_opts(b).full().mask(c.t().surface);
            if widgets::button(ui, c, &cap(&b.text), None, opts).clicked() {
                action = Some(b.action.clone());
            }
        }
    });
    action
}

fn picker_dialog(app: &mut App, ui: &mut Ui, c: &Ctx, items: &[crate::api::PickerItem], buttons: &[DialogButton], w: f32) -> Option<DialogAction> {
    let th = c.t();
    ui.spacing_mut().item_spacing = Vec2::new(0.0, 12.0);
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 14.0;
        widgets::icon_tile(ui, c, "photo", 44.0, th.accent);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            ui.add_space(2.0);
            widgets::text(ui, cap(&t("dialog.picker.title")), theme::semibold(17.0), th.text);
            widgets::wrapped(ui, &cap(&t("dialog.picker.description.desktop")), theme::regular(13.0), th.text_muted, w - 58.0, 1.45);
        });
    });
    let gap = 8.0;
    let cols = if items.len() <= 2 { 2 } else if items.len() <= 9 { 3 } else { 4 };
    let item_size = ((w - gap * (cols as f32 - 1.0)) / cols as f32).min(170.0);
    egui::ScrollArea::vertical().max_height(420.0).auto_shrink([false, true]).show(ui, |ui| {
        ui.spacing_mut().item_spacing = Vec2::new(gap, gap);
        for row in items.chunks(cols) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for item in row {
                    let thumb_url = item.thumb.clone().unwrap_or_else(|| item.url.clone());
                    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(item_size), Sense::click());
                    ui.painter().rect_filled(rect, cr(12.0), th.inset);
                    match app.thumbs.get(&thumb_url) {
                        Some(Some(bytes)) => {
                            let img = egui::Image::new(egui::ImageSource::Bytes {
                                uri: std::borrow::Cow::Owned(format!("bytes://thumb/{}", short_hash(&thumb_url))),
                                bytes: egui::load::Bytes::Shared(bytes.clone()),
                            })
                            .corner_radius(cr(12.0))
                            .fit_to_exact_size(Vec2::splat(item_size));
                            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
                            child.set_clip_rect(rect);
                            img.paint_at(&mut child, rect);
                        }
                        Some(None) => widgets::spinner_at(ui, c, Rect::from_center_size(rect.center(), Vec2::splat(22.0)), th.text_faint),
                        _ => {
                            let icon = match item.kind.as_str() {
                                "video" => "movie",
                                "gif" => "gif",
                                _ => "photo",
                            };
                            c.icons.image(icon, 28.0, th.text_faint).paint_at(ui, Rect::from_center_size(rect.center(), Vec2::splat(28.0)));
                        }
                    }
                    ui.painter().rect_stroke(rect, cr(12.0), Stroke::new(1.0, if resp.hovered() { th.accent } else { th.border }), egui::StrokeKind::Inside);
                    if item.kind == "video" || item.kind == "gif" {
                        let badge = Rect::from_min_size(rect.min + Vec2::new(8.0, 8.0), Vec2::splat(24.0));
                        ui.painter().rect_filled(badge, cr(7.0), Color32::from_black_alpha(150));
                        let icon = if item.kind == "gif" { "gif" } else { "player-play" };
                        c.icons.image(icon, 14.0, Color32::WHITE).paint_at(ui, Rect::from_center_size(badge.center(), Vec2::splat(14.0)));
                    }
                    if resp.hovered() {
                        ui.painter().rect_filled(rect, cr(12.0), Color32::from_white_alpha(14));
                        let dl = Rect::from_center_size(rect.center(), Vec2::splat(36.0));
                        ui.painter().circle_filled(dl.center(), 18.0, th.accent);
                        c.icons.image("download", 18.0, Color32::WHITE).paint_at(ui, Rect::from_center_size(dl.center(), Vec2::splat(18.0)));
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if resp.clicked() {
                        let name = picker_filename(item);
                        app.download_url(&item.url, &name, None);
                    }
                }
            });
        }
    });
    ui.add_space(4.0);
    buttons_row(ui, c, buttons)
}

fn picker_filename(item: &crate::api::PickerItem) -> String {
    let from_url = crate::app::filename_from_url(&item.url);
    if from_url.contains('.') && from_url.len() < 120 {
        return from_url;
    }
    let ext = match item.kind.as_str() {
        "video" => "mp4",
        "gif" => "gif",
        _ => "jpg",
    };
    format!("{}_{}.{}", item.kind, short_hash(&item.url), ext)
}

pub fn short_hash(s: &str) -> String {
    let mut h: u64 = 1469598103934665603;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    format!("{h:016x}")
}
