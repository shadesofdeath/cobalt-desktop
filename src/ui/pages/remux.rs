//! remux page: drop zone + explainer cards

use crate::app::App;
use crate::i18n::{t, tf};
use crate::theme::{self, cap};
use crate::ui::page_body;
use crate::ui::widgets::{self, cr, ButtonOpts, Ctx};
use egui::{Align, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};

const FORMATS: &[&str] = &["mp4", "webm", "mp3", "ogg", "opus", "wav", "m4a"];

pub fn show(app: &mut App, c: &Ctx, ui: &mut Ui) {
    let th = c.t();
    let hovering = ui.input(|i| !i.raw.hovered_files.is_empty());
    app.remux_dragging = hovering;
    let dropped: Vec<std::path::PathBuf> = ui.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).collect());
    if !dropped.is_empty() {
        app.remux_files(dropped);
    }

    page_body(ui, "remux", 860.0, |ui, w| {
        ui.add_space(10.0);
        // drop zone
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 240.0), Sense::click());
        let bg = if hovering { th.accent_soft } else if resp.hovered() { th.surface_hover } else { th.surface };
        ui.painter().rect_filled(rect, cr(16.0), bg);
        dashed_rect(ui, rect, 16.0, if hovering { th.accent } else { th.border_strong });
        let tile = Rect::from_center_size(Pos2::new(rect.center().x, rect.top() + 72.0), Vec2::splat(56.0));
        widgets::icon_tile_at(ui, c, tile, "upload", th.accent);
        let title = if hovering { t("receiver.title.drop.multiple") } else { t("receiver.title.multiple") };
        ui.painter().text(Pos2::new(rect.center().x, tile.bottom() + 16.0), egui::Align2::CENTER_TOP, cap(&title), theme::semibold(17.0), th.text);
        let formats = cap(&tf("receiver.accept", &[("formats", &FORMATS.join(", "))]));
        ui.painter().text(Pos2::new(rect.center().x, tile.bottom() + 44.0), egui::Align2::CENTER_TOP, formats, theme::regular(13.0), th.text_muted);
        let brect = Rect::from_center_size(Pos2::new(rect.center().x, rect.bottom() - 40.0), Vec2::new(160.0, 38.0));
        let mut bui = ui.new_child(egui::UiBuilder::new().max_rect(brect).layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight)));
        let open = widgets::button(&mut bui, c, &cap(&t("remux.select_files")), Some("folder"), ButtonOpts::secondary().full());
        if resp.hovered() && !open.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if open.clicked() || (resp.clicked() && !open.hovered()) {
            app.pick_remux_files();
        }

        if app.ffmpeg_path.is_none() {
            widgets::banner(ui, c, "alert-triangle", th.danger, th.danger_soft, &cap(&t("error.ffmpeg.not_found")));
        }

        // explainer cards
        let gap = 12.0;
        let cols = if w > 700.0 { 3 } else { 1 };
        let cw = (w - gap * (cols as f32 - 1.0)) / cols as f32;
        let bullets = [
            ("repeat", "remux.bullet.purpose.title", "remux.bullet.purpose.description", th.accent),
            ("info-circle", "remux.bullet.explainer.title", "remux.bullet.explainer.description", th.accent_2),
            ("devices", "remux.bullet.privacy.title", "remux.bullet.privacy.description", th.success),
        ];
        for row in bullets.chunks(cols) {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for (icon, title, desc, color) in row {
                    ui.allocate_ui_with_layout(Vec2::new(cw, 0.0), egui::Layout::top_down(Align::Min), |ui| {
                        widgets::card(ui, c, cw, |ui| {
                            ui.spacing_mut().item_spacing.y = 10.0;
                            widgets::icon_tile(ui, c, icon, 40.0, *color);
                            widgets::text(ui, cap(&t(title)), theme::semibold(14.5), th.text);
                            widgets::wrapped(ui, &cap(&t(desc)), theme::regular(13.0), th.text_muted, cw - 40.0, 1.5);
                        });
                    });
                }
            });
        }
    });
    let _ = StrokeKind::Inside;
}

fn dashed_rect(ui: &Ui, rect: Rect, radius: f32, color: egui::Color32) {
    let r = radius;
    let mut pts: Vec<Pos2> = Vec::new();
    let corners = [
        (Pos2::new(rect.right() - r, rect.top() + r), -std::f32::consts::FRAC_PI_2),
        (Pos2::new(rect.right() - r, rect.bottom() - r), 0.0),
        (Pos2::new(rect.left() + r, rect.bottom() - r), std::f32::consts::FRAC_PI_2),
        (Pos2::new(rect.left() + r, rect.top() + r), std::f32::consts::PI),
    ];
    for (center, start) in corners {
        for i in 0..=8 {
            let a = start + (i as f32 / 8.0) * std::f32::consts::FRAC_PI_2;
            pts.push(Pos2::new(center.x + r * a.cos(), center.y + r * a.sin()));
        }
    }
    pts.push(pts[0]);
    let stroke = Stroke::new(1.5, color);
    let (dash, gap) = (7.0, 6.0);
    let mut acc = 0.0f32;
    let mut drawing = true;
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let len = a.distance(b);
        let mut pos = 0.0f32;
        while pos < len {
            let remaining: f32 = if drawing { dash - acc } else { gap - acc };
            let step = remaining.min(len - pos);
            let p0 = a + (b - a) * (pos / len);
            let p1 = a + (b - a) * ((pos + step) / len);
            if drawing {
                ui.painter().line_segment([p0, p1], stroke);
            }
            acc += step;
            pos += step;
            if (drawing && acc >= dash) || (!drawing && acc >= gap) {
                drawing = !drawing;
                acc = 0.0;
            }
        }
    }
}
