//! support page: donation options, share, crypto

use crate::app::App;
use crate::i18n::{t, tf};
use crate::theme::{self, cap};
use crate::ui::page_body;
use crate::ui::widgets::{self, cr, ButtonKind, ButtonOpts, Ctx, InputOpts, SegOption};
use egui::{Align, Align2, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};
use std::time::Instant;

const STRIPE: &str = "https://donate.stripe.com/3cs2cc6ew1Qda4wbII";
const LIBERAPAY: &str = "https://liberapay.com/imput/donate";
const CRYPTO: &[(&str, &str)] = &[
    ("ethereum", "0xDA47A671B2411468E8320916C3e57D2F60FE7197"),
    ("monero", "463y93PsQDTYGVPAHUNcjiYDsxWjn7bL2FS9GYXjetEH5XEoNKB7kCHHQXsuoebbSv8RqGspo61pxhMQQrudDky2AfTGbs3"),
    ("solana", "BWPQpPvSyfauUm1BwmV55qE1vJT56Pc6qHrNFzCmtmFJ"),
    ("litecoin", "ltc1qfdemqtfsj7pgnfmtv7n5agtrh0yzwk2pzgr96y"),
    ("bitcoin", "bc1qeqd27qknt3fwvuzpvv2ne730klggggwcqm43yq"),
    ("ton", "UQBosUGIkvZcV8k02bdm-lRFLXrlr1A_sdO1FnXhAsUOLx1S"),
];
const OTHER: &[(&str, &str)] = &[("boosty", "https://boosty.to/wukko/donate")];
const ONCE: &[(u32, &str)] = &[(5, "cup"), (10, "coin"), (15, "device-laptop"), (30, "users-group"), (50, "box-multiple"), (100, "world-www"), (200, "cpu"), (500, "heart"), (1599, "device-laptop"), (4900, "diamond"), (7398, "device-laptop"), (8629, "world"), (9433, "sun-high")];
const RECURRING: &[(u32, &str)] = &[(5, "cup"), (10, "coin"), (15, "device-laptop"), (30, "users-group"), (50, "box-multiple"), (100, "world-www"), (200, "cpu"), (500, "heart")];

pub fn show(app: &mut App, c: &Ctx, ui: &mut Ui) {
    let th = c.t();
    page_body(ui, "support", 860.0, |ui, w| {
        ui.add_space(8.0);
        // banner
        let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 170.0), Sense::hover());
        widgets::gradient_rounded(ui.painter(), rect, 16.0, widgets::with_alpha(th.accent, 0.35), widgets::with_alpha(th.accent_2, 0.25), th.bg);
        ui.painter().rect_stroke(rect, cr(16.0), Stroke::new(1.0, th.border), StrokeKind::Inside);
        let mut y = rect.top() + 32.0;
        for line in t("donate.banner.title").lines() {
            ui.painter().text(Pos2::new(rect.left() + 28.0, y), Align2::LEFT_TOP, line, theme::bold(26.0), th.text);
            y += 32.0;
        }
        y += 8.0;
        for line in t("donate.banner.subtitle").lines() {
            ui.painter().text(Pos2::new(rect.left() + 28.0, y), Align2::LEFT_TOP, line, theme::regular(14.0), th.text_muted);
            y += 20.0;
        }
        let tile = Rect::from_center_size(Pos2::new(rect.right() - 80.0, rect.center().y), Vec2::splat(64.0));
        widgets::icon_tile_at(ui, c, tile, "heart", th.accent);

        let two = w > 640.0;
        let gap = 12.0;
        let cw = if two { (w - gap) / 2.0 } else { w };
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            ui.allocate_ui_with_layout(Vec2::new(cw, 0.0), egui::Layout::top_down(Align::Min), |ui| options_card(app, c, ui, cw));
            if two {
                ui.allocate_ui_with_layout(Vec2::new(cw, 0.0), egui::Layout::top_down(Align::Min), |ui| {
                    share_card(app, c, ui, cw);
                    ui.add_space(gap);
                    crypto_card(app, c, ui, cw);
                });
            }
        });
        if !two {
            share_card(app, c, ui, w);
            crypto_card(app, c, ui, w);
        }
        widgets::card(ui, c, w, |ui| {
            ui.spacing_mut().item_spacing.y = 10.0;
            for key in ["donate.body.motivation", "donate.body.no_bullshit", "donate.body.keep_going"] {
                widgets::wrapped(ui, &cap(&t(key)), theme::regular(14.0), th.text_muted, w - 40.0, 1.6);
            }
        });
    });
}

