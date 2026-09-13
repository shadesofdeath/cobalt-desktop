//! changelog markdown files from `web/changelogs/*.md`, sorted by version (newest first).

use once_cell::sync::Lazy;

macro_rules! changelog_files {
    ($($v:literal),* $(,)?) => {
        &[$(($v, include_str!(concat!("../assets/changelogs/", $v, ".md")))),*]
    };
}

const FILES: &[(&str, &str)] = changelog_files![
    "10.0", "10.1", "10.3", "10.5", "11.0", "11.2",
    "2.0", "2.2", "2.2.5", "2.2.6", "2.2.8", "2.2.9",
    "3.0", "3.1", "3.2", "3.4", "3.5", "3.5.2", "3.5.4", "3.6", "3.6.3", "3.7",
    "4.0", "4.1", "4.2", "4.3", "4.3.2", "4.4", "4.5", "4.6", "4.7", "4.8",
    "5.0", "5.1", "5.2", "5.3", "5.4",
    "6.0", "6.2",
    "7.0", "7.1", "7.3", "7.4", "7.5", "7.6", "7.7", "7.8", "7.9", "7.11", "7.13", "7.14",
];

#[derive(Clone, Debug)]
pub struct Changelog {
    pub version: String,
    pub title: String,
    pub date: String,
    pub banner_alt: String,
    pub body: String,
}

fn parse(version: &str, text: &str) -> Changelog {
    let mut title = String::new();
    let mut date = String::new();
    let mut banner_alt = String::new();
    let mut body = text.to_string();
    if let Some(rest) = text.strip_prefix("---") {
        if let Some(end) = rest.find("\n---") {
            let front = &rest[..end];
            body = rest[end + 4..].trim_start_matches('\n').to_string();
            for line in front.lines() {
                let line = line.trim();
                let unquote = |s: &str| s.trim().trim_matches('"').to_string();
                if let Some(v) = line.strip_prefix("title:") {
                    title = unquote(v);
                } else if let Some(v) = line.strip_prefix("date:") {
                    date = unquote(v);
                } else if let Some(v) = line.strip_prefix("alt:") {
                    banner_alt = unquote(v);
                }
            }
        }
    }
    Changelog { version: version.to_string(), title, date, banner_alt, body }
}

fn version_key(v: &str) -> (u32, u32, u32) {
    let mut it = v.split('.');
    let a = it.next().and_then(|x| x.parse().ok()).unwrap_or(0);
    let b = it.next().and_then(|x| x.parse().ok()).unwrap_or(0);
    let c = it.next().and_then(|x| x.parse().ok()).unwrap_or(0);
    (a, b, c)
}

pub static CHANGELOGS: Lazy<Vec<Changelog>> = Lazy::new(|| {
    let mut list: Vec<Changelog> = FILES.iter().map(|(v, t)| parse(v, t)).collect();
    list.sort_by(|a, b| version_key(&b.version).cmp(&version_key(&a.version)));
    list
});
