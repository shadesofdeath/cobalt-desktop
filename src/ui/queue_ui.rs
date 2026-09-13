//! activity: the processing queue as cards (inline on the download page and in the side panel)

use super::widgets::{self, cr, ButtonKind, ButtonOpts, Ctx};
use crate::app::App;
use crate::i18n::t;
use crate::queue::{format_file_size, ItemState, MediaType, QueueItem, WorkerKind};
use crate::theme::{self, cap, ACTIVITY_WIDTH};
use egui::{Align, Align2, Color32, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};

/// top-bar button with a progress ring and item count
pub fn status_button(app: &mut App, c: &Ctx, ui: &mut Ui) {
    let th = c.t();
    let (total, indeterminate) = app.tm.total_progress();
    let items = app.tm.snapshot();
    let count = items.len();
    let active = items.iter().filter(|i| matches!(i.state, ItemState::Running | ItemState::Waiting)).count();
    let size = 36.0;
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    let hovered = resp.hovered();
    let bg = if app.activity_open { th.accent_soft } else if hovered { th.surface_hover } else { th.surface };
    ui.painter().rect_filled(rect, cr(10.0), bg);
    ui.painter().rect_stroke(rect, cr(10.0), Stroke::new(1.0, if app.activity_open { widgets::with_alpha(th.accent, 0.4) } else { th.border }), StrokeKind::Inside);
    let fg = if app.activity_open { th.accent } else { th.text_muted };
    if active > 0 {
        let r = 13.0;
        ui.painter().circle_stroke(rect.center(), r, Stroke::new(2.0, th.inset));
        if indeterminate {
            let time = ui.input(|i| i.time) as f32;
            let start = time * 2.5 % std::f32::consts::TAU;
            widgets::arc(ui.painter(), rect.center(), r, start, start + 1.2, Stroke::new(2.0, th.accent));
            ui.ctx().request_repaint();
        } else {
            let anim = ui.ctx().animate_value_with_time(ui.id().with("ring"), total, 0.2);
            widgets::arc(ui.painter(), rect.center(), r, -std::f32::consts::FRAC_PI_2, -std::f32::consts::FRAC_PI_2 + anim * std::f32::consts::TAU, Stroke::new(2.0, th.accent));
        }
        c.icons.image("arrow-down", 14.0, th.accent).paint_at(ui, Rect::from_center_size(rect.center(), Vec2::splat(14.0)));
    } else {
        c.icons.image("arrow-down", 18.0, fg).paint_at(ui, Rect::from_center_size(rect.center(), Vec2::splat(18.0)));
    }
    if count > 0 {
        let label = count.to_string();
        let galley = ui.painter().layout_no_wrap(label, theme::semibold(10.0), Color32::WHITE);
        let br = Rect::from_center_size(Pos2::new(rect.right() - 3.0, rect.top() + 3.0), Vec2::new((galley.size().x + 8.0).max(16.0), 16.0));
        ui.painter().rect_filled(br, cr(8.0), th.accent);
        ui.painter().galley(Pos2::new(br.center().x - galley.size().x / 2.0, br.center().y - galley.size().y / 2.0), galley, Color32::WHITE);
    }
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if resp.clicked() {
        app.activity_open = !app.activity_open;
    }
}

