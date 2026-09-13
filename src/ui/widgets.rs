//! design-system widgets: cards, buttons, segmented controls, toggles, inputs, chips, progress.

use crate::theme::{self, Icons, Theme, CARD_RADIUS, CONTROL_RADIUS};
use egui::{
    epaint::CornerRadius, Align, Align2, Color32, FontId, Frame, Margin, Mesh, Pos2, Rect, Response, RichText, Sense,
    Shape, Stroke, StrokeKind, Ui, Vec2,
};

pub struct Ctx<'a> {
    pub theme: &'a Theme,
    pub icons: &'a Icons,
}

impl<'a> Ctx<'a> {
    pub fn t(&self) -> &Theme {
        self.theme
    }
}

pub fn cr(r: f32) -> CornerRadius {
    CornerRadius::same(r.round().clamp(0.0, 255.0) as u8)
}

pub fn margin(x: f32, y: f32) -> Margin {
    Margin::symmetric(x.round() as i8, y.round() as i8)
}

pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_unmultiplied(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()), l(a.a(), b.a()))
}

pub fn with_alpha(c: Color32, a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (a * 255.0) as u8)
}

// ---------- text ----------

pub fn text(ui: &mut Ui, s: impl Into<String>, font: FontId, color: Color32) -> Response {
    ui.label(RichText::new(s.into()).font(font).color(color))
}

pub fn wrapped_job(s: &str, font: FontId, color: Color32, width: f32, line_height: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = width;
    let mut fmt = egui::TextFormat::simple(font.clone(), color);
    fmt.line_height = Some(font.size * line_height);
    job.append(s, 0.0, fmt);
    job
}

/// lays out a job with its own wrap width (ignoring the ui's available width) and paints it
pub fn galley_label(ui: &mut Ui, job: egui::text::LayoutJob, color: Color32) -> Response {
    let galley = ui.painter().layout_job(job);
    let (rect, resp) = ui.allocate_exact_size(galley.size(), Sense::hover());
    ui.painter().galley(rect.min, galley, color);
    resp
}

pub fn wrapped(ui: &mut Ui, s: &str, font: FontId, color: Color32, width: f32, line_height: f32) -> Response {
    galley_label(ui, wrapped_job(s, font, color, width, line_height), color)
}

pub fn page_title(ui: &mut Ui, c: &Ctx, s: &str) -> Response {
    text(ui, s, theme::semibold(20.0), c.t().text)
}
pub fn heading(ui: &mut Ui, c: &Ctx, s: &str) -> Response {
    text(ui, s, theme::semibold(15.0), c.t().text)
}
pub fn muted(ui: &mut Ui, c: &Ctx, s: &str, size: f32) -> Response {
    text(ui, s, theme::regular(size), c.t().text_muted)
}
pub fn muted_wrapped(ui: &mut Ui, c: &Ctx, s: &str, size: f32, width: f32) -> Response {
    wrapped(ui, s, theme::regular(size), c.t().text_muted, width, 1.45)
}
pub fn body_wrapped(ui: &mut Ui, c: &Ctx, s: &str, width: f32) -> Response {
    wrapped(ui, s, theme::regular(14.5), c.t().text, width, 1.65)
}

// ---------- surfaces ----------

pub fn card_frame(t: &Theme) -> Frame {
    Frame::new()
        .fill(t.surface)
        .stroke(Stroke::new(1.0, t.border))
        .corner_radius(cr(CARD_RADIUS))
        .inner_margin(margin(20.0, 18.0))
}

pub fn card<R>(ui: &mut Ui, c: &Ctx, width: f32, add: impl FnOnce(&mut Ui) -> R) -> R {
    card_frame(c.t())
        .show(ui, |ui| {
            ui.set_width(width - 40.0);
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 10.0);
            add(ui)
        })
        .inner
}

pub fn card_plain(ui: &mut Ui, c: &Ctx, width: f32, add: impl FnOnce(&mut Ui)) {
    card_frame(c.t()).inner_margin(margin(0.0, 0.0)).show(ui, |ui| {
        ui.set_width(width);
        add(ui);
    });
}

pub fn divider(ui: &mut Ui, c: &Ctx) {
    let w = ui.available_width();
    let (r, _) = ui.allocate_exact_size(Vec2::new(w, 1.0), Sense::hover());
    ui.painter().rect_filled(r, 0.0, c.t().border);
}

