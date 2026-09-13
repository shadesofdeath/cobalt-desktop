#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

mod api;
mod app;
mod changelogs;
mod ffmpeg;
mod i18n;
mod queue;
mod screenshot;
mod settings;
mod theme;
mod ui;

use app::App;
use std::sync::Arc;

struct Cobalt {
    app: App,
}

impl eframe::App for Cobalt {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        let ctx = &ctx;
        self.app.pump_events();
        ui::show(&mut self.app, root);
        self.screenshot_step(ctx);
        // background tasks may change state while nothing repaints
        ctx.request_repaint_after(std::time::Duration::from_millis(250));
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        let c = self.app.theme.bg_elevated;
        [c.r() as f32 / 255.0, c.g() as f32 / 255.0, c.b() as f32 / 255.0, 1.0]
    }
}

impl Cobalt {
    fn screenshot_step(&mut self, ctx: &egui::Context) {
        let Some(plan) = self.app.screenshot.as_mut() else { return };
        // collect finished screenshots
        let mut done: Vec<(String, Arc<egui::ColorImage>)> = Vec::new();
        ctx.input(|i| {
            for ev in &i.raw.events {
                if let egui::Event::Screenshot { user_data, image, .. } = ev {
                    if let Some(name) = user_data.data.as_ref().and_then(|d| d.downcast_ref::<String>()) {
                        done.push((name.clone(), image.clone()));
                    }
                }
            }
        });
        let mut advance = false;
        for (name, image) in done {
            let path = plan.out_dir.join(format!("{name}.png"));
            match screenshot::save_png(&image, &path) {
                Ok(()) => eprintln!("saved {}", path.display()),
                Err(e) => eprintln!("failed to save {}: {e}", path.display()),
            }
            advance = true;
        }
        if advance {
            plan.index += 1;
            plan.frames = 0;
            plan.requested = false;
            plan.started = None;
        }
        if plan.index >= plan.scenes.len() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        plan.frames += 1;
        if plan.frames == 1 {
            let setup = plan.scenes[plan.index].setup;
            plan.started = Some(std::time::Instant::now());
            let _ = plan;
            setup(&mut self.app);
            ctx.request_repaint();
            return;
        }
        let plan = self.app.screenshot.as_mut().unwrap();
        let wait = plan.scenes[plan.index].wait;
        let elapsed = plan.started.map(|s| s.elapsed().as_secs_f32()).unwrap_or(0.0);
        if plan.frames >= 6 && elapsed >= wait && !plan.requested {
            plan.requested = true;
            let name = plan.scenes[plan.index].name.to_string();
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(name)));
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(16));
    }
}

/// window icon: the cobalt double chevron on a dark rounded square
fn window_icon() -> egui::IconData {
    let size = 64usize;
    let mut rgba = vec![0u8; size * size * 4];
    // the two chevron polygons from the logo svg (24x16 viewbox), scaled into the icon
    let paths: [[(f32, f32); 7]; 2] = [
        [(0.0, 15.6363), (0.0, 12.8594), (9.47552, 8.293), (0.0, 3.14038), (0.0, 0.363525), (12.8575, 7.4908), (12.8575, 9.21862)],
        [(11.1425, 15.6363), (11.1425, 12.8594), (20.6181, 8.293), (11.1425, 3.14038), (11.1425, 0.363525), (24.0, 7.4908), (24.0, 9.21862)],
    ];
    let scale = 40.0 / 24.0;
    let ox = 12.0;
    let oy = (64.0 - 16.0 * scale) / 2.0;
    let inside = |px: f32, py: f32, poly: &[(f32, f32); 7]| -> bool {
        let mut c = false;
        let n = poly.len();
        let mut j = n - 1;
        for i in 0..n {
            let (xi, yi) = (poly[i].0 * scale + ox, poly[i].1 * scale + oy);
            let (xj, yj) = (poly[j].0 * scale + ox, poly[j].1 * scale + oy);
            if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi + 1e-6) + xi) {
                c = !c;
            }
            j = i;
        }
        c
    };
    for y in 0..size {
        for x in 0..size {
            let i = (y * size + x) * 4;
            // rounded square background
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;
            let r = 14.0;
            let cx = fx.clamp(r, 64.0 - r);
            let cy = fy.clamp(r, 64.0 - r);
            let d = ((fx - cx).powi(2) + (fy - cy).powi(2)).sqrt();
            if d > r {
                continue;
            }
            rgba[i..i + 4].copy_from_slice(&[0x13, 0x13, 0x13, 0xff]);
            // supersample the chevrons
            let mut cover = 0;
            for sy in 0..4 {
                for sx in 0..4 {
                    let px = x as f32 + (sx as f32 + 0.5) / 4.0;
                    let py = y as f32 + (sy as f32 + 0.5) / 4.0;
                    if paths.iter().any(|p| inside(px, py, p)) {
                        cover += 1;
                    }
                }
            }
            if cover > 0 {
                let a = cover as f32 / 16.0;
                let v = (0x13 as f32 * (1.0 - a) + 0xe1 as f32 * a) as u8;
                rgba[i..i + 3].copy_from_slice(&[v, v, v]);
            }
        }
    }
    egui::IconData { rgba, width: size as u32, height: size as u32 }
}

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    let mut plan = None;
    if let Some(pos) = args.iter().position(|a| a == "--screenshot") {
        let dir = args.get(pos + 1).map(std::path::PathBuf::from).unwrap_or_else(|| std::path::PathBuf::from("screenshots"));
        let live = args.iter().any(|a| a == "--live");
        plan = Some(screenshot::default_plan(dir, live));
    }

    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("tokio runtime");
    let _guard = rt.enter();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("cobalt")
            .with_inner_size([1200.0, 780.0])
            .with_min_inner_size([760.0, 560.0])
            .with_icon(Arc::new(window_icon())),
        ..Default::default()
    };
    eframe::run_native(
        "cobalt",
        options,
        Box::new(move |cc| Ok(Box::new(Cobalt { app: App::new(cc, plan) }))),
    )
}