/// right side panel
pub fn panel(app: &mut App, c: &Ctx, ui: &mut Ui, content_rect: Rect) {
    let th = c.t();
    let width = ACTIVITY_WIDTH.min(content_rect.width() - 40.0);
    let anim = ui.ctx().animate_bool_with_time(ui.id().with("activity-panel"), true, 0.18);
    let x = content_rect.right() - width * anim;
    let rect = Rect::from_min_max(Pos2::new(x, content_rect.top()), Pos2::new(content_rect.right() + 2.0, content_rect.bottom()));
    let items = app.tm.snapshot();

    let area = egui::Area::new(ui.id().with("activity-area")).order(egui::Order::Foreground).fixed_pos(rect.min).interactable(true).show(ui.ctx(), |ui| {
        ui.set_clip_rect(rect);
        let painter = ui.painter();
        painter.rect_filled(rect.translate(Vec2::new(-8.0, 0.0)).with_max_x(rect.left()), 0.0, Color32::TRANSPARENT);
        painter.rect_filled(rect, 0.0, th.surface);
        painter.line_segment([rect.left_top(), rect.left_bottom()], Stroke::new(1.0, th.border));
        // shadow strip
        for i in 0..8 {
            let a = (8 - i) as f32 / 8.0 * 0.18;
            painter.line_segment([Pos2::new(rect.left() - i as f32, rect.top()), Pos2::new(rect.left() - i as f32, rect.bottom())], Stroke::new(1.0, widgets::with_alpha(th.shadow, a)));
        }
        let inner = rect.shrink2(Vec2::new(20.0, 0.0));
        let mut pui = ui.new_child(egui::UiBuilder::new().max_rect(inner).layout(egui::Layout::top_down(Align::Min)));
        pui.spacing_mut().item_spacing = Vec2::new(0.0, 10.0);
        pui.add_space(16.0);
        egui::Sides::new().height(36.0).show(
            &mut pui,
            |ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                widgets::text(ui, cap(&t("home.activity")), theme::semibold(16.0), th.text);
                if !items.is_empty() {
                    widgets::badge(ui, c, &items.len().to_string(), th.accent);
                }
            },
            |ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                if widgets::icon_button(ui, c, "x", 32.0, ButtonKind::Ghost).clicked() {
                    app.activity_open = false;
                }
                if !items.is_empty() && widgets::button(ui, c, &cap(&t("button.clear")), Some("trash"), ButtonOpts::ghost().small()).clicked() {
                    app.tm.clear_queue();
                }
            },
        );
        pui.add_space(4.0);
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(&mut pui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(0.0, 10.0);
            if items.is_empty() {
                widgets::empty_state(ui, c, "arrow-down", &cap(&t("home.activity.empty.title")), &t("home.activity.empty.description"));
            }
            for item in &items {
                activity_card(app, c, ui, item, inner.width());
            }
            ui.add_space(20.0);
        });
    });

    if ui.input(|i| i.pointer.any_pressed()) {
        let pos = ui.input(|i| i.pointer.interact_pos().unwrap_or(Pos2::ZERO));
        let in_topbar_button = pos.y < content_rect.top() + crate::theme::TOPBAR_HEIGHT && pos.x > content_rect.right() - 80.0;
        if !area.response.rect.contains(pos) && !in_topbar_button && app.dialogs.is_empty() {
            app.activity_open = false;
        }
    }
}

fn media_style(m: MediaType, th: &crate::theme::Theme) -> (&'static str, Color32) {
    match m {
        MediaType::File => ("file", th.text_muted),
        MediaType::Video => ("movie", th.accent),
        MediaType::Audio => ("music", th.accent_2),
        MediaType::Image => ("photo", th.success),
    }
}

/// `generateStatusText()` port
fn status_text(item: &QueueItem) -> String {
    match &item.state {
        ItemState::Running => {
            let progress = item.progress();
            let running_workers: Vec<_> = item.pipeline.iter().filter(|w| item.current_tasks.contains_key(&w.worker_id)).collect();
            let mut kinds: Vec<WorkerKind> = Vec::new();
            for w in &running_workers {
                if !kinds.contains(&w.worker) {
                    kinds.push(w.worker);
                }
            }
            let mut total_size: u64 = running_workers.iter().filter_map(|w| item.current_tasks.get(&w.worker_id).and_then(|p| p.as_ref()).map(|p| p.size)).sum();
            if kinds.len() == 1 && kinds[0] == WorkerKind::Fetch {
                total_size += item.pipeline_results.values().map(|(_, s)| s).sum::<u64>();
            }
            let running_text = kinds.iter().map(|k| t(&format!("queue.state.running.{}", k.key()))).collect::<Vec<_>>().join(", ");
            if !running_workers.is_empty() && total_size > 0 {
                return format!("{} · {}% · {}", cap(&running_text), (progress * 100.0).floor() as u32, format_file_size(total_size));
            }
            let first_unstarted = item.pipeline.iter().find(|w| {
                !item.pipeline_results.contains_key(&w.worker_id) && !matches!(item.current_tasks.get(&w.worker_id), Some(Some(_)))
            });
            if let Some(w) = first_unstarted {
                return cap(&t(&format!("queue.state.starting.{}", w.worker.key())));
            }
            cap(&running_text)
        }
        ItemState::Done { size, saved_to, .. } => match saved_to {
            Some(p) => format!("{} · {}", format_file_size(*size), p.parent().map(|d| d.display().to_string()).unwrap_or_default()),
            None => format_file_size(*size),
        },
        ItemState::Error { code } => {
            if item.retrying {
                cap(&t("queue.state.retrying"))
            } else {
                cap(&t(&format!("error.{code}")))
            }
        }
        ItemState::Waiting => cap(&t("queue.state.waiting")),
    }
}

