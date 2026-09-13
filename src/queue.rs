//! processing queue & task manager.
//! a port of `web/src/lib/task-manager/*` and `web/src/lib/state/task-manager/*`:
//! fetch workers stream tunnels to disk, remux/encode workers run ffmpeg with the
//! same arguments the web app passes to libav.js.

use crate::api::{LocalProcessingResponse, SaveRequest};
use crate::settings::CobaltSettings;
use futures_util::StreamExt;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::Notify;

pub type Uuid = String;

pub fn uuid() -> Uuid {
    uuid::Uuid::new_v4().to_string()
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum WorkerKind {
    Fetch,
    Remux,
    Encode,
}

impl WorkerKind {
    pub fn key(&self) -> &'static str {
        match self {
            WorkerKind::Fetch => "fetch",
            WorkerKind::Remux => "remux",
            WorkerKind::Encode => "encode",
        }
    }
}

#[derive(Clone, Debug)]
pub enum WorkerArgs {
    Fetch {
        url: String,
        /// take the filename from Content-Disposition / content type (plain proxy downloads)
        header_filename: bool,
    },
    Ffmpeg {
        ffargs: Vec<String>,
        /// output container/extension (e.g. `mp4`)
        format: String,
        /// output mime type
        mime: String,
        /// input files that are already on disk (remux tab)
        files: Vec<PathBuf>,
    },
}

#[derive(Clone, Debug)]
pub struct PipelineItem {
    pub worker: WorkerKind,
    pub worker_id: Uuid,
    pub parent_id: Uuid,
    pub depends_on: Vec<Uuid>,
    pub args: WorkerArgs,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaType {
    File,
    Video,
    Audio,
    Image,
}

/// `getMediaType()` from queue.ts
pub fn media_type_from_mime(mime: &str) -> MediaType {
    match mime.split('/').next().unwrap_or("") {
        "video" => MediaType::Video,
        "audio" => MediaType::Audio,
        "image" => MediaType::Image,
        _ => MediaType::File,
    }
}

#[derive(Clone, Debug, Default)]
pub struct WorkerProgress {
    pub percentage: f32,
    pub size: u64,
}

#[derive(Clone, Debug)]
pub enum ItemState {
    Waiting,
    Running,
    Done {
        file: PathBuf,
        size: u64,
        /// where the file was saved with the user's saving method (if it was)
        saved_to: Option<PathBuf>,
    },
    Error {
        code: String,
    },
}

#[derive(Clone, Debug)]
pub struct QueueItem {
    pub id: Uuid,
    pub pipeline: Vec<PipelineItem>,
    pub can_retry: bool,
    pub original_request: Option<SaveRequest>,
    pub filename: String,
    pub mime: String,
    pub media_type: MediaType,
    pub state: ItemState,
    /// worker id -> (file, size)
    pub pipeline_results: HashMap<Uuid, (PathBuf, u64)>,
    /// running workers with their latest progress (`currentTasks` in the web app)
    pub current_tasks: HashMap<Uuid, Option<WorkerProgress>>,
    pub retrying: bool,
}

impl QueueItem {
    /// `getProgress()` from queue.ts
    pub fn progress(&self) -> f32 {
        match self.state {
            ItemState::Done { .. } | ItemState::Error { .. } => return 1.0,
            ItemState::Waiting => return 0.0,
            ItemState::Running => {}
        }
        if self.pipeline.is_empty() {
            return 0.0;
        }
        let mut sum = 0.0;
        for w in &self.pipeline {
            if self.pipeline_results.contains_key(&w.worker_id) {
                sum += 1.0;
            } else if let Some(Some(p)) = self.current_tasks.get(&w.worker_id) {
                sum += p.percentage / 100.0;
            }
        }
        sum / self.pipeline.len() as f32
    }

