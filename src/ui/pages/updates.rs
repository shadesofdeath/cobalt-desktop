//! updates: version list + changelog

use crate::app::App;
use crate::changelogs::CHANGELOGS;
use crate::i18n::t;
use crate::theme::{self, cap, PAGE_PADDING};
use crate::ui::markdown;
use crate::ui::widgets::{self, cr, Ctx};
use egui::{Align, Align2, Color32, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};

pub fn show(app: &mut App, c: &Ctx, ui: &mut Ui) {
    let th = c.t();
    let logs = &*CHANGELOGS;
    if logs.is_empty() {
        return;
    }
    app.updates_index = app.updates_index.min(logs.len() - 1);
    let (left, right) = ui.input(|i| (i.key_pressed(egui::Key::ArrowLeft), i.key_pressed(egui::Key::ArrowRight)));
    if left && app.updates_index > 0 && app.dialogs.is_empty() {
        app.updates_index -= 1;
    }
    if right && app.updates_index + 1 < logs.len() && app.dialogs.is_empty() {
        app.updates_index += 1;
    }

    let full = ui.max_rect();
    let rail_w = 150.0;
    let rail = Rect::from_min_max(Pos2::new(full.left() + PAGE_PADDING, full.top()), Pos2::new(full.left() + PAGE_PADDING + rail_w, full.bottom()));
    let content = Rect::from_min_max(Pos2::new(rail.right() + 20.0, full.top()), full.max);

    let mut clicked = None;
    let mut rui = ui.new_child(egui::UiBuilder::new().max_rect(rail).layout(egui::Layout::top_down(Align::Min)));
    rui.set_clip_rect(rail);
    egui::ScrollArea::vertical().id_salt("versions").scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(&mut rui, |ui| {
        ui.set_width(rail_w);
        ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add_space(12.0);
            widgets::text(ui, t("updates.versions").to_uppercase(), theme::semibold(10.5), th.text_faint);
        });
        ui.add_space(4.0);
        for (i, log) in logs.iter().enumerate() {
            let active = i == app.updates_index;
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(rail_w, 34.0), Sense::click());
            let bg = if active { th.surface } else if resp.hovered() { th.surface_hover } else { Color32::TRANSPARENT };
            ui.painter().rect_filled(rect, cr(10.0), bg);
            if active {
                ui.painter().rect_stroke(rect, cr(10.0), Stroke::new(1.0, th.border), StrokeKind::Inside);
            }
            ui.painter().text(Pos2::new(rect.left() + 12.0, rect.center().y), Align2::LEFT_CENTER, format!("v{}", log.version), theme::mono_medium(13.0), if active { th.text } else { th.text_muted });
            if i == 0 {
                let galley = ui.painter().layout_no_wrap("LATEST".into(), theme::semibold(9.0), th.success);
                let br = Rect::from_center_size(Pos2::new(rect.right() - 12.0 - galley.size().x / 2.0 - 5.0, rect.center().y), Vec2::new(galley.size().x + 10.0, 16.0));
                ui.painter().rect_filled(br, cr(4.0), th.success_soft);
                ui.painter().galley(Pos2::new(br.center().x - galley.size().x / 2.0, br.center().y - galley.size().y / 2.0), galley, th.success);
            }
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if resp.clicked() {
                clicked = Some(i);
            }
        }
        ui.add_space(20.0);
    });
    if let Some(i) = clicked {
        app.updates_index = i;
    }

    let log = &logs[app.updates_index];
    let mut cui = ui.new_child(egui::UiBuilder::new().max_rect(content).layout(egui::Layout::top_down(Align::Min)));
    cui.set_clip_rect(content);
    egui::ScrollArea::vertical().id_salt(("changelog", &log.version)).auto_shrink([false, false]).show(&mut cui, |ui| {
        let w = (ui.available_width() - PAGE_PADDING - 8.0).min(760.0).max(320.0);
        ui.allocate_ui_with_layout(Vec2::new(w, 0.0), egui::Layout::top_down(Align::Min), |ui| {
            ui.add_space(8.0);
            widgets::card(ui, c, w, |ui| {
                ui.spacing_mut().item_spacing.y = 10.0;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    widgets::badge(ui, c, &format!("v{}", log.version), th.accent);
                    widgets::text(ui, &log.date, theme::regular(12.5), th.text_muted);
                });
                widgets::wrapped(ui, &cap(&log.title), theme::bold(24.0), th.text, w - 40.0, 1.2);
                if !log.banner_alt.is_empty() {
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(w - 40.0, 96.0), Sense::hover());
                    widgets::gradient_rounded(ui.painter(), rect, 12.0, widgets::with_alpha(th.accent, 0.25), widgets::with_alpha(th.accent_2, 0.18), th.surface);
                    let mut job = widgets::wrapped_job(&log.banner_alt, theme::regular(12.0), th.text_muted, w - 40.0 - 32.0, 1.4);
                    job.wrap.max_rows = 4;
                    let galley = ui.painter().layout_job(job);
                    ui.painter().galley(Pos2::new(rect.left() + 16.0, rect.center().y - galley.size().y / 2.0), galley, th.text_muted);
                }
                widgets::divider(ui, c);
                markdown::render(ui, c, &log.body);
            });
            ui.add_space(40.0);
        });
    });
}