pub fn popup_frame(t: &Theme) -> Frame {
    Frame::new()
        .fill(t.surface)
        .stroke(Stroke::new(1.0, t.border_strong))
        .corner_radius(cr(14.0))
        .inner_margin(margin(8.0, 8.0))
        .shadow(egui::epaint::Shadow { offset: [0, 10], blur: 32, spread: 0, color: t.shadow })
}

/// horizontal gradient inside a rounded rect. `mask` is the color behind the rect
/// (used to hide the square corners of the gradient mesh).
pub fn gradient_rounded(painter: &egui::Painter, rect: Rect, radius: f32, c1: Color32, c2: Color32, mask: Color32) {
    let mut mesh = Mesh::default();
    let idx = mesh.vertices.len() as u32;
    mesh.colored_vertex(rect.left_top(), c1);
    mesh.colored_vertex(rect.right_top(), c2);
    mesh.colored_vertex(rect.right_bottom(), c2);
    mesh.colored_vertex(rect.left_bottom(), c1);
    mesh.add_triangle(idx, idx + 1, idx + 2);
    mesh.add_triangle(idx, idx + 2, idx + 3);
    painter.add(Shape::mesh(mesh));
    if radius > 0.0 {
        painter.rect_stroke(rect, cr(radius), Stroke::new(radius, mask), StrokeKind::Outside);
    }
}

/// soft radial glow
pub fn glow(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32) {
    let mut mesh = Mesh::default();
    let n = 48;
    let cidx = mesh.vertices.len() as u32;
    mesh.colored_vertex(center, color);
    for i in 0..=n {
        let a = i as f32 / n as f32 * std::f32::consts::TAU;
        mesh.colored_vertex(Pos2::new(center.x + radius * a.cos(), center.y + radius * a.sin() * 0.55), Color32::TRANSPARENT);
    }
    for i in 0..n {
        mesh.add_triangle(cidx, cidx + 1 + i as u32, cidx + 2 + i as u32);
    }
    painter.add(Shape::mesh(mesh));
}

// ---------- buttons ----------

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    Primary,
    Secondary,
    Ghost,
    Soft,
    Danger,
}

#[derive(Clone, Copy)]
pub struct ButtonOpts {
    pub kind: ButtonKind,
    pub full_width: bool,
    pub height: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub disabled: bool,
    /// background behind the button (for gradient corner masking)
    pub mask: Option<Color32>,
}

