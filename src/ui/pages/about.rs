//! about: tabs + markdown pages from cobalt's i18n, community cards, desktop notes

use crate::app::{AboutPage, App};
use crate::i18n::t;
use crate::theme::{self, cap};
use crate::ui::markdown;
use crate::ui::page_body;
use crate::ui::widgets::{self, Ctx, SegOption};
use egui::{Align, Sense, Ui, Vec2};

const GITHUB: &str = "https://github.com/imputnet/cobalt";
const DISCORD: &str = "https://discord.gg/pQPt8HBUPu";
const TWITTER: &str = "https://x.com/justusecobalt";
const BLUESKY: &str = "https://bsky.app/profile/cobalt.tools";
const TELEGRAM_RU: &str = "https://t.me/justusecobalt_ru";
const INSTANCE_HOSTING: &str = "https://github.com/imputnet/cobalt/blob/main/docs/run-an-instance.md";
const WEB_LICENSE: &str = "https://github.com/imputnet/cobalt/blob/main/web/LICENSE";
const API_LICENSE: &str = "https://github.com/imputnet/cobalt/blob/main/api/LICENSE";
const ROYALEHOSTING: &str = "https://royalehosting.net/?partner=cobalt";

fn page_source(page: AboutPage, locale: &str) -> &'static str {
    match (page, locale) {
        (AboutPage::General, "ru") => include_str!("../../../assets/i18n/ru/about/general.md"),
        (AboutPage::Privacy, "ru") => include_str!("../../../assets/i18n/ru/about/privacy.md"),
        (AboutPage::Terms, "ru") => include_str!("../../../assets/i18n/ru/about/terms.md"),
        (AboutPage::Credits, "ru") => include_str!("../../../assets/i18n/ru/about/credits.md"),
        (AboutPage::General, _) => include_str!("../../../assets/i18n/en/about/general.md"),
        (AboutPage::Privacy, _) => include_str!("../../../assets/i18n/en/about/privacy.md"),
        (AboutPage::Terms, _) => include_str!("../../../assets/i18n/en/about/terms.md"),
        (AboutPage::Credits, _) => include_str!("../../../assets/i18n/en/about/credits.md"),
        _ => "",
    }
}

fn prepare(src: &str) -> String {
    let mut out = String::new();
    let mut skip_plausible = false;
    let mut in_heading = false;
    let mut heading_title = String::new();
    for line in src.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("{#if env.PLAUSIBLE_ENABLED}") {
            skip_plausible = true;
            continue;
        }
        if skip_plausible {
            if trimmed.starts_with("{/if}") {
                skip_plausible = false;
            }
            continue;
        }
        if trimmed.starts_with("<SectionHeading") {
            in_heading = true;
            heading_title.clear();
        }
        if in_heading {
            if let Some(rest) = trimmed.strip_prefix("title=") {
                let rest = rest.trim_end_matches("/>").trim();
                if let Some(start) = rest.find("$t(\"") {
                    let key = &rest[start + 4..];
                    if let Some(end) = key.find('"') {
                        heading_title = t(&key[..end]);
                    }
                } else {
                    heading_title = rest.trim_matches('"').to_string();
                }
            }
            if trimmed.ends_with("/>") {
                in_heading = false;
                if !heading_title.is_empty() {
                    out.push_str(&format!("### {}\n", cap(&heading_title)));
                }
            }
            continue;
        }
        let mut l = line.to_string();
        for (from, to) in [
            ("{contacts.github}", GITHUB),
            ("{contacts.discord}", DISCORD),
            ("{contacts.twitter}", TWITTER),
            ("{contacts.bluesky}", BLUESKY),
            ("{docs.instanceHosting}", INSTANCE_HOSTING),
            ("{docs.webLicense}", WEB_LICENSE),
            ("{docs.apiLicense}", API_LICENSE),
            ("{partners.royalehosting}", ROYALEHOSTING),
        ] {
            l = l.replace(from, to);
        }
        out.push_str(&l);
        out.push('\n');
    }
    out
}

const DESKTOP_ABOUT: &str = r#"### About this desktop client

A native desktop client written in Rust. It speaks the public cobalt API exactly like the web app does: the same request body, the same response handling (tunnel, redirect, picker, local processing) and the very same ffmpeg arguments for on-device remuxing and transcoding. The interface is our own design.

The official instance (api.cobalt.tools) requires a Cloudflare Turnstile check that only a browser can pass, so point the client at your own or a community instance in [settings → instances](/settings/instances).

### Notes

- on-device processing uses a real ffmpeg binary. put `ffmpeg.exe` next to the app or install it into PATH.
- saving methods are "ask", "download" and "copy"; files land in the folder configured in desktop settings.
- turnstile (captcha) checks can't be completed outside of a browser.

### Credits