    pub fn worker_progress(&self, worker_id: &str) -> f32 {
        if self.pipeline_results.contains_key(worker_id) {
            return 100.0;
        }
        if let Some(Some(p)) = self.current_tasks.get(worker_id) {
            return p.percentage.clamp(0.0, 100.0);
        }
        0.0
    }
}

pub enum QueueEvent {
    /// an item finished: the ui decides how to save it
    ItemDone(Uuid),
    ItemError(Uuid, String),
}

#[derive(Default)]
pub struct QueueState {
    pub items: Vec<QueueItem>,
}

impl QueueState {
    pub fn get_mut(&mut self, id: &str) -> Option<&mut QueueItem> {
        self.items.iter_mut().find(|i| i.id == id)
    }
    pub fn get(&self, id: &str) -> Option<&QueueItem> {
        self.items.iter().find(|i| i.id == id)
    }
    fn any_running_worker(&self) -> bool {
        self.items.iter().any(|i| !i.current_tasks.is_empty())
    }
}

pub struct TaskManager {
    pub state: Arc<Mutex<QueueState>>,
    notify: Arc<Notify>,
    http: reqwest::Client,
    cache_dir: PathBuf,
    ffmpeg: Arc<Mutex<Option<PathBuf>>>,
    events: crossbeam_channel::Sender<QueueEvent>,
    repaint: Arc<Mutex<Option<egui::Context>>>,
}

impl TaskManager {
    pub fn new(http: reqwest::Client, events: crossbeam_channel::Sender<QueueEvent>) -> Arc<Self> {
        let cache_dir = CobaltSettings::cache_dir().join("queue");
        let _ = std::fs::create_dir_all(&cache_dir);
        let tm = Arc::new(Self {
            state: Arc::new(Mutex::new(QueueState::default())),
            notify: Arc::new(Notify::new()),
            http,
            cache_dir,
            ffmpeg: Arc::new(Mutex::new(None)),
            events,
            repaint: Arc::new(Mutex::new(None)),
        });
        // clear old files from storage on first launch (clearFileStorage in ProcessingQueue.svelte)
        tm.clear_file_storage();
        let sched = tm.clone();
        tokio::spawn(async move {
            loop {
                let _ = tokio::time::timeout(std::time::Duration::from_millis(500), sched.notify.notified()).await;
                sched.schedule();
            }
        });
        tm
    }

    pub fn set_repaint_context(&self, ctx: egui::Context) {
        *self.repaint.lock() = Some(ctx);
    }

    fn repaint(&self) {
        if let Some(ctx) = self.repaint.lock().as_ref() {
            ctx.request_repaint();
        }
    }

    pub fn set_ffmpeg(&self, path: Option<PathBuf>) {
        *self.ffmpeg.lock() = path;
    }

    pub fn ffmpeg_path(&self) -> Option<PathBuf> {
        self.ffmpeg.lock().clone()
    }