impl ButtonOpts {
    pub fn new(kind: ButtonKind) -> Self {
        Self { kind, full_width: false, height: 38.0, font_size: 14.0, icon_size: 18.0, disabled: false, mask: None }
    }
    pub fn primary() -> Self {
        Self::new(ButtonKind::Primary)
    }
    pub fn secondary() -> Self {
        Self::new(ButtonKind::Secondary)
    }
    pub fn ghost() -> Self {
        Self::new(ButtonKind::Ghost)
    }
    pub fn soft() -> Self {
        Self::new(ButtonKind::Soft)
    }
    pub fn danger() -> Self {
        Self::new(ButtonKind::Danger)
    }
    pub fn full(mut self) -> Self {
        self.full_width = true;
        self
    }
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
    pub fn small(mut self) -> Self {
        self.height = 32.0;
        self.font_size = 13.0;
        self.icon_size = 16.0;
        self
    }
    pub fn large(mut self) -> Self {
        self.height = 46.0;
        self.font_size = 15.0;
        self.icon_size = 20.0;
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn mask(mut self, m: Color32) -> Self {
        self.mask = Some(m);
        self
    }
}

fn button_palette(t: &Theme, kind: ButtonKind, hovered: bool, pressed: bool) -> (Color32, Color32, Option<Color32>) {
    // (bg, fg, border)
    match kind {
        ButtonKind::Primary => {
            let bg = if pressed { lerp_color(t.accent, Color32::BLACK, 0.18) } else if hovered { lerp_color(t.accent, Color32::WHITE, 0.08) } else { t.accent };
            (bg, t.accent_text, None)
        }
        ButtonKind::Secondary => {
            let bg = if pressed { t.surface_active } else if hovered { t.surface_hover } else { t.surface };
            (bg, t.text, Some(t.border_strong))
        }
        ButtonKind::Ghost => {
            let bg = if pressed { t.surface_active } else if hovered { t.surface_hover } else { Color32::TRANSPARENT };
            (bg, t.text_muted, None)
        }
        ButtonKind::Soft => {
            let bg = if pressed || hovered { with_alpha(t.accent, 0.26) } else { t.accent_soft };
            (bg, t.accent, None)
        }
        ButtonKind::Danger => {
            let bg = if pressed { lerp_color(t.danger, Color32::BLACK, 0.18) } else if hovered { lerp_color(t.danger, Color32::WHITE, 0.08) } else { t.danger };
            (bg, Color32::WHITE, None)
        }
    }
}

pub fn button(ui: &mut Ui, c: &Ctx, label: &str, icon: Option<&str>, o: ButtonOpts) -> Response {
    let t = c.t();
    let font = theme::medium(o.font_size);
    let galley = if label.is_empty() { None } else { Some(ui.painter().layout_no_wrap(label.to_string(), font, Color32::PLACEHOLDER)) };
    let text_w = galley.as_ref().map(|g| g.size().x).unwrap_or(0.0);
    let gap = if icon.is_some() && !label.is_empty() { 8.0 } else { 0.0 };
    let content_w = text_w + gap + if icon.is_some() { o.icon_size } else { 0.0 };
    let pad_x = if label.is_empty() { (o.height - o.icon_size) / 2.0 } else { 16.0 };
    let mut size = Vec2::new(content_w + pad_x * 2.0, o.height);
    if o.full_width {
        size.x = ui.available_width();
    }
    let (rect, resp) = ui.allocate_exact_size(size, if o.disabled { Sense::hover() } else { Sense::click() });
    if !ui.is_rect_visible(rect) {
        return resp;
    }
    let hovered = resp.hovered() && !o.disabled;
    let pressed = resp.is_pointer_button_down_on() && !o.disabled;
    let (bg, fg, border) = button_palette(t, o.kind, hovered, pressed);
    let painter = ui.painter();
    if o.kind == ButtonKind::Primary && !o.disabled {
        let mask = o.mask.unwrap_or(t.surface);
        let c2 = lerp_color(t.accent, t.accent_2, 0.55);
        let (a, b) = if pressed { (lerp_color(bg, Color32::BLACK, 0.1), lerp_color(c2, Color32::BLACK, 0.1)) } else { (bg, c2) };
        gradient_rounded(painter, rect, CONTROL_RADIUS, a, b, mask);
    } else {
        painter.rect_filled(rect, cr(CONTROL_RADIUS), if o.disabled { t.inset } else { bg });
    }
    if let Some(b) = border {
        painter.rect_stroke(rect, cr(CONTROL_RADIUS), Stroke::new(1.0, b), StrokeKind::Inside);
    }
    let fg = if o.disabled { t.text_faint } else { fg };
    let mut x = rect.center().x - content_w / 2.0;
    if let Some(icon) = icon {
        let ir = Rect::from_center_size(Pos2::new(x + o.icon_size / 2.0, rect.center().y), Vec2::splat(o.icon_size));
        c.icons.image(icon, o.icon_size, fg).paint_at(ui, ir);
        x += o.icon_size + gap;
    }
    if let Some(g) = galley {
        ui.painter().galley(Pos2::new(x, rect.center().y - g.size().y / 2.0), g, fg);
    }
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

pub fn icon_button(ui: &mut Ui, c: &Ctx, icon: &str, size: f32, kind: ButtonKind) -> Response {
    let t = c.t();
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    let hovered = resp.hovered();
    let pressed = resp.is_pointer_button_down_on();
    let (bg, fg, border) = button_palette(t, kind, hovered, pressed);
    ui.painter().rect_filled(rect, cr(CONTROL_RADIUS), bg);
    if let Some(b) = border {
        ui.painter().rect_stroke(rect, cr(CONTROL_RADIUS), Stroke::new(1.0, b), StrokeKind::Inside);
    }
    let icon_size = (size * 0.5).round();
    c.icons.image(icon, icon_size, fg).paint_at(ui, Rect::from_center_size(rect.center(), Vec2::splat(icon_size)));
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

// ---------- segmented control ----------

pub struct SegOption<'a> {
    pub value: &'a str,
    pub label: String,
    pub icon: Option<&'a str>,
}

pub fn segmented(ui: &mut Ui, c: &Ctx, current: &str, options: &[SegOption], full_width: bool, height: f32) -> Option<String> {
    let t = c.t();
    let font = theme::medium(13.5);
    let pad = 4.0;
    let inner_h = height - pad * 2.0;
    let mut widths: Vec<f32> = options
        .iter()
        .map(|o| {
            let tw = if o.label.is_empty() { 0.0 } else { ui.painter().layout_no_wrap(o.label.clone(), font.clone(), Color32::WHITE).size().x };
            let iw = if o.icon.is_some() { 16.0 + if o.label.is_empty() { 0.0 } else { 6.0 } } else { 0.0 };
            tw + iw + 28.0
        })
        .collect();
    let natural: f32 = widths.iter().sum::<f32>() + pad * 2.0;
    let outer_w = if full_width { ui.available_width() } else { natural };
    if full_width {
        let each = (outer_w - pad * 2.0) / options.len().max(1) as f32;
        for w in widths.iter_mut() {
            *w = each;
        }
    }
    let (rect, _) = ui.allocate_exact_size(Vec2::new(outer_w, height), Sense::hover());
    ui.painter().rect_filled(rect, cr(CONTROL_RADIUS), t.inset);
    ui.painter().rect_stroke(rect, cr(CONTROL_RADIUS), Stroke::new(1.0, t.border), StrokeKind::Inside);
    let mut selected = None;
    let mut x = rect.left() + pad;
    for (i, o) in options.iter().enumerate() {
        let brect = Rect::from_min_size(Pos2::new(x, rect.top() + pad), Vec2::new(widths[i], inner_h));
        let resp = ui.interact(brect, ui.id().with(("seg", o.value, i)), Sense::click());
        let active = o.value == current;
        let hovered = resp.hovered();
        let (bg, fg) = if active {
            (t.surface_active, t.text)
        } else if hovered {
            (with_alpha(t.surface_active, 0.5), t.text)
        } else {
            (Color32::TRANSPARENT, t.text_muted)
        };
        if active {
            ui.painter().rect_filled(brect, cr(CONTROL_RADIUS - 3.0), bg);
            ui.painter().rect_stroke(brect, cr(CONTROL_RADIUS - 3.0), Stroke::new(1.0, t.border_strong), StrokeKind::Inside);
        } else if hovered {
            ui.painter().rect_filled(brect, cr(CONTROL_RADIUS - 3.0), bg);
        }
        let fg = if active { t.accent } else { fg };
        let tw = if o.label.is_empty() { 0.0 } else { ui.painter().layout_no_wrap(o.label.clone(), font.clone(), fg).size().x };
        let iw = if o.icon.is_some() { 16.0 } else { 0.0 };
        let g = if o.icon.is_some() && !o.label.is_empty() { 6.0 } else { 0.0 };
        let mut cx = brect.center().x - (tw + iw + g) / 2.0;
        if let Some(icon) = o.icon {
            c.icons.image(icon, 16.0, fg).paint_at(ui, Rect::from_center_size(Pos2::new(cx + 8.0, brect.center().y), Vec2::splat(16.0)));
            cx += 16.0 + g;
        }
        if !o.label.is_empty() {
            ui.painter().text(Pos2::new(cx, brect.center().y), Align2::LEFT_CENTER, &o.label, font.clone(), fg);
        }
        if hovered {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if resp.clicked() && !active {
            selected = Some(o.value.to_string());
        }
        x += widths[i];
    }
    selected
}

// ---------- toggle ----------

pub fn toggle(ui: &mut Ui, c: &Ctx, value: &mut bool, disabled: bool) -> bool {
    let t = c.t();
    let size = Vec2::new(44.0, 24.0);
    let (rect, resp) = ui.allocate_exact_size(size, if disabled { Sense::hover() } else { Sense::click() });
    let mut changed = false;
    if resp.clicked() && !disabled {
        *value = !*value;
        changed = true;
    }
    let anim = ui.ctx().animate_bool_with_time(resp.id.with("toggle"), *value, 0.15);
    let track = lerp_color(t.border_strong, t.accent, anim);
    ui.painter().rect_filled(rect, cr(12.0), if disabled { t.inset } else { track });
    let kx = egui::lerp((rect.left() + 12.0)..=(rect.right() - 12.0), anim);
    ui.painter().circle_filled(Pos2::new(kx, rect.center().y), 9.0, if disabled { t.text_faint } else { lerp_color(t.text_muted, Color32::WHITE, anim) });
    if resp.hovered() && !disabled {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    changed
}

// ---------- settings rows ----------

/// a settings row: title + description on the left, a control on the right
pub fn setting_row(ui: &mut Ui, c: &Ctx, title: &str, description: &str, control_w: f32, control: impl FnOnce(&mut Ui)) {
    let t = c.t();
    let total_w = ui.available_width();
    let text_w = (total_w - control_w - 24.0).max(120.0);
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = 24.0;
        ui.allocate_ui_with_layout(Vec2::new(text_w, 0.0), egui::Layout::top_down(Align::Min), |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            wrapped(ui, title, theme::medium(14.0), t.text, text_w, 1.3);
            if !description.is_empty() {
                wrapped(ui, description, theme::regular(12.5), t.text_muted, text_w, 1.45);
            }
        });
        ui.with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
            ui.allocate_ui_with_layout(Vec2::new(control_w, 0.0), egui::Layout::top_down(Align::Max), |ui| {
                control(ui);
            });
        });
    });
}