pub fn activity_card(app: &mut App, c: &Ctx, ui: &mut Ui, item: &QueueItem, width: f32) {
    let th = c.t();
    let (icon, color) = media_style(item.media_type, th);
    egui::Frame::new().fill(th.surface).stroke(Stroke::new(1.0, th.border)).corner_radius(cr(14.0)).inner_margin(widgets::margin(14.0, 12.0)).show(ui, |ui| {
        let inner_w = width - 28.0;
        ui.set_width(inner_w);
        ui.spacing_mut().item_spacing = Vec2::new(0.0, 8.0);
        let actions_n = match &item.state {
            ItemState::Done { .. } => 2,
            ItemState::Error { .. } if item.can_retry && !item.retrying => 2,
            _ => 1,
        };
        let actions_w = actions_n as f32 * 30.0 + (actions_n as f32 - 1.0) * 4.0;
        let text_w = inner_w - 36.0 - 12.0 - actions_w - 10.0;
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            let (tile, _) = ui.allocate_exact_size(Vec2::splat(36.0), Sense::hover());
            widgets::icon_tile_at(ui, c, tile, icon, color);
            ui.allocate_ui_with_layout(Vec2::new(text_w, 0.0), egui::Layout::top_down(Align::Min), |ui| {
                ui.spacing_mut().item_spacing.y = 3.0;
                let mut job = widgets::wrapped_job(&item.filename, theme::medium(13.5), th.text, text_w, 1.3);
                job.wrap.max_rows = 2;
                job.wrap.break_anywhere = true;
                widgets::galley_label(ui, job, th.text);
                let status_color = match &item.state {
                    ItemState::Done { .. } => th.success,
                    ItemState::Error { .. } if !item.retrying => th.danger,
                    _ => th.text_muted,
                };
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    let (ir, _) = ui.allocate_exact_size(Vec2::splat(14.0), Sense::hover());
                    match &item.state {
                        ItemState::Done { .. } => c.icons.image("check", 14.0, status_color).paint_at(ui, ir),
                        ItemState::Error { .. } if !item.retrying => c.icons.image("exclamation-circle", 14.0, status_color).paint_at(ui, ir),
                        ItemState::Waiting if !item.retrying => c.icons.image("loader-2", 14.0, th.text_faint).paint_at(ui, ir),
                        _ => widgets::spinner_at(ui, c, ir, th.accent),
                    }
                    let mut job = widgets::wrapped_job(&status_text(item), theme::regular(12.0), status_color, text_w - 19.0, 1.35);
                    job.wrap.max_rows = 3;
                    job.wrap.break_anywhere = true;
                    widgets::galley_label(ui, job, status_color);
                });
            });
            ui.with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                if !item.retrying {
                    let r = widgets::icon_button(ui, c, "x", 30.0, ButtonKind::Ghost).on_hover_text(cap(&t("button.remove")));
                    if r.clicked() {
                        app.tm.remove_item(&item.id);
                    }
                    if let ItemState::Error { .. } = &item.state {
                        if item.can_retry && widgets::icon_button(ui, c, "reload", 30.0, ButtonKind::Ghost).on_hover_text(cap(&t("button.retry"))).clicked() {
                            app.retry_item(&item.id);
                        }
                    }
                }
                if let ItemState::Done { file, saved_to, .. } = &item.state {
                    let (icon, tip) = match saved_to {
                        Some(_) => ("folder-open", t("activity.open_folder")),
                        None => ("download", t("activity.save")),
                    };
                    let r = widgets::icon_button(ui, c, icon, 30.0, ButtonKind::Soft).on_hover_text(cap(&tip));
                    if r.clicked() {
                        match saved_to {
                            Some(p) => crate::app::show_in_folder(p),
                            None => app.open_saving_dialog(None, Some((item.id.clone(), file.clone(), item.filename.clone())), String::new()),
                        }
                    }
                }
            });
        });
        if matches!(item.state, ItemState::Running) {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                let n = item.pipeline.len().max(1) as f32;
                let seg_w = (inner_w - 4.0 * (n - 1.0)) / n;
                for w in &item.pipeline {
                    let pct = item.worker_progress(&w.worker_id) / 100.0;
                    let color = if w.worker == WorkerKind::Fetch { th.accent } else { th.accent_2 };
                    widgets::progress_bar(ui, c, seg_w, pct, color);
                }
            });
        }
    });
    let _ = Align2::LEFT_TOP;
}