    pub fn clear_file_storage(&self) {
        if let Ok(entries) = std::fs::read_dir(&self.cache_dir) {
            for e in entries.flatten() {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }

    pub fn snapshot(&self) -> Vec<QueueItem> {
        self.state.lock().items.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.state.lock().items.is_empty()
    }

    /// `totalProgress` from ProcessingQueue.svelte
    pub fn total_progress(&self) -> (f32, bool) {
        let st = self.state.lock();
        if st.items.is_empty() {
            return (0.0, false);
        }
        let sum: f32 = st.items.iter().map(|i| i.progress()).sum();
        let total = sum / st.items.len() as f32;
        (total, total == 0.0)
    }

    pub fn add_item(&self, item: QueueItem) {
        {
            let mut st = self.state.lock();
            // retry re-uses the old task id: replace the old entry in place
            if let Some(pos) = st.items.iter().position(|i| i.id == item.id) {
                st.items[pos] = item;
            } else {
                st.items.push(item);
            }
        }
        self.notify.notify_one();
        self.repaint();
    }

    pub fn remove_item(&self, id: &str) {
        let removed = {
            let mut st = self.state.lock();
            let pos = st.items.iter().position(|i| i.id == id);
            pos.map(|p| st.items.remove(p))
        };
        if let Some(item) = removed {
            self.clear_pipeline_cache(&item, true);
        }
        self.notify.notify_one();
    }

    pub fn clear_queue(&self) {
        let items: Vec<QueueItem> = std::mem::take(&mut self.state.lock().items);
        for item in &items {
            self.clear_pipeline_cache(item, true);
        }
        self.clear_file_storage();
    }

    pub fn set_retrying(&self, id: &str, retrying: bool) {
        if let Some(item) = self.state.lock().get_mut(id) {
            item.retrying = retrying;
        }
        self.repaint();
    }

    pub fn mark_saved(&self, id: &str, path: PathBuf) {
        if let Some(item) = self.state.lock().get_mut(id) {
            if let ItemState::Done { saved_to, .. } = &mut item.state {
                *saved_to = Some(path);
            }
        }
        self.repaint();
    }

    fn clear_pipeline_cache(&self, item: &QueueItem, including_result: bool) {
        for (_, (path, _)) in &item.pipeline_results {
            let _ = std::fs::remove_file(path);
        }
        if including_result {
            if let ItemState::Done { file, .. } = &item.state {
                let _ = std::fs::remove_file(file);
            }
        }
    }

    // --- pipeline builders (queue.ts) ---

    /// `createSavePipeline()`
    pub fn create_save_pipeline(
        &self,
        info: &LocalProcessingResponse,
        request: SaveRequest,
        old_task_id: Option<String>,
    ) -> Result<Uuid, String> {
        if info.output.filename.is_empty() || info.output.mime.is_empty() {
            return Err("pipeline.missing_response_data".into());
        }
        let parent_id = old_task_id.unwrap_or_else(uuid);
        let mut pipeline: Vec<PipelineItem> = Vec::new();

        // reverse is needed for audio (second item) to be downloaded first
        let mut tunnels = info.tunnel.clone();
        tunnels.reverse();
        for tunnel in tunnels {
            pipeline.push(PipelineItem {
                worker: WorkerKind::Fetch,
                worker_id: uuid(),
                parent_id: parent_id.clone(),
                depends_on: vec![],
                args: WorkerArgs::Fetch { url: tunnel, header_filename: false },
            });
        }

        if info.kind != "proxy" {
            let (worker, ffargs) = match info.kind.as_str() {
                "merge" | "mute" | "remux" => (WorkerKind::Remux, make_remux_args(info)),
                "audio" => match make_audio_args(info) {
                    Some(a) => (WorkerKind::Encode, a),
                    None => return Err("pipeline.missing_response_data".into()),
                },
                "gif" => (WorkerKind::Encode, make_gif_args()),
                _ => return Err("pipeline.missing_response_data".into()),
            };
            let depends_on: Vec<Uuid> = pipeline.iter().map(|w| w.worker_id.clone()).collect();
            pipeline.push(PipelineItem {
                worker,
                worker_id: uuid(),
                parent_id: parent_id.clone(),
                depends_on,
                args: WorkerArgs::Ffmpeg {
                    ffargs,
                    format: info.output.filename.rsplit('.').next().unwrap_or("bin").to_string(),
                    mime: info.output.mime.clone(),
                    files: vec![],
                },
            });
        }

        self.add_item(QueueItem {
            id: parent_id.clone(),
            pipeline,
            can_retry: true,
            original_request: Some(request),
            filename: info.output.filename.clone(),
            mime: info.output.mime.clone(),
            media_type: media_type_from_mime(&info.output.mime),
            state: ItemState::Waiting,
            pipeline_results: HashMap::new(),
            current_tasks: HashMap::new(),
            retrying: false,
        });
        Ok(parent_id)
    }

    /// a plain proxied download (tunnel / redirect url) that goes through the queue
    pub fn create_proxy_pipeline(&self, url: &str, filename: &str, request: Option<SaveRequest>) -> Uuid {
        let parent_id = uuid();
        let mime = mime_from_filename(filename);
        self.add_item(QueueItem {
            id: parent_id.clone(),
            pipeline: vec![PipelineItem {
                worker: WorkerKind::Fetch,
                worker_id: uuid(),
                parent_id: parent_id.clone(),
                depends_on: vec![],
                args: WorkerArgs::Fetch { url: url.to_string(), header_filename: true },
            }],
            can_retry: request.is_some(),
            original_request: request,
            filename: filename.to_string(),
            mime: mime.clone(),
            media_type: media_type_from_mime(&mime),
            state: ItemState::Waiting,
            pipeline_results: HashMap::new(),
            current_tasks: HashMap::new(),
            retrying: false,
        });
        parent_id
    }

    /// `createRemuxPipeline()` - remux tab
    pub fn create_remux_pipeline(&self, file: &Path) -> Option<Uuid> {
        let name = file.file_name()?.to_string_lossy().to_string();
        let mime = mime_from_filename(&name);
        let media_type = media_type_from_mime(&mime);
        if !matches!(media_type, MediaType::Video | MediaType::Audio) {
            return None;
        }
        let parent_id = uuid();
        let format = name.rsplit('.').next().unwrap_or("mp4").to_string();
        self.add_item(QueueItem {
            id: parent_id.clone(),
            pipeline: vec![PipelineItem {
                worker: WorkerKind::Remux,
                worker_id: uuid(),
                parent_id: parent_id.clone(),
                depends_on: vec![],
                args: WorkerArgs::Ffmpeg {
                    ffargs: vec!["-c".into(), "copy".into(), "-map".into(), "0".into()],
                    format,
                    mime: mime.clone(),
                    files: vec![file.to_path_buf()],
                },
            }],
            can_retry: false,
            original_request: None,
            filename: name,
            mime,
            media_type,
            state: ItemState::Waiting,
            pipeline_results: HashMap::new(),
            current_tasks: HashMap::new(),
            retrying: false,
        });
        Some(parent_id)
    }

    // --- scheduler (scheduler.ts) ---

    fn schedule(self: &Arc<Self>) {
        let mut to_start: Vec<PipelineItem> = Vec::new();
        let mut done_events: Vec<QueueEvent> = Vec::new();
        {
            let mut st = self.state.lock();
            let any_running = st.any_running_worker();
            let mut finished: Vec<(Uuid, Result<(PathBuf, u64), String>, Vec<PathBuf>)> = Vec::new();
            let mut started_waiting = false;

            for item in st.items.iter_mut() {
                match item.state {
                    ItemState::Running => {
                        if item.pipeline_results.len() == item.pipeline.len() && !item.pipeline.is_empty() {
                            let final_worker = item.pipeline.last().unwrap().worker_id.clone();
                            let final_file = item.pipeline_results.remove(&final_worker);
                            let intermediates: Vec<PathBuf> =
                                item.pipeline_results.drain().map(|(_, (p, _))| p).collect();
                            match final_file {
                                Some((path, size)) => {
                                    item.state = ItemState::Done { file: path.clone(), size, saved_to: None };
                                    finished.push((item.id.clone(), Ok((path, size)), intermediates));
                                }
                                None => {
                                    item.state = ItemState::Error { code: "queue.no_final_file".into() };
                                    finished.push((item.id.clone(), Err("queue.no_final_file".into()), intermediates));
                                }
                            }
                            continue;
                        }
                        for worker in &item.pipeline {
                            if item.pipeline_results.contains_key(&worker.worker_id)
                                || item.current_tasks.contains_key(&worker.worker_id)
                            {
                                continue;
                            }
                            let needs_to_wait =
                                worker.depends_on.iter().any(|id| !item.pipeline_results.contains_key(id));
                            if needs_to_wait {
                                break;
                            }
                            item.current_tasks.insert(worker.worker_id.clone(), None);
                            to_start.push(worker.clone());
                        }
                        // don't start next tasks before this one is done
                        started_waiting = true;
                        break;
                    }
                    ItemState::Waiting if !item.pipeline.is_empty() && !any_running && !started_waiting => {
                        item.state = ItemState::Running;
                        let first = item.pipeline[0].clone();
                        item.current_tasks.insert(first.worker_id.clone(), None);
                        to_start.push(first);
                        started_waiting = true;
                        break;
                    }
                    _ => {}
                }
            }

            for (id, result, intermediates) in finished {
                for p in intermediates {
                    let _ = std::fs::remove_file(p);
                }
                match result {
                    Ok(_) => done_events.push(QueueEvent::ItemDone(id)),
                    Err(code) => done_events.push(QueueEvent::ItemError(id, code)),
                }
            }
        }

        for ev in done_events {
            let _ = self.events.send(ev);
            self.repaint();
        }

        if !to_start.is_empty() {
            self.repaint();
        }
        for worker in to_start {
            let tm = self.clone();
            tokio::spawn(async move {
                tm.start_worker(worker).await;
            });
        }
    }

    fn set_progress(&self, parent: &str, worker: &str, progress: WorkerProgress) {
        if let Some(item) = self.state.lock().get_mut(parent) {
            if let Some(slot) = item.current_tasks.get_mut(worker) {
                *slot = Some(progress);
            }
        }
        self.repaint();
    }

    fn pipeline_task_done(self: &Arc<Self>, parent: &str, worker: &str, file: PathBuf, size: u64) {
        {
            let mut st = self.state.lock();
            if let Some(item) = st.get_mut(parent) {
                if matches!(item.state, ItemState::Running) {
                    item.pipeline_results.insert(worker.to_string(), (file, size));
                }
                item.current_tasks.remove(worker);
            } else {
                let _ = std::fs::remove_file(&file);
            }
        }
        self.notify.notify_one();
        self.repaint();
    }

    fn item_error(self: &Arc<Self>, parent: &str, worker: &str, code: &str) {
        let code = if code.starts_with("queue.") { code.to_string() } else { format!("queue.{code}") };
        let mut to_remove = Vec::new();
        {
            let mut st = self.state.lock();
            if let Some(item) = st.get_mut(parent) {
                for (_, (p, _)) in item.pipeline_results.drain() {
                    to_remove.push(p);
                }
                item.current_tasks.clear();
                item.state = ItemState::Error { code: code.clone() };
                let _ = worker;
            }
        }
        for p in to_remove {
            let _ = std::fs::remove_file(p);
        }
        let _ = self.events.send(QueueEvent::ItemError(parent.to_string(), code));
        self.notify.notify_one();
        self.repaint();
    }

    async fn start_worker(self: Arc<Self>, worker: PipelineItem) {
        match &worker.args {
            WorkerArgs::Fetch { url, header_filename } => {
                let url = url.clone();
                self.run_fetch_worker(&worker, &url, *header_filename).await;
            }
            WorkerArgs::Ffmpeg { ffargs, format, mime, files } => {
                let mut inputs: Vec<PathBuf> = files.clone();
                {
                    let st = self.state.lock();
                    if let Some(parent) = st.get(&worker.parent_id) {
                        for dep in &worker.depends_on {
                            match parent.pipeline_results.get(dep) {
                                Some((p, _)) => inputs.push(p.clone()),
                                None => {
                                    drop(st);
                                    self.item_error(&worker.parent_id, &worker.worker_id, "ffmpeg.no_args");
                                    return;
                                }
                            }
                        }
                    }
                }
                if inputs.is_empty() || ffargs.is_empty() && format.is_empty() {
                    self.item_error(&worker.parent_id, &worker.worker_id, "ffmpeg.no_args");
                    return;
                }
                self.run_ffmpeg_worker(&worker, inputs, ffargs.clone(), format.clone(), mime.clone()).await;
            }
        }
    }

    /// `workers/fetch.ts`
    async fn run_fetch_worker(self: &Arc<Self>, worker: &PipelineItem, url: &str, header_filename: bool) {
        let out_path = self.cache_dir.join(format!("{}.part", worker.worker_id));
        let mut attempts = 0;
        loop {
            match self.fetch_once(worker, url, &out_path, header_filename).await {
                Ok(size) => {
                    self.pipeline_task_done(&worker.parent_id, &worker.worker_id, out_path, size);
                    return;
                }
                Err((code, retry)) => {
                    attempts += 1;
                    if retry && attempts <= 3 {
                        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
                        continue;
                    }
                    let _ = std::fs::remove_file(&out_path);
                    self.item_error(&worker.parent_id, &worker.worker_id, &code);
                    return;
                }
            }
        }
    }

    async fn fetch_once(
        &self,
        worker: &PipelineItem,
        url: &str,
        out_path: &Path,
        header_filename: bool,
    ) -> Result<u64, (String, bool)> {
        // plain proxy downloads may point at redirecting service urls
        let client = if header_filename {
            reqwest::Client::builder()
                .user_agent(format!("cobalt-desktop/{}", env!("CARGO_PKG_VERSION")))
                .redirect(reqwest::redirect::Policy::limited(10))
                .build()
                .unwrap_or_else(|_| self.http.clone())
        } else {
            self.http.clone()
        };
        let resp = client.get(url).send().await.map_err(|e| {
            if e.is_connect() || e.is_timeout() || e.is_request() {
                ("fetch.network_error".to_string(), true)
            } else {
                ("fetch.crashed".to_string(), false)
            }
        })?;
        if !resp.status().is_success() {
            return Err(("fetch.bad_response".into(), true));
        }
        if header_filename {
            let content_type = resp
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.split(';').next().unwrap_or("").trim().to_string())
                .unwrap_or_default();
            let disposition_name = resp
                .headers()
                .get(reqwest::header::CONTENT_DISPOSITION)
                .and_then(|v| v.to_str().ok())
                .and_then(parse_disposition_filename);
            let mut st = self.state.lock();
            if let Some(item) = st.get_mut(&worker.parent_id) {
                if let Some(name) = disposition_name {
                    item.filename = name;
                } else if item.filename.is_empty() || !item.filename.contains('.') {
                    let base = if item.filename.is_empty() { "file".to_string() } else { item.filename.clone() };
                    let ext = extension_for_mime(&content_type)
                        .map(|e| e.to_string())
                        .or_else(|| resp.url().path().rsplit('.').next().filter(|e| e.len() <= 5).map(|e| e.to_string()))
                        .unwrap_or_else(|| "bin".into());
                    item.filename = format!("{base}.{ext}");
                }
                if !content_type.is_empty() && content_type != "application/octet-stream" {
                    item.mime = content_type.clone();
                } else {
                    item.mime = mime_from_filename(&item.filename);
                }
                item.media_type = media_type_from_mime(&item.mime);
            }
        }
        let content_length: Option<u64> = resp
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok());
        let estimated: Option<u64> = resp
            .headers()
            .get("Estimated-Content-Length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok());
        let expected = content_length.or(estimated);

        let mut file = tokio::fs::File::create(out_path).await.map_err(|_| ("fetch.no_file_reader".to_string(), false))?;
        let mut stream = resp.bytes_stream();
        let mut received: u64 = 0;
        let mut last_emit = std::time::Instant::now();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|_| ("fetch.network_error".to_string(), true))?;
            file.write_all(&chunk).await.map_err(|_| ("fetch.no_file_reader".to_string(), false))?;
            received += chunk.len() as u64;
            if last_emit.elapsed().as_millis() > 80 {
                last_emit = std::time::Instant::now();
                let pct = expected.map(|e| ((received as f64 / e.max(1) as f64) * 100.0) as f32).unwrap_or(0.0);
                self.set_progress(
                    &worker.parent_id,
                    &worker.worker_id,
                    WorkerProgress { percentage: pct.min(100.0), size: received },
                );
            }
        }
        file.flush().await.ok();
        drop(file);
        if received == 0 {
            return Err(("fetch.empty_tunnel".into(), true));
        }
        if let Some(cl) = content_length {
            if cl != received {
                return Err(("fetch.corrupted_file".into(), false));
            }
        }
        self.set_progress(&worker.parent_id, &worker.worker_id, WorkerProgress { percentage: 100.0, size: received });
        Ok(received)
    }