fn options_card(app: &mut App, c: &Ctx, ui: &mut Ui, w: f32) {
    let th = c.t();
    widgets::card(ui, c, w, |ui| {
        ui.spacing_mut().item_spacing.y = 12.0;
        let options = [
            SegOption { value: "once", label: cap(&t("donate.card.once")), icon: None },
            SegOption { value: "recurring", label: cap(&t("donate.card.recurring")), icon: None },
        ];
        let current = if app.donate.recurring { "recurring" } else { "once" };
        if let Some(v) = widgets::segmented(ui, c, current, &options, true, 38.0) {
            app.donate.recurring = v == "recurring";
        }
        let list: &[(u32, &str)] = if app.donate.recurring { RECURRING } else { ONCE };
        let gap = 8.0;
        let iw = (ui.available_width() - gap) / 2.0;
        for row in list.chunks(2) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for (amount, icon) in row {
                    let (rect, resp) = ui.allocate_exact_size(Vec2::new(iw, 58.0), Sense::click());
                    let bg = if resp.hovered() { th.surface_hover } else { th.inset };
                    ui.painter().rect_filled(rect, cr(12.0), bg);
                    ui.painter().rect_stroke(rect, cr(12.0), Stroke::new(1.0, if resp.hovered() { th.border_strong } else { th.border }), StrokeKind::Inside);
                    let tile = Rect::from_center_size(Pos2::new(rect.left() + 12.0 + 16.0, rect.center().y), Vec2::splat(32.0));
                    widgets::icon_tile_at(ui, c, tile, icon, th.accent);
                    ui.painter().text(Pos2::new(tile.right() + 10.0, rect.center().y - 9.0), Align2::LEFT_CENTER, format!("${amount}"), theme::semibold(15.0), th.text);
                    let mut job = widgets::wrapped_job(&t(&format!("donate.card.option.{amount}")), theme::regular(11.5), th.text_muted, rect.width() - 70.0, 1.2);
                    job.wrap.max_rows = 1;
                    let galley = ui.painter().layout_job(job);
                    ui.painter().galley(Pos2::new(tile.right() + 10.0, rect.center().y + 3.0), galley, th.text_muted);
                    if resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if resp.clicked() {
                        let url = if app.donate.recurring { LIBERAPAY.to_string() } else { format!("{STRIPE}?__prefilled_amount={}", amount * 100) };
                        let _ = open::that_detached(url);
                    }
                }
            });
        }
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let iw = ui.available_width() - 40.0 - 8.0;
            let r = widgets::text_input(ui, c, "custom-amount", &mut app.donate.custom_amount, iw, InputOpts { placeholder: &cap(&t("donate.card.custom")), icon: Some("coin"), ..Default::default() });
            app.donate.custom_amount.retain(|ch| ch.is_ascii_digit());
            let amount: u32 = app.donate.custom_amount.parse().unwrap_or(0);
            let ok = amount >= 2;
            let b = widgets::icon_button(ui, c, "arrow-right", 40.0, if ok { ButtonKind::Primary } else { ButtonKind::Secondary });
            if (b.clicked() || (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))) && ok {
                let url = if app.donate.recurring { LIBERAPAY.to_string() } else { format!("{STRIPE}?__prefilled_amount={}", amount * 100) };
                let _ = open::that_detached(url);
            }
        });
        let processor = if app.donate.recurring { "liberapay" } else { "stripe" };
        widgets::text(ui, tf("donate.card.processor", &[("value", processor)]), theme::regular(12.0), th.text_faint);
    });
}