/// a settings row whose control sits below the text (for wide segmented controls)
pub fn setting_block(ui: &mut Ui, c: &Ctx, title: &str, description: &str, control: impl FnOnce(&mut Ui)) {
    let t = c.t();
    let w = ui.available_width();
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 4.0;
        wrapped(ui, title, theme::medium(14.0), t.text, w, 1.3);
        if !description.is_empty() {
            wrapped(ui, description, theme::regular(12.5), t.text_muted, w, 1.45);
        }
        ui.add_space(6.0);
        control(ui);
    });
}

pub fn toggle_row(ui: &mut Ui, c: &Ctx, title: &str, description: &str, value: &mut bool, disabled: bool) -> bool {
    let mut changed = false;
    setting_row(ui, c, title, description, 44.0, |ui| {
        changed = toggle(ui, c, value, disabled);
    });
    changed
}

// ---------- dropdown ----------

pub fn dropdown_popup_id(id: &str) -> egui::Id {
    egui::Id::new(("dropdown", id))
}

pub fn dropdown(ui: &mut Ui, c: &Ctx, id: &str, items: &[(String, String)], current: &mut String, width: f32, disabled: bool) -> bool {
    let t = c.t();
    let mut changed = false;
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 36.0), if disabled { Sense::hover() } else { Sense::click() });
    let hovered = resp.hovered() && !disabled;
    let bg = if hovered { t.surface_hover } else { t.inset };
    ui.painter().rect_filled(rect, cr(CONTROL_RADIUS), bg);
    ui.painter().rect_stroke(rect, cr(CONTROL_RADIUS), Stroke::new(1.0, if hovered { t.border_strong } else { t.border }), StrokeKind::Inside);
    let label = items.iter().find(|(v, _)| v == current).map(|(_, l)| l.clone()).unwrap_or_else(|| current.clone());
    let fg = if disabled { t.text_faint } else { t.text };
    let mut job = wrapped_job(&label, theme::medium(13.5), fg, width - 44.0, 1.0);
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    let galley = ui.painter().layout_job(job);
    ui.painter().galley(Pos2::new(rect.left() + 12.0, rect.center().y - galley.size().y / 2.0), galley, fg);
    c.icons.image("selector", 16.0, t.text_muted).paint_at(ui, Rect::from_center_size(Pos2::new(rect.right() - 18.0, rect.center().y), Vec2::splat(16.0)));
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let popup_id = dropdown_popup_id(id);
    if resp.clicked() && !disabled {
        egui::Popup::toggle_id(ui.ctx(), popup_id);
    }
    egui::Popup::from_toggle_button_response(&resp)
        .id(popup_id)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClick)
        .align(egui::RectAlign::BOTTOM_END)
        .gap(6.0)
        .width(width.max(260.0))
        .frame(popup_frame(t))
        .show(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);
            egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                for (value, label) in items {
                    let active = value == current;
                    let r = menu_item(ui, c, label, active);
                    if r.clicked() {
                        *current = value.clone();
                        changed = true;
                    }
                }
            });
        });
    changed
}