    /// `workers/ffmpeg.ts`
    async fn run_ffmpeg_worker(
        self: &Arc<Self>,
        worker: &PipelineItem,
        inputs: Vec<PathBuf>,
        ffargs: Vec<String>,
        format: String,
        mime: String,
    ) {
        let Some(ffmpeg) = self.ffmpeg_path() else {
            self.item_error(&worker.parent_id, &worker.worker_id, "ffmpeg.not_found");
            return;
        };
        // probing just the first file in files array (usually audio) for duration progress
        let probe_file = &inputs[0];
        let streams = crate::ffmpeg::probe_streams(&ffmpeg, probe_file).await;
        match streams {
            Err(code) => {
                self.item_error(&worker.parent_id, &worker.worker_id, code.trim_start_matches("queue."));
                return;
            }
            Ok((has_video, has_audio)) => {
                if !has_video && !has_audio {
                    self.item_error(&worker.parent_id, &worker.worker_id, "ffmpeg.no_input_format");
                    return;
                }
                // edge case: user tries to extract audio from a video without an audio track
                if inputs.len() == 1 && mime.starts_with("audio") && !has_audio {
                    self.item_error(&worker.parent_id, &worker.worker_id, "ffmpeg.no_audio_channel");
                    return;
                }
            }
        }
        let duration = crate::ffmpeg::probe_duration(&ffmpeg, probe_file).await;
        let output = self.cache_dir.join(format!("{}.{}", worker.worker_id, format));
        let parent = worker.parent_id.clone();
        let wid = worker.worker_id.clone();
        let this = self.clone();
        let result = crate::ffmpeg::render(&ffmpeg, &inputs, &ffargs, &output, duration, |pct, size| {
            this.set_progress(&parent, &wid, WorkerProgress { percentage: pct, size });
        })
        .await;
        match result {
            Ok(()) => {
                let size = std::fs::metadata(&output).map(|m| m.len()).unwrap_or(0);
                self.pipeline_task_done(&worker.parent_id, &worker.worker_id, output, size);
            }
            Err(code) => {
                let _ = std::fs::remove_file(&output);
                self.item_error(&worker.parent_id, &worker.worker_id, &code);
            }
        }
    }
}

