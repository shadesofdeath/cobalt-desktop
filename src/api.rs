//! cobalt api client. mirrors `web/src/lib/api/*` and `docs/api.md`.

use crate::settings::CobaltSettings;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// `CobaltSaveRequestBody` - only keys that differ from the defaults are sent,
/// exactly like `lazySettingGetter` does in the web app.
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SaveRequest {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_processing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub always_proxy: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle_lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_metadata: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_bitrate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tiktok_full_audio: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub youtube_dub_lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub youtube_better_audio: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_quality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub youtube_video_codec: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub youtube_video_container: Option<String>,
    #[serde(rename = "youtubeHLS", skip_serializing_if = "Option::is_none")]
    pub youtube_hls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_h265: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub convert_gif: Option<bool>,
}

impl SaveRequest {
    pub fn from_settings(url: &str, settings: &CobaltSettings) -> Self {
        let d = crate::settings::Save::default();
        let s = &settings.save;
        fn lazy<T: PartialEq + Clone>(value: &T, default: &T) -> Option<T> {
            if value != default { Some(value.clone()) } else { None }
        }
        Self {
            url: url.to_string(),
            // not lazy in the web app either ("default depends on device capabilities")
            local_processing: Some(s.local_processing.clone()),
            always_proxy: lazy(&s.always_proxy, &d.always_proxy),
            download_mode: lazy(&s.download_mode, &d.download_mode),
            subtitle_lang: lazy(&s.subtitle_lang, &d.subtitle_lang),
            filename_style: lazy(&s.filename_style, &d.filename_style),
            disable_metadata: lazy(&s.disable_metadata, &d.disable_metadata),
            audio_format: lazy(&s.audio_format, &d.audio_format),
            audio_bitrate: lazy(&s.audio_bitrate, &d.audio_bitrate),
            tiktok_full_audio: lazy(&s.tiktok_full_audio, &d.tiktok_full_audio),
            youtube_dub_lang: lazy(&s.youtube_dub_lang, &d.youtube_dub_lang),
            youtube_better_audio: lazy(&s.youtube_better_audio, &d.youtube_better_audio),
            video_quality: lazy(&s.video_quality, &d.video_quality),
            youtube_video_codec: lazy(&s.youtube_video_codec, &d.youtube_video_codec),
            youtube_video_container: lazy(&s.youtube_video_container, &d.youtube_video_container),
            // deprecated in the web app (only sent when ENABLE_DEPRECATED_YOUTUBE_HLS)
            youtube_hls: None,
            allow_h265: lazy(&s.allow_h265, &d.allow_h265),
            convert_gif: lazy(&s.convert_gif, &d.convert_gif),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct ErrorContext {
    pub service: Option<String>,
    pub limit: Option<f64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ApiError {
    pub code: String,
    #[serde(default)]
    pub context: Option<ErrorContext>,
}

impl ApiError {
    pub fn new(code: &str) -> Self {
        Self { code: code.to_string(), context: None }
    }
    pub fn message(&self) -> String {
        let ctx = self.context.clone().unwrap_or_default();
        crate::i18n::t_error(&self.code, ctx.service.as_deref(), ctx.limit)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct PickerItem {
    #[serde(rename = "type")]
    pub kind: String,
    pub url: String,
    pub thumb: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct OutputInfo {
    #[serde(rename = "type")]
    pub mime: String,
    pub filename: String,
    #[serde(default)]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub subtitles: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct AudioInfo {
    pub copy: bool,
    pub format: String,
    pub bitrate: String,
    #[serde(default)]
    pub cover: Option<bool>,
    #[serde(default, rename = "cropCover")]
    pub crop_cover: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LocalProcessingResponse {
    #[serde(rename = "type")]
    pub kind: String,
    pub service: String,
    pub tunnel: Vec<String>,
    pub output: OutputInfo,
    #[serde(default)]
    pub audio: Option<AudioInfo>,
    #[serde(default, rename = "isHLS")]
    pub is_hls: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum ApiResponse {
    Error { error: ApiError },
    Picker {
        picker: Vec<PickerItem>,
        #[serde(default)]
        audio: Option<String>,
        #[serde(default, rename = "audioFilename")]
        audio_filename: Option<String>,
    },
    Redirect { url: String, filename: String },
    Tunnel { url: String, filename: String },
    LocalProcessing(LocalProcessingResponse),
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServerInfoCobalt {
    pub version: String,
    pub url: String,
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde(rename = "turnstileSitekey")]
    pub turnstile_sitekey: Option<String>,
    #[serde(default)]
    pub services: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServerInfoGit {
    pub commit: String,
    pub branch: String,
    pub remote: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServerInfo {
    pub cobalt: ServerInfoCobalt,
    pub git: ServerInfoGit,
}

#[derive(Clone, Debug)]
pub struct CachedServerInfo {
    pub info: ServerInfo,
    pub origin: String,
}

#[derive(Clone, Debug, Deserialize)]
struct SessionResponse {
    token: Option<String>,
    exp: Option<u64>,
    status: Option<String>,
    error: Option<ApiError>,
}

#[derive(Clone, Debug)]
struct Session {
    token: String,
    exp_at: u64,
}

pub struct Client {
    http: reqwest::Client,
    server_info: parking_lot::Mutex<Option<CachedServerInfo>>,
    session: parking_lot::Mutex<Option<Session>>,
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub type SharedClient = Arc<Client>;

impl Client {
    pub fn new() -> SharedClient {
        let http = reqwest::Client::builder()
            .user_agent(format!("cobalt-desktop/{}", env!("CARGO_PKG_VERSION")))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("http client");
        Arc::new(Self {
            http,
            server_info: parking_lot::Mutex::new(None),
            session: parking_lot::Mutex::new(None),
        })
    }

    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    pub fn cached_server_info(&self) -> Option<CachedServerInfo> {
        self.server_info.lock().clone()
    }

    pub fn clear_server_info(&self) {
        *self.server_info.lock() = None;
        *self.session.lock() = None;
    }

    /// `getServerInfo()` - GET / , cached per origin
    pub async fn server_info(&self, api: &str) -> Result<CachedServerInfo, ApiError> {
        if let Some(cache) = self.server_info.lock().clone() {
            if cache.origin == api {
                return Ok(cache);
            }
        }
        let resp = self
            .http
            .get(format!("{api}/"))
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() { ApiError::new("error.api.timed_out") } else { ApiError::new("error.api.unreachable") }
            })?;
        let value: serde_json::Value = resp.json().await.map_err(|_| ApiError::new("error.api.unreachable"))?;
        if value.get("status").and_then(|s| s.as_str()) == Some("error") {
            let err: ApiError = serde_json::from_value(value["error"].clone())
                .unwrap_or_else(|_| ApiError::new("error.api.unreachable"));
            return Err(err);
        }
        let info: ServerInfo = serde_json::from_value(value).map_err(|_| ApiError::new("error.api.unreachable"))?;
        let cached = CachedServerInfo { info, origin: api.to_string() };
        *self.server_info.lock() = Some(cached.clone());
        Ok(cached)
    }

    /// POST /session - only works when the instance doesn't require turnstile
    async fn session(&self, api: &str) -> Result<String, ApiError> {
        if let Some(s) = self.session.lock().clone() {
            if s.exp_at > now_secs() + 2 {
                return Ok(s.token);
            }
        }
        let resp = self
            .http
            .post(format!("{api}/session"))
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() { ApiError::new("error.api.timed_out") } else { ApiError::new("error.api.unreachable") }
            })?;
        let body: SessionResponse = resp.json().await.map_err(|_| ApiError::new("error.api.unreachable"))?;
        if body.status.as_deref() == Some("error") {
            return Err(body.error.unwrap_or_else(|| ApiError::new("error.api.unreachable")));
        }
        let token = body.token.ok_or_else(|| ApiError::new("error.api.unknown_response"))?;
        let exp = body.exp.unwrap_or(0);
        *self.session.lock() = Some(Session { token: token.clone(), exp_at: now_secs() + exp });
        Ok(token)
    }

    /// `getAuthorization()` from api.ts, adapted for a desktop without turnstile
    async fn authorization(&self, api: &str, settings: &CobaltSettings, info: &ServerInfo) -> Result<Option<String>, ApiError> {
        let p = &settings.processing;
        if p.enable_custom_api_key && !p.custom_api_key.is_empty() {
            return Ok(Some(format!("Api-Key {}", p.custom_api_key)));
        }
        if info.cobalt.turnstile_sitekey.is_none() {
            return Ok(None);
        }
        // turnstile is enabled: a jwt session is required. the desktop can't solve
        // the captcha, but some instances still issue tokens without a solution.
        match self.session(api).await {
            Ok(token) => Ok(Some(format!("Bearer {token}"))),
            Err(e) => {
                if e.code == "error.api.auth.not_configured" {
                    Ok(None)
                } else if e.code.starts_with("error.api.auth.turnstile") {
                    Err(ApiError::new("error.api.turnstile_desktop"))
                } else {
                    Err(e)
                }
            }
        }
    }

    /// POST / - the main processing request (`API.request` in api.ts)
    pub async fn request(&self, settings: &CobaltSettings, body: &SaveRequest) -> Result<ApiResponse, ApiError> {
        let api = settings.api_url();
        let info = self.server_info(&api).await?;
        self.request_inner(&api, settings, &info.info, body, false).await
    }

    async fn request_inner(
        &self,
        api: &str,
        settings: &CobaltSettings,
        info: &ServerInfo,
        body: &SaveRequest,
        just_retried: bool,
    ) -> Result<ApiResponse, ApiError> {
        let auth = self.authorization(api, settings, info).await?;
        let mut req = self
            .http
            .post(format!("{api}/"))
            .timeout(Duration::from_secs(20))
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(body);
        if let Some(a) = &auth {
            req = req.header("Authorization", a);
        }
        let resp = req.send().await.map_err(|e| {
            if e.is_timeout() { ApiError::new("error.api.timed_out") } else { ApiError::new("error.api.unreachable") }
        })?;
        let text = resp.text().await.map_err(|_| ApiError::new("error.api.unknown_response"))?;
        let parsed: ApiResponse = serde_json::from_str(&text).map_err(|_| ApiError::new("error.api.unknown_response"))?;

        if let ApiResponse::Error { error } = &parsed {
            if error.code == "error.api.auth.jwt.invalid" && !just_retried {
                *self.session.lock() = None;
                return Box::pin(self.request_inner(api, settings, info, body, true)).await;
            }
        }
        Ok(parsed)
    }

    /// `probeCobaltTunnel` - GET tunnel&p=1 must return 200
    pub async fn probe_tunnel(&self, url: &str) -> bool {
        let probe = format!("{url}&p=1");
        match self.http.get(&probe).timeout(Duration::from_secs(15)).send().await {
            Ok(r) => r.status().as_u16() == 200,
            Err(_) => false,
        }
    }
}

/// the same regex-ish check as `validLink()` in Omnibox.svelte
pub fn valid_link(text: &str) -> bool {
    match url::Url::parse(text.trim()) {
        Ok(u) => matches!(u.scheme(), "http" | "https") && u.host().is_some(),
        Err(_) => false,
    }
}

/// `pastedData.match(/https?:\/\/[^\s]+/g)[0].split('，')[0]`
pub fn extract_link(text: &str) -> Option<String> {
    let re = regex::Regex::new(r"https?://[^\s]+").ok()?;
    let m = re.find(text)?;
    Some(m.as_str().split('，').next().unwrap_or("").to_string())
}
