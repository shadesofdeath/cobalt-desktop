//! app shell: sidebar navigation, top bar, activity panel, toasts, dialogs

pub mod dialogs;
pub mod markdown;
pub mod pages;
pub mod queue_ui;
pub mod widgets;

use crate::app::{App, Page, SettingsPage, ToastKind};
use crate::i18n::t;
use crate::theme::{self, cap, Theme, ACTIVITY_WIDTH, PAGE_PADDING, SIDEBAR_WIDTH, TOPBAR_HEIGHT};
use egui::{Align, Align2, Color32, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};
use widgets::{cr, Ctx};

pub fn show(app: &mut App, root: &mut Ui) {
    let ctx = &root.ctx().clone();
    let system_dark = ctx.system_theme().map(|t| t == egui::Theme::Dark).unwrap_or(app.theme.dark);
    app.theme = Theme::for_setting(&app.settings.appearance.theme, system_dark);
    ctx.set_theme(if app.theme.dark { egui::Theme::Dark } else { egui::Theme::Light });
    let theme = app.theme.clone();
    let icons = app.icons.clone();
    let c = Ctx { theme: &theme, icons: &icons };
    app.tick_toasts();

    ctx.all_styles_mut(|s| {
        s.visuals.selection.bg_fill = widgets::with_alpha(theme.accent, 0.35);
        s.visuals.selection.stroke = Stroke::new(1.0, theme.accent);
        s.visuals.text_cursor.stroke = Stroke::new(2.0, theme.accent);
        s.visuals.override_text_color = Some(theme.text);
        s.visuals.extreme_bg_color = Color32::TRANSPARENT;
        s.visuals.panel_fill = theme.bg;
        s.visuals.window_fill = theme.surface;
        s.spacing.item_spacing = Vec2::new(8.0, 8.0);
        s.spacing.scroll.bar_width = 6.0;
        s.spacing.scroll.floating = true;
        s.spacing.scroll.foreground_color = false;
        s.spacing.scroll.bar_inner_margin = 3.0;
        s.visuals.widgets.inactive.bg_fill = theme.border_strong;
        s.visuals.widgets.hovered.bg_fill = theme.text_faint;
        s.visuals.widgets.active.bg_fill = theme.text_muted;
        s.animation_time = if app.settings.accessibility.reduce_motion { 0.0 } else { 0.12 };
    });

    let ui = root;
    let full = ui.max_rect();
    ui.painter().rect_filled(full, 0.0, theme.bg);
    let sidebar_rect = Rect::from_min_max(full.min, Pos2::new(full.left() + SIDEBAR_WIDTH, full.bottom()));
    let content_rect = Rect::from_min_max(Pos2::new(full.left() + SIDEBAR_WIDTH, full.top()), full.max);

    // sidebar
    ui.painter().rect_filled(sidebar_rect, 0.0, theme.bg_elevated);
    ui.painter().line_segment([sidebar_rect.right_top(), sidebar_rect.right_bottom()], Stroke::new(1.0, theme.border));
    let mut sb = ui.new_child(egui::UiBuilder::new().max_rect(sidebar_rect.shrink2(Vec2::new(12.0, 0.0))).layout(egui::Layout::top_down(Align::Min)));
    sidebar(app, &c, &mut sb, sidebar_rect);

    // top bar
    let top_rect = Rect::from_min_size(content_rect.min, Vec2::new(content_rect.width(), TOPBAR_HEIGHT));
    let mut tb = ui.new_child(egui::UiBuilder::new().max_rect(top_rect.shrink2(Vec2::new(PAGE_PADDING, 0.0))).layout(egui::Layout::left_to_right(Align::Center)));
    topbar(app, &c, &mut tb);

    // page
    let page_rect = Rect::from_min_max(Pos2::new(content_rect.left(), top_rect.bottom()), content_rect.max);
    let mut pg = ui.new_child(egui::UiBuilder::new().max_rect(page_rect).layout(egui::Layout::top_down(Align::Min)));
    pg.set_clip_rect(page_rect);
    match app.page {
        Page::Save => pages::save::show(app, &c, &mut pg),
        Page::Remux => pages::remux::show(app, &c, &mut pg),
        Page::Settings => pages::settings::show(app, &c, &mut pg),
        Page::Donate => pages::donate::show(app, &c, &mut pg),
        Page::Updates => pages::updates::show(app, &c, &mut pg),
        Page::About => pages::about::show(app, &c, &mut pg),
    }

    if app.activity_open {
        queue_ui::panel(app, &c, ui, content_rect);
    }
    toasts(app, &c, ui, content_rect);
    dialogs::show(app, &c, ctx);

    if !app.dialogs.is_empty() && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        if app.dialogs.last().map(|d| d.dismissable).unwrap_or(true) {
            app.close_dialog();
        }
    }
}