// --- ffmpeg argument builders, 1:1 with queue.ts ---

fn metadata_args(info: &LocalProcessingResponse) -> Vec<String> {
    const KEYS: &[&str] = &[
        "album", "composer", "genre", "copyright", "title", "artist", "album_artist", "track", "date", "sublanguage",
    ];
    let mut out = Vec::new();
    if let Some(meta) = &info.output.metadata {
        for (name, value) in meta {
            if let (true, Some(v)) = (KEYS.contains(&name.as_str()), value.as_str()) {
                let clean: String = v.chars().filter(|c| !('\u{0}'..='\u{9}').contains(c)).collect();
                if name == "sublanguage" {
                    out.push("-metadata:s:s:0".into());
                    out.push(format!("language={clean}"));
                } else {
                    out.push("-metadata".into());
                    out.push(format!("{name}={clean}"));
                }
            }
        }
    }
    out
}

fn make_remux_args(info: &LocalProcessingResponse) -> Vec<String> {
    let mut ff: Vec<String> = vec!["-c:v".into(), "copy".into()];
    if info.kind == "merge" || info.kind == "remux" {
        ff.push("-c:a".into());
        ff.push("copy".into());
    } else if info.kind == "mute" {
        ff.push("-an".into());
    }
    if info.output.subtitles == Some(true) {
        ff.push("-c:s".into());
        ff.push(if info.output.filename.ends_with(".mp4") { "mov_text".into() } else { "webvtt".into() });
    }
    ff.extend(metadata_args(info));
    ff
}