pub fn menu_item(ui: &mut Ui, c: &Ctx, label: &str, active: bool) -> Response {
    let t = c.t();
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 34.0), Sense::click());
    let bg = if active { t.accent_soft } else if resp.hovered() { t.surface_hover } else { Color32::TRANSPARENT };
    ui.painter().rect_filled(rect, cr(8.0), bg);
    let fg = if active { t.accent } else { t.text };
    ui.painter().text(Pos2::new(rect.left() + 12.0, rect.center().y), Align2::LEFT_CENTER, label, theme::medium(13.5), fg);
    if active {
        c.icons.image("check", 16.0, t.accent).paint_at(ui, Rect::from_center_size(Pos2::new(rect.right() - 18.0, rect.center().y), Vec2::splat(16.0)));
    }
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

// ---------- inputs ----------

pub struct InputOpts<'a> {
    pub placeholder: &'a str,
    pub icon: Option<&'a str>,
    pub mono: bool,
    pub sensitive: bool,
    pub height: f32,
    pub font_size: f32,
}

impl<'a> Default for InputOpts<'a> {
    fn default() -> Self {
        Self { placeholder: "", icon: None, mono: false, sensitive: false, height: 40.0, font_size: 14.0 }
    }
}

pub fn text_input(ui: &mut Ui, c: &Ctx, id: &str, value: &mut String, width: f32, o: InputOpts) -> Response {
    let t = c.t();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, o.height), Sense::hover());
    let id = ui.make_persistent_id(id);
    let focused = ui.ctx().memory(|m| m.has_focus(id));
    ui.painter().rect_filled(rect, cr(CONTROL_RADIUS), t.inset);
    let border = if focused { t.accent } else { t.border };
    ui.painter().rect_stroke(rect, cr(CONTROL_RADIUS), Stroke::new(if focused { 1.5 } else { 1.0 }, border), StrokeKind::Inside);
    let mut left = rect.left() + 12.0;
    if let Some(icon) = o.icon {
        c.icons.image(icon, 18.0, if focused { t.accent } else { t.text_faint }).paint_at(ui, Rect::from_center_size(Pos2::new(left + 9.0, rect.center().y), Vec2::splat(18.0)));
        left += 18.0 + 10.0;
    }
    let inner = Rect::from_min_max(Pos2::new(left, rect.top()), Pos2::new(rect.right() - 12.0, rect.bottom()));
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner).layout(egui::Layout::left_to_right(Align::Center)));
    let font = if o.mono { theme::mono(o.font_size) } else { theme::regular(o.font_size) };
    let edit = egui::TextEdit::singleline(value)
        .id(id)
        .frame(Frame::NONE)
        .hint_text(RichText::new(o.placeholder).color(t.text_faint).font(font.clone()))
        .font(font)
        .text_color(t.text)
        .password(o.sensitive)
        .vertical_align(Align::Center)
        .desired_width(inner.width());
    edit.show(&mut child).response.response
}