pub fn page_title_for(app: &App) -> String {
    match app.page {
        Page::Save => cap(&t("nav.download")),
        Page::Remux => cap(&t("tabs.remux")),
        Page::Settings => cap(&t("tabs.settings")),
        Page::Donate => cap(&t("nav.support")),
        Page::Updates => cap(&t("tabs.updates")),
        Page::About => cap(&t("tabs.about")),
    }
}

fn topbar(app: &mut App, c: &Ctx, ui: &mut Ui) {
    let th = c.t();
    widgets::page_title(ui, c, &page_title_for(app));
    if app.page == Page::Remux {
        ui.add_space(8.0);
        widgets::badge(ui, c, &t("general.beta").to_uppercase(), th.accent);
    }
    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        queue_ui::status_button(app, c, ui);
        // instance chip
        instance_chip(app, c, ui);
    });
}

fn instance_chip(app: &mut App, c: &Ctx, ui: &mut Ui) {
    let th = c.t();
    let (color, label) = instance_status(app, c);
    let host = url::Url::parse(&app.settings.api_url()).ok().and_then(|u| u.host_str().map(|h| h.to_string())).unwrap_or_default();
    let text = format!("{host}  ·  {label}");
    let galley = ui.painter().layout_no_wrap(text, theme::medium(12.5), th.text_muted);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(galley.size().x + 34.0, 32.0), Sense::click());
    let bg = if resp.hovered() { th.surface_hover } else { th.surface };
    ui.painter().rect_filled(rect, cr(16.0), bg);
    ui.painter().rect_stroke(rect, cr(16.0), Stroke::new(1.0, th.border), StrokeKind::Inside);
    ui.painter().circle_filled(Pos2::new(rect.left() + 14.0, rect.center().y), 4.0, color);
    ui.painter().galley(Pos2::new(rect.left() + 24.0, rect.center().y - galley.size().y / 2.0), galley, th.text_muted);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if resp.clicked() {
        app.page = Page::Settings;
        app.settings_page = SettingsPage::Instances;
    }
}

pub fn instance_status(app: &App, c: &Ctx) -> (Color32, String) {
    let th = c.t();
    match (&app.server_info, &app.server_info_error, app.server_info_loading) {
        (Some(info), _, _) => {
            if info.info.cobalt.turnstile_sitekey.is_some() && !app.settings.processing.enable_custom_api_key {
                (th.warning, t("instance.turnstile"))
            } else {
                (th.success, t("instance.connected"))
            }
        }
        (None, Some(_), _) => (th.danger, t("instance.offline")),
        _ => (th.text_faint, t("instance.connecting")),
    }
}

