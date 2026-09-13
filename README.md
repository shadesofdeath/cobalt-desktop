<div align="center">

# cobalt desktop

**A modern, native desktop client for [cobalt](https://github.com/imputnet/cobalt) — written in Rust.**

Paste a link, pick a mode, hit download. Powered by the cobalt API and your own processing instance.

Developed by [Berkay AY (@shadesofdeath)](https://github.com/shadesofdeath)

[![Release](https://img.shields.io/github/v/release/shadesofdeath/cobalt-desktop?style=flat-square)](https://github.com/shadesofdeath/cobalt-desktop/releases)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
![Rust](https://img.shields.io/badge/rust-egui%2Feframe-orange?style=flat-square)
![Platform](https://img.shields.io/badge/platform-windows-lightgrey?style=flat-square)

<img src="docs/screenshots/download-dark.png" width="820" alt="cobalt desktop – download page (dark)">

</div>

## Features

- **Download** from 20+ services (YouTube, TikTok, Instagram, Twitter/X, Reddit, SoundCloud, …) through any cobalt API instance.
- **Auto / Audio / Mute** modes, quality presets, codec & container selection, filename styles, subtitles, metadata.
- **On-device processing**: remuxing and transcoding run locally with ffmpeg, using the exact same arguments the cobalt web app uses.
- **Activity**: live progress for every download and remux, inline on the download page and in a slide-in side panel. Retry, remove, open folder.
- **Picker** for multi-media posts with thumbnails, **saving methods** (ask / download / copy link), toast notifications.
- **Remux** any local media file losslessly (drag & drop).
- **Custom instances & access keys**, live instance status, all cobalt settings, import/export, light & dark themes.
- Keyboard shortcuts: `/` focus, `Enter` download, `Esc` clear, `Shift+D` paste, `Shift+J/K/L` switch modes.

## Screenshots

| Download (light) | Activity |
|---|---|
| ![](docs/screenshots/download-light.png) | ![](docs/screenshots/activity.png) |

| Settings | Activity panel |
|---|---|
| ![](docs/screenshots/settings-video.png) | ![](docs/screenshots/activity-panel.png) |

| Remux | Updates |
|---|---|
| ![](docs/screenshots/remux.png) | ![](docs/screenshots/updates.png) |

## Download

Grab the latest build from the [Releases](https://github.com/shadesofdeath/cobalt-desktop/releases) page,
extract it and run `cobalt-desktop.exe`.

### Requirements

- Windows 10/11 (the code also builds on Linux and macOS).
- [ffmpeg](https://ffmpeg.org/) in `PATH` or next to the executable (needed for local processing and remux).
- A cobalt processing instance. The official `api.cobalt.tools` requires a Cloudflare Turnstile check that only a
  browser can pass, so use [your own instance](https://github.com/imputnet/cobalt/blob/main/docs/run-an-instance.md)
  or a community instance and set it in **Settings → Instances**.

## Build from source

```
git clone https://github.com/shadesofdeath/cobalt-desktop
cd cobalt-desktop
cargo build --release
./target/release/cobalt-desktop.exe
```

### Screenshot mode (for docs / testing)

```
cobalt-desktop.exe --screenshot <dir>          # renders every page and saves PNGs
cobalt-desktop.exe --screenshot <dir> --live   # also runs a real download via http://localhost:9000
```

### Running a local API for testing

```
git clone https://github.com/imputnet/cobalt
cd cobalt && corepack pnpm install --filter @imput/cobalt-api...
cd api && API_URL=http://localhost:9000 API_PORT=9000 node src/cobalt.js
```

Then set `http://localhost:9000` as the custom instance in the app.

## How it works

The client speaks cobalt's public API exactly like the web app does: `GET /` for instance info, `POST /` with the
same request body (only settings that differ from defaults are sent), tunnel probing, and the same handling of
`tunnel`, `redirect`, `picker` and `local-processing` responses. Local processing is a port of the web app's
task manager: fetch workers stream tunnels to disk, then ffmpeg merges / mutes / remuxes / encodes with the same
arguments libav.js receives in the browser.

## Project structure

```
src/
  api.rs        cobalt api client (requests, responses, auth, tunnel probe)
  queue.rs      processing queue & scheduler (fetch + ffmpeg workers)
  ffmpeg.rs     ffmpeg runner with -progress parsing
  settings.rs   settings model (cobalt schema v6 + desktop section)
  i18n.rs       translations (cobalt's en/ru files + desktop strings)
  theme.rs      design system: palette, fonts, icons
  ui/           shell, widgets, pages, activity panel, dialogs
  screenshot.rs --screenshot mode
```

## License

Code: [MIT](LICENSE) © 2026 Berkay AY.
Translations and changelogs come from the cobalt web app (CC-BY-NC-SA 4.0). Fonts: Inter & IBM Plex Mono (OFL).
Icons: Tabler Icons (MIT). UI: egui.

This project is not affiliated with imput or cobalt. cobalt is made by [imput](https://imput.net/).