fn share_card(app: &mut App, c: &Ctx, ui: &mut Ui, w: f32) {
    let th = c.t();
    widgets::card(ui, c, w, |ui| {
        ui.spacing_mut().item_spacing.y = 12.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            widgets::icon_tile(ui, c, "share-2", 32.0, th.accent_2);
            widgets::text(ui, cap(&t("donate.share.title")), theme::semibold(15.0), th.text);
        });
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let iw = ui.available_width() - 40.0 - 8.0;
            let (rect, _) = ui.allocate_exact_size(Vec2::new(iw, 40.0), Sense::hover());
            ui.painter().rect_filled(rect, cr(10.0), th.inset);
            ui.painter().rect_stroke(rect, cr(10.0), Stroke::new(1.0, th.border), StrokeKind::Inside);
            ui.painter().text(Pos2::new(rect.left() + 12.0, rect.center().y), Align2::LEFT_CENTER, "https://cobalt.tools", theme::mono(13.5), th.text);
            let copied = app.donate.copied.as_ref().map(|(k, at)| k == "share" && at.elapsed().as_millis() < 1500).unwrap_or(false);
            if widgets::icon_button(ui, c, if copied { "check" } else { "copy" }, 40.0, ButtonKind::Soft).clicked() {
                app.copy_text("https://cobalt.tools");
                app.donate.copied = Some(("share".into(), Instant::now()));
            }
        });
    });
}

fn crypto_card(app: &mut App, c: &Ctx, ui: &mut Ui, w: f32) {
    let th = c.t();
    widgets::card(ui, c, w, |ui| {
        ui.spacing_mut().item_spacing.y = 6.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            widgets::icon_tile(ui, c, "diamond", 32.0, th.warning);
            widgets::text(ui, cap(&t("donate.alternative.title")), theme::semibold(15.0), th.text);
        });
        ui.add_space(4.0);
        let all: Vec<(&str, &str, bool)> = CRYPTO.iter().map(|(n, a)| (*n, *a, true)).chain(OTHER.iter().map(|(n, a)| (*n, *a, false))).collect();
        let iw = ui.available_width();
        for (name, address, copy) in all {
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(iw, 44.0), Sense::click());
            if resp.hovered() {
                ui.painter().rect_filled(rect, cr(8.0), th.surface_hover);
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            ui.painter().text(Pos2::new(rect.left() + 8.0, rect.top() + 12.0), Align2::LEFT_CENTER, cap(name), theme::medium(13.0), th.text);
            let mut job = widgets::wrapped_job(address, theme::mono(11.0), th.text_muted, iw - 50.0, 1.2);
            job.wrap.max_rows = 1;
            job.wrap.break_anywhere = true;
            let galley = ui.painter().layout_job(job);
            ui.painter().galley(Pos2::new(rect.left() + 8.0, rect.bottom() - 8.0 - galley.size().y), galley, th.text_muted);
            let copied = app.donate.copied.as_ref().map(|(k, at)| k == name && at.elapsed().as_millis() < 1500).unwrap_or(false);
            let icon = if copied { "check" } else if copy { "copy" } else { "external-link" };
            c.icons.image(icon, 16.0, if copied { th.success } else { th.text_faint }).paint_at(ui, Rect::from_center_size(Pos2::new(rect.right() - 16.0, rect.center().y), Vec2::splat(16.0)));
            if resp.clicked() {
                if copy {
                    app.copy_text(address);
                    app.donate.copied = Some((name.to_string(), Instant::now()));
                } else {
                    let _ = open::that_detached(address);
                }
            }
        }
        let _ = ButtonOpts::ghost();
    });
}