fn sidebar(app: &mut App, c: &Ctx, ui: &mut Ui, sidebar_rect: Rect) {
    let th = c.t();
    ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);
    ui.add_space(18.0);

    // brand
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        let (tile, _) = ui.allocate_exact_size(Vec2::splat(34.0), Sense::hover());
        widgets::gradient_rounded(ui.painter(), tile, 10.0, th.accent, th.accent_2, th.bg_elevated);
        c.icons.image("download@bold", 18.0, Color32::WHITE).paint_at(ui, Rect::from_center_size(tile.center(), Vec2::splat(18.0)));
        ui.add_space(10.0);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            widgets::text(ui, "cobalt", theme::semibold(15.0), th.text);
            widgets::text(ui, "desktop", theme::medium(11.0), th.text_faint);
        });
    });
    ui.add_space(22.0);

    let mut clicked: Option<Page> = None;
    section_label(ui, c, &t("nav.section.tools"));
    if nav_item(ui, c, "download", &cap(&t("nav.download")), app.page == Page::Save, false) {
        clicked = Some(Page::Save);
    }
    if !app.settings.appearance.hide_remux_tab && nav_item(ui, c, "repeat", &cap(&t("tabs.remux")), app.page == Page::Remux, true) {
        clicked = Some(Page::Remux);
    }
    ui.add_space(14.0);
    section_label(ui, c, &t("nav.section.more"));
    if nav_item(ui, c, "settings", &cap(&t("tabs.settings")), app.page == Page::Settings, false) {
        clicked = Some(Page::Settings);
    }
    if nav_item(ui, c, "comet", &cap(&t("tabs.updates")), app.page == Page::Updates, false) {
        clicked = Some(Page::Updates);
    }
    if nav_item(ui, c, "info-circle", &cap(&t("tabs.about")), app.page == Page::About, false) {
        clicked = Some(Page::About);
    }
    if nav_item(ui, c, "heart", &cap(&t("nav.support")), app.page == Page::Donate, false) {
        clicked = Some(Page::Donate);
    }
    if let Some(page) = clicked {
        app.page = page;
        app.activity_open = false;
        if page == Page::Save {
            app.focus_input = true;
        }
    }

    // bottom: theme toggle + version
    let bottom = Rect::from_min_max(Pos2::new(sidebar_rect.left() + 12.0, sidebar_rect.bottom() - 60.0), Pos2::new(sidebar_rect.right() - 12.0, sidebar_rect.bottom() - 16.0));
    let mut b = ui.new_child(egui::UiBuilder::new().max_rect(bottom).layout(egui::Layout::left_to_right(Align::Center)));
    b.add_space(8.0);
    let is_dark = th.dark;
    let r = widgets::icon_button(&mut b, c, if is_dark { "sun-high" } else { "moon" }, 34.0, widgets::ButtonKind::Secondary).on_hover_text(cap(&t("theme.toggle")));
    if r.clicked() {
        app.settings.appearance.theme = if is_dark { "light".into() } else { "dark".into() };
        app.settings_changed();
    }
    b.add_space(10.0);
    b.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 1.0;
        widgets::text(ui, format!("v{}", env!("CARGO_PKG_VERSION")), theme::medium(12.0), th.text_muted);
        widgets::text(ui, "cobalt api 11.7", theme::regular(11.0), th.text_faint);
    });
}

fn section_label(ui: &mut Ui, c: &Ctx, s: &str) {
    ui.horizontal(|ui| {
        ui.add_space(12.0);
        widgets::text(ui, s.to_uppercase(), theme::semibold(10.5), c.t().text_faint);
    });
    ui.add_space(4.0);
}