fn make_audio_args(info: &LocalProcessingResponse) -> Option<Vec<String>> {
    let audio = info.audio.as_ref()?;
    let mut ff: Vec<String> = Vec::new();
    if audio.cover == Some(true) && audio.format == "mp3" {
        ff.extend(["-map", "0", "-map", "1"].iter().map(|s| s.to_string()));
        if audio.crop_cover == Some(true) {
            ff.extend(["-c:v", "mjpeg", "-vf", "scale=-1:720,crop=720:720"].iter().map(|s| s.to_string()));
        } else {
            ff.extend(["-c:v", "copy"].iter().map(|s| s.to_string()));
        }
    } else {
        ff.push("-vn".into());
    }
    if audio.copy {
        ff.extend(["-c:a", "copy"].iter().map(|s| s.to_string()));
    } else {
        ff.push("-b:a".into());
        ff.push(format!("{}k", audio.bitrate));
    }
    ff.extend(metadata_args(info));
    if audio.format == "mp3" && audio.bitrate == "8" {
        ff.extend(["-ar", "12000"].iter().map(|s| s.to_string()));
    }
    if audio.format == "opus" {
        ff.extend(["-vbr", "off"].iter().map(|s| s.to_string()));
    }
    let out_format = if audio.format == "m4a" { "ipod" } else { audio.format.as_str() };
    ff.push("-f".into());
    ff.push(out_format.into());
    Some(ff)
}