// ---------- chips, badges, progress ----------

pub fn chip(ui: &mut Ui, c: &Ctx, label: &str, active: bool) -> Response {
    let t = c.t();
    let galley = ui.painter().layout_no_wrap(label.to_string(), theme::medium(12.5), Color32::PLACEHOLDER);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(galley.size().x + 20.0, 28.0), Sense::click());
    let (bg, fg, border) = if active {
        (t.accent_soft, t.accent, with_alpha(t.accent, 0.4))
    } else if resp.hovered() {
        (t.surface_hover, t.text, t.border_strong)
    } else {
        (t.surface, t.text_muted, t.border)
    };
    ui.painter().rect_filled(rect, cr(14.0), bg);
    ui.painter().rect_stroke(rect, cr(14.0), Stroke::new(1.0, border), StrokeKind::Inside);
    ui.painter().galley(Pos2::new(rect.center().x - galley.size().x / 2.0, rect.center().y - galley.size().y / 2.0), galley, fg);
    resp
}

pub fn badge(ui: &mut Ui, c: &Ctx, label: &str, color: Color32) {
    let galley = ui.painter().layout_no_wrap(label.to_string(), theme::semibold(11.0), color);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(galley.size().x + 14.0, 20.0), Sense::hover());
    ui.painter().rect_filled(rect, cr(6.0), with_alpha(color, if c.t().dark { 0.16 } else { 0.12 }));
    ui.painter().galley(Pos2::new(rect.center().x - galley.size().x / 2.0, rect.center().y - galley.size().y / 2.0), galley, color);
}