developed by **Berkay AY** ([@shadesofdeath](https://github.com/shadesofdeath)). source code and releases: [github.com/shadesofdeath/cobalt-desktop](https://github.com/shadesofdeath/cobalt-desktop) (MIT).

cobalt is made by [imput](https://imput.net/). the web app is licensed under CC-BY-NC-SA 4.0 and the api under AGPL-3.0; this client is not affiliated with imput.

fonts: Inter and IBM Plex Mono (OFL). icons: [tabler icons](https://tabler.io/icons) (MIT). ui: [egui](https://github.com/emilk/egui).
"#;

pub fn show(app: &mut App, c: &Ctx, ui: &mut Ui) {
    let th = c.t();
    let page = app.about_page;
    let locale = crate::i18n::current_locale();
    page_body(ui, "about", 860.0, |ui, w| {
        let options = [
            SegOption { value: "general", label: cap(&t("about.page.general")), icon: None },
            SegOption { value: "community", label: cap(&t("about.page.community")), icon: None },
            SegOption { value: "privacy", label: cap(&t("about.page.privacy")), icon: None },
            SegOption { value: "terms", label: cap(&t("about.page.terms")), icon: None },
            SegOption { value: "credits", label: cap(&t("about.page.credits")), icon: None },
            SegOption { value: "desktop", label: cap(&t("about.page.desktop")), icon: None },
        ];
        let current = match page {
            AboutPage::General => "general",
            AboutPage::Community => "community",
            AboutPage::Privacy => "privacy",
            AboutPage::Terms => "terms",
            AboutPage::Credits => "credits",
            AboutPage::Desktop => "desktop",
        };
        ui.add_space(8.0);
        if let Some(v) = widgets::segmented(ui, c, current, &options, true, 40.0) {
            app.about_page = match v.as_str() {
                "community" => AboutPage::Community,
                "privacy" => AboutPage::Privacy,
                "terms" => AboutPage::Terms,
                "credits" => AboutPage::Credits,
                "desktop" => AboutPage::Desktop,
                _ => AboutPage::General,
            };
        }
        match page {
            AboutPage::Community => community(c, ui, &locale, w),
            AboutPage::Desktop => {
                widgets::card(ui, c, w, |ui| markdown::render(ui, c, DESKTOP_ABOUT));
            }
            _ => {
                let md = prepare(page_source(page, &locale));
                widgets::card(ui, c, w, |ui| markdown::render(ui, c, &md));
            }
        }
        let _ = th;
    });
}

fn community(c: &Ctx, ui: &mut Ui, locale: &str, w: f32) {
    let th = c.t();
    let platforms: Vec<(&str, &str, &str, &str)> = if locale == "ru" {
        vec![("GitHub", "brand-github", GITHUB, "about.support.github"), ("Telegram", "brand-telegram", TELEGRAM_RU, "about.support.telegram")]
    } else {
        vec![
            ("GitHub", "brand-github", GITHUB, "about.support.github"),
            ("Discord", "brand-discord", DISCORD, "about.support.discord"),
            ("X / Twitter", "brand-x", TWITTER, "about.support.twitter"),
            ("Bluesky", "brand-bluesky", BLUESKY, "about.support.bluesky"),
        ]
    };
    let cols = if w >= 560.0 { 2 } else { 1 };
    let gap = 12.0;
    let cw = (w - gap * (cols as f32 - 1.0)) / cols as f32;
    for row in platforms.chunks(cols) {
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for (name, icon, url, desc) in row {
                ui.allocate_ui_with_layout(Vec2::new(cw, 0.0), egui::Layout::top_down(Align::Min), |ui| {
                    let r = ui.scope(|ui| {
                        widgets::card(ui, c, cw, |ui| {
                            ui.spacing_mut().item_spacing.y = 10.0;
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 10.0;
                                widgets::icon_tile(ui, c, icon, 36.0, th.accent);
                                widgets::text(ui, *name, theme::semibold(15.0), th.text);
                                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                    let (r, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                                    c.icons.image("external-link", 16.0, th.text_faint).paint_at(ui, r);
                                });
                            });
                            widgets::wrapped(ui, &cap(&t(desc)), theme::regular(13.0), th.text_muted, cw - 40.0, 1.5);
                        });
                    });
                    let resp = ui.interact(r.response.rect, ui.id().with(("community", name)), Sense::click());
                    if resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if resp.clicked() {
                        let _ = open::that_detached(url);
                    }
                });
            }
        });
    }
    let mut note = t("about.support.description.issue");
    if locale != "ru" {
        note.push(' ');
        note.push_str(&t("about.support.description.help"));
    }
    note.push(' ');
    note.push_str(&t("about.support.description.best-effort"));
    widgets::wrapped(ui, &cap(&note), theme::regular(13.0), th.text_muted, w, 1.5);
}