fn make_gif_args() -> Vec<String> {
    [
        "-vf",
        "scale=-1:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse",
        "-loop",
        "0",
        "-f",
        "gif",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

fn parse_disposition_filename(header: &str) -> Option<String> {
    // filename*=UTF-8''encoded  or  filename="name"
    for part in header.split(';') {
        let part = part.trim();
        if let Some(v) = part.strip_prefix("filename*=") {
            let v = v.trim_matches('"');
            let v = v.split("''").nth(1).unwrap_or(v);
            return Some(percent_decode(v));
        }
    }
    for part in header.split(';') {
        let part = part.trim();
        if let Some(v) = part.strip_prefix("filename=") {
            return Some(v.trim_matches('"').to_string());
        }
    }
    None
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() && s.is_char_boundary(i + 1) && s.is_char_boundary(i + 3) {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

pub fn extension_for_mime(mime: &str) -> Option<&'static str> {
    Some(match mime {
        "video/mp4" => "mp4",
        "video/webm" => "webm",
        "video/x-matroska" => "mkv",
        "video/quicktime" => "mov",
        "audio/mpeg" => "mp3",
        "audio/mp4" | "audio/x-m4a" => "m4a",
        "audio/ogg" => "ogg",
        "audio/opus" => "opus",
        "audio/wav" | "audio/x-wav" => "wav",
        "audio/flac" => "flac",
        "image/gif" => "gif",
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        _ => return None,
    })
}

pub fn mime_from_filename(name: &str) -> String {
    let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "mp4" | "m4v" => "video/mp4",
        "webm" => "video/webm",
        "mkv" => "video/x-matroska",
        "mov" => "video/quicktime",
        "avi" => "video/x-msvideo",
        "mp3" => "audio/mpeg",
        "m4a" => "audio/mp4",
        "ogg" | "oga" => "audio/ogg",
        "opus" => "audio/opus",
        "wav" => "audio/wav",
        "flac" => "audio/flac",
        "aac" => "audio/aac",
        "gif" => "image/gif",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "heic" => "image/heic",
        _ => "application/octet-stream",
    }
    .to_string()
}

/// `formatFileSize()` from util.ts
pub fn format_file_size(size: u64) -> String {
    let mut size = size as f64;
    let mut units = vec!["G", "M", "K", ""];
    while size >= 1024.0 && units.len() > 1 {
        size /= 1024.0;
        units.pop();
    }
    format!("{:.2} {}B", size, units.last().unwrap_or(&""))
}