pub fn progress_bar(ui: &mut Ui, c: &Ctx, width: f32, pct: f32, color: Color32) {
    let t = c.t();
    let (r, _) = ui.allocate_exact_size(Vec2::new(width, 6.0), Sense::hover());
    ui.painter().rect_filled(r, cr(3.0), t.inset);
    let fill = Rect::from_min_size(r.min, Vec2::new((r.width() * pct.clamp(0.0, 1.0)).max(if pct > 0.0 { 6.0 } else { 0.0 }), r.height()));
    if pct > 0.0 {
        ui.painter().rect_filled(fill, cr(3.0), color);
    }
}

pub fn arc(painter: &egui::Painter, center: Pos2, r: f32, from: f32, to: f32, stroke: Stroke) {
    let n = 48;
    let pts: Vec<Pos2> = (0..=n)
        .map(|i| {
            let a = from + (to - from) * i as f32 / n as f32;
            Pos2::new(center.x + r * a.cos(), center.y + r * a.sin())
        })
        .collect();
    painter.add(Shape::line(pts, stroke));
}

pub fn spinner_at(ui: &mut Ui, c: &Ctx, rect: Rect, color: Color32) {
    let angle = (ui.input(|i| i.time) as f32 * std::f32::consts::TAU * 0.9) % std::f32::consts::TAU;
    c.icons.image("loader-2", rect.width(), color).rotate(angle, Vec2::splat(0.5)).paint_at(ui, rect);
    ui.ctx().request_repaint();
}

pub fn spinner(ui: &mut Ui, c: &Ctx, size: f32, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    spinner_at(ui, c, rect, color);
}

/// info / warning banner
pub fn banner(ui: &mut Ui, c: &Ctx, icon: &str, color: Color32, soft: Color32, text: &str) {
    let w = ui.available_width();
    Frame::new()
        .fill(soft)
        .stroke(Stroke::new(1.0, with_alpha(color, 0.35)))
        .corner_radius(cr(12.0))
        .inner_margin(margin(14.0, 12.0))
        .show(ui, |ui| {
            ui.set_width(w - 28.0);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                let (r, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
                c.icons.image(icon, 18.0, color).paint_at(ui, r);
                wrapped(ui, text, theme::regular(13.0), c.t().text, w - 28.0 - 28.0, 1.45);
            });
        });
}

pub fn empty_state(ui: &mut Ui, c: &Ctx, icon: &str, title: &str, description: &str) {
    let t = c.t();
    ui.vertical_centered(|ui| {
        ui.add_space(22.0);
        let (r, _) = ui.allocate_exact_size(Vec2::splat(52.0), Sense::hover());
        ui.painter().circle_filled(r.center(), 26.0, t.accent_soft);
        c.icons.image(icon, 24.0, t.accent).paint_at(ui, Rect::from_center_size(r.center(), Vec2::splat(24.0)));
        ui.add_space(12.0);
        text(ui, title, theme::semibold(14.5), t.text);
        ui.add_space(4.0);
        let w = (ui.available_width() - 40.0).max(100.0);
        let mut job = wrapped_job(description, theme::regular(13.0), t.text_muted, w, 1.45);
        job.halign = Align::Center;
        ui.label(job);
        ui.add_space(22.0);
    });
}

/// icon in a rounded tinted square
pub fn icon_tile(ui: &mut Ui, c: &Ctx, icon: &str, size: f32, color: Color32) {
    let (r, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    icon_tile_at(ui, c, r, icon, color);
}

pub fn icon_tile_at(ui: &mut Ui, c: &Ctx, r: Rect, icon: &str, color: Color32) {
    ui.painter().rect_filled(r, cr(r.width() * 0.3), with_alpha(color, if c.t().dark { 0.16 } else { 0.12 }));
    let isz = (r.width() * 0.55).round();
    c.icons.image(icon, isz, color).paint_at(ui, Rect::from_center_size(r.center(), Vec2::splat(isz)));
}

pub fn link(ui: &mut Ui, c: &Ctx, label: &str, url: &str, font: FontId) -> Response {
    let r = ui.add(egui::Label::new(RichText::new(label).font(font).color(c.t().accent)).sense(Sense::click()));
    if r.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if r.clicked() {
        let _ = open::that_detached(url);
    }
    r
}

pub fn wrap_row(ui: &mut Ui, gap: f32, add: impl FnOnce(&mut Ui)) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(gap, gap);
        add(ui);
    });
}