fn nav_item(ui: &mut Ui, c: &Ctx, icon: &str, label: &str, active: bool, beta: bool) -> bool {
    let th = c.t();
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 38.0), Sense::click());
    let hovered = resp.hovered();
    let bg = if active { th.accent_soft } else if hovered { th.surface_hover } else { Color32::TRANSPARENT };
    ui.painter().rect_filled(rect, cr(10.0), bg);
    let fg = if active { th.accent } else if hovered { th.text } else { th.text_muted };
    if active {
        let bar = Rect::from_center_size(Pos2::new(rect.left() + 2.0, rect.center().y), Vec2::new(3.0, 18.0));
        ui.painter().rect_filled(bar, cr(2.0), th.accent);
    }
    c.icons.image(icon, 18.0, fg).paint_at(ui, Rect::from_center_size(Pos2::new(rect.left() + 12.0 + 9.0, rect.center().y), Vec2::splat(18.0)));
    ui.painter().text(Pos2::new(rect.left() + 40.0, rect.center().y), Align2::LEFT_CENTER, label, theme::medium(14.0), if active { th.text } else { fg });
    if beta {
        let galley = ui.painter().layout_no_wrap("BETA".into(), theme::semibold(9.5), th.accent);
        let br = Rect::from_center_size(Pos2::new(rect.right() - 12.0 - galley.size().x / 2.0 - 6.0, rect.center().y), Vec2::new(galley.size().x + 12.0, 18.0));
        ui.painter().rect_filled(br, cr(5.0), th.accent_soft);
        ui.painter().galley(Pos2::new(br.center().x - galley.size().x / 2.0, br.center().y - galley.size().y / 2.0), galley, th.accent);
    }
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

fn toasts(app: &mut App, c: &Ctx, ui: &mut Ui, content_rect: Rect) {
    let th = c.t();
    if app.toasts.is_empty() {
        return;
    }
    let mut y = content_rect.bottom() - 20.0;
    for toast in app.toasts.iter().rev() {
        let age = toast.at.elapsed().as_secs_f32();
        let alpha = if age < 0.15 { age / 0.15 } else if age > 2.8 { ((3.2 - age) / 0.4).clamp(0.0, 1.0) } else { 1.0 };
        let (icon, color) = match toast.kind {
            ToastKind::Info => ("info-circle", th.accent),
            ToastKind::Success => ("check", th.success),
            ToastKind::Error => ("exclamation-circle", th.danger),
        };
        let galley = ui.painter().layout_no_wrap(cap(&toast.text), theme::medium(13.0), th.text);
        let w = (galley.size().x + 58.0).min(content_rect.width() - 40.0);
        let h = 42.0;
        let rect = Rect::from_min_size(Pos2::new(content_rect.right() - 20.0 - w, y - h), Vec2::new(w, h));
        let painter = ui.painter();
        painter.rect_filled(rect.translate(Vec2::new(0.0, 4.0)), cr(12.0), widgets::with_alpha(th.shadow, 0.5 * alpha));
        painter.rect_filled(rect, cr(12.0), widgets::with_alpha(th.surface, alpha));
        painter.rect_stroke(rect, cr(12.0), Stroke::new(1.0, widgets::with_alpha(th.border_strong, alpha)), StrokeKind::Inside);
        c.icons.image(icon, 18.0, widgets::with_alpha(color, alpha)).paint_at(ui, Rect::from_center_size(Pos2::new(rect.left() + 21.0, rect.center().y), Vec2::splat(18.0)));
        ui.painter().galley(Pos2::new(rect.left() + 40.0, rect.center().y - galley.size().y / 2.0), galley, widgets::with_alpha(th.text, alpha));
        y -= h + 8.0;
    }
    ui.ctx().request_repaint();
}

/// scrollable page body with standard padding and a max content width
pub fn page_body(ui: &mut Ui, id: &str, max_w: f32, add: impl FnOnce(&mut Ui, f32)) {
    egui::ScrollArea::vertical()
        .id_salt(("page", id))
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let avail = ui.available_width();
            let w = (avail - PAGE_PADDING * 2.0).min(max_w).max(320.0);
            let left = ((avail - w) / 2.0).max(PAGE_PADDING);
            ui.horizontal_top(|ui| {
                ui.add_space(left - 8.0);
                ui.allocate_ui_with_layout(Vec2::new(w, 0.0), egui::Layout::top_down(Align::Min), |ui| {
                    ui.add_space(6.0);
                    ui.spacing_mut().item_spacing = Vec2::new(12.0, 16.0);
                    add(ui, w);
                    ui.add_space(40.0);
                });
            });
        });
}
