use crate::app::config::{PakeConfig, WindowConfig, MAIN_WINDOW_LABEL};
use crate::util::{
    check_file_or_append, get_data_dir, get_download_message_with_lang, show_toast, MessageType,
};
use std::{
    collections::HashMap,
    path::PathBuf,
    str::FromStr,
    sync::{atomic::{AtomicU32, Ordering}, Mutex},
};
use tauri::{
    webview::{DownloadEvent, NewWindowFeatures, NewWindowResponse},
    AppHandle, Config, Manager, Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

use tauri::Theme;

#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;

#[cfg(target_os = "windows")]
fn build_proxy_browser_arg(url: &Url) -> Option<String> {
    let host = url.host_str()?;
    let scheme = url.scheme();
    let port = url.port().or_else(|| match scheme {
        "http" => Some(80),
        "socks5" => Some(1080),
        _ => None,
    })?;

    match scheme {
        "http" | "socks5" => Some(format!("--proxy-server={scheme}://{host}:{port}")),
        _ => None,
    }
}

pub struct MultiWindowState {
    pub pake_config: PakeConfig,
    pub tauri_config: Config,
    next_popup_index: AtomicU32,
    next_route_index: AtomicU32,
    route_instance_counters: Mutex<HashMap<String, u32>>,
}

impl MultiWindowState {
    pub fn new(pake_config: PakeConfig, tauri_config: Config) -> Self {
        Self {
            pake_config,
            tauri_config,
            next_popup_index: AtomicU32::new(0),
            next_route_index: AtomicU32::new(0),
            route_instance_counters: Mutex::new(HashMap::new()),
        }
    }

    fn next_popup_label(&self) -> String {
        let index = self.next_popup_index.fetch_add(1, Ordering::Relaxed) + 1;
        format!("pake-{index}")
    }

    fn next_route_instance_label(&self, template_label: &str) -> String {
        let mut counters = self
            .route_instance_counters
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let count = counters.entry(template_label.to_string()).or_insert(0);
        *count += 1;
        format!("{template_label}-{count}")
    }

    fn route_templates(&self) -> Vec<WindowConfig> {
        self.pake_config
            .route_window_templates()
            .into_iter()
            .cloned()
            .collect()
    }

    fn next_route_template(&self) -> Option<WindowConfig> {
        let templates = self.route_templates();
        if templates.is_empty() {
            return None;
        }

        let start = self.next_route_index.fetch_add(1, Ordering::Relaxed) as usize;
        Some(templates[start % templates.len()].clone())
    }

    fn route_template_label(template: &WindowConfig) -> &str {
        template
            .label
            .as_deref()
            .filter(|value| !value.is_empty())
            .unwrap_or(MAIN_WINDOW_LABEL)
    }
}

pub fn set_window(
    app: &AppHandle,
    config: &PakeConfig,
    tauri_config: &Config,
) -> tauri::Result<WebviewWindow> {
    let main_label = config.main_window_label();
    let window_config = config.windows.first().ok_or_else(|| {
        tauri::Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "pake.json must define at least one window configuration",
        ))
    })?;
    build_window_with_config(app, config, tauri_config, main_label, window_config)
}

pub fn open_window_by_label(app: &AppHandle, label: &str) -> tauri::Result<WebviewWindow> {
    if let Some(existing) = app.get_webview_window(label) {
        let _ = existing.unminimize();
        let _ = existing.show();
        let _ = existing.set_focus();
        return Ok(existing);
    }

    let state = app.state::<MultiWindowState>();
    let window_config = state
        .pake_config
        .resolve_window_config(label)
        .map_err(|error| {
            tauri::Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                error,
            ))
        })?;

    let window = build_window_with_config(
        app,
        &state.pake_config,
        &state.tauri_config,
        label,
        window_config,
    )?;
    let _ = window.show();
    let _ = window.set_focus();
    Ok(window)
}

pub fn open_route_template_window(
    app: &AppHandle,
    template_label: &str,
) -> tauri::Result<WebviewWindow> {
    let state = app.state::<MultiWindowState>();
    let window_config = state
        .pake_config
        .window_config_by_label(template_label)
        .map_err(|error| {
            tauri::Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                error,
            ))
        })?;
    let instance_label = state.next_route_instance_label(template_label);
    let window = build_window_with_config(
        app,
        &state.pake_config,
        &state.tauri_config,
        &instance_label,
        window_config,
    )?;
    let _ = window.show();
    let _ = window.set_focus();
    Ok(window)
}

pub fn open_route_template_window_safe(app: &AppHandle, template_label: &str) {
    #[cfg(target_os = "windows")]
    {
        let app_handle = app.clone();
        let template_label = template_label.to_string();
        std::thread::spawn(move || {
            let _ = open_route_template_window(&app_handle, &template_label);
        });
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = open_route_template_window(app, template_label);
    }
}

#[allow(dead_code)]
pub fn open_window_by_label_safe(app: &AppHandle, label: &str) {
    #[cfg(target_os = "windows")]
    {
        let app_handle = app.clone();
        let label = label.to_string();
        std::thread::spawn(move || {
            let _ = open_window_by_label(&app_handle, &label);
        });
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = open_window_by_label(app, label);
    }
}

pub fn open_additional_window(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    let state = app.state::<MultiWindowState>();

    if state.pake_config.has_route_templates() {
        if let Some(template) = state.next_route_template() {
            let label = MultiWindowState::route_template_label(&template);
            return open_route_template_window(app, label);
        }
    }

    let window_config = state.pake_config.windows.first().ok_or_else(|| {
        tauri::Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "pake.json must define at least one window configuration",
        ))
    })?;
    let label = state.next_popup_label();
    build_window_with_config(
        app,
        &state.pake_config,
        &state.tauri_config,
        &label,
        window_config,
    )
}

struct WindowBuildOptions<'a> {
    label: &'a str,
    url: WebviewUrl,
    visible: bool,
    new_window_features: Option<NewWindowFeatures>,
}

fn open_requested_window(
    app: &AppHandle,
    config: &PakeConfig,
    tauri_config: &Config,
    target_url: Url,
    features: NewWindowFeatures,
) -> tauri::Result<WebviewWindow> {
    let state = app.state::<MultiWindowState>();
    let label = state.next_popup_label();
    let popup_config = config
        .windows
        .first()
        .cloned()
        .ok_or_else(|| {
            tauri::Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "pake.json must define at least one window configuration",
            ))
        })?;
    let window = build_window(
        app,
        config,
        tauri_config,
        &popup_config,
        WindowBuildOptions {
            label: &label,
            url: WebviewUrl::External(target_url.clone()),
            visible: true,
            new_window_features: Some(features),
        },
    )?;

    let title = target_url.host_str().unwrap_or(target_url.as_str());
    let _ = window.set_title(title);
    let _ = window.set_focus();

    Ok(window)
}

pub fn open_additional_window_safe(app: &AppHandle) {
    #[cfg(target_os = "windows")]
    {
        let app_handle = app.clone();
        std::thread::spawn(move || {
            if let Ok(window) = open_additional_window(&app_handle) {
                let _ = window.show();
                let _ = window.set_focus();
            }
        });
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(window) = open_additional_window(app) {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

fn webview_url_for_config(window_config: &WindowConfig) -> tauri::Result<WebviewUrl> {
    match window_config.url_type.as_str() {
        "web" => {
            let parsed = window_config.url.parse::<Url>().map_err(|err| {
                tauri::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "Invalid 'web' url '{}' in pake.json: {err}",
                        window_config.url
                    ),
                ))
            })?;
            if parsed.scheme() == "http" || parsed.scheme() == "https" {
                Ok(WebviewUrl::External(parsed))
            } else {
                Ok(WebviewUrl::App(PathBuf::from(parsed.path())))
            }
        }
        "local" => Ok(WebviewUrl::App(PathBuf::from(&window_config.url))),
        other => Err(tauri::Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("url_type must be 'web' or 'local', got '{other}'"),
        ))),
    }
}

fn build_window_with_config(
    app: &AppHandle,
    config: &PakeConfig,
    tauri_config: &Config,
    label: &str,
    window_config: &WindowConfig,
) -> tauri::Result<WebviewWindow> {
    let url = webview_url_for_config(window_config)?;

    build_window(
        app,
        config,
        tauri_config,
        window_config,
        WindowBuildOptions {
            label,
            url,
            visible: false,
            new_window_features: None,
        },
    )
}

fn build_window(
    app: &AppHandle,
    config: &PakeConfig,
    tauri_config: &Config,
    window_config: &WindowConfig,
    opts: WindowBuildOptions,
) -> tauri::Result<WebviewWindow> {
    let WindowBuildOptions {
        label,
        url,
        visible,
        new_window_features,
    } = opts;
    let package_name = tauri_config
        .product_name
        .clone()
        .unwrap_or_else(|| "pake".to_string());
    let _data_dir = get_data_dir(app, package_name).map_err(tauri::Error::Io)?;

    let user_agent = config.user_agent.get();

    let config_script = format!(
        "window.pakeConfig = {}",
        serde_json::to_string(window_config).unwrap_or_else(|_| "{}".to_string())
    );

    // Platform-specific title: macOS prefers empty, others fallback to product name
    let effective_title = window_config.title.as_deref().unwrap_or_else(|| {
        if cfg!(target_os = "macos") {
            ""
        } else {
            tauri_config.product_name.as_deref().unwrap_or("")
        }
    });

    let mut window_builder = WebviewWindowBuilder::new(app, label, url)
        .title(effective_title)
        .visible(visible)
        .user_agent(user_agent)
        .resizable(window_config.resizable)
        .maximized(window_config.maximize);

    #[cfg(target_os = "windows")]
    {
        let scale_factor = app
            .primary_monitor()
            .ok()
            .flatten()
            .map(|m| m.scale_factor())
            .unwrap_or(1.0);
        let logical_width = window_config.width / scale_factor;
        let logical_height = window_config.height / scale_factor;
        window_builder = window_builder.inner_size(logical_width, logical_height);
    }

    #[cfg(not(target_os = "windows"))]
    {
        window_builder = window_builder.inner_size(window_config.width, window_config.height);
    }

    window_builder = window_builder
        .always_on_top(window_config.always_on_top)
        .incognito(window_config.incognito);

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        window_builder = window_builder.fullscreen(window_config.fullscreen);
    }

    if window_config.min_width > 0.0 || window_config.min_height > 0.0 {
        let min_w = if window_config.min_width > 0.0 {
            window_config.min_width
        } else {
            window_config.width
        };
        let min_h = if window_config.min_height > 0.0 {
            window_config.min_height
        } else {
            window_config.height
        };
        window_builder = window_builder.min_inner_size(min_w, min_h);
    }

    if !window_config.enable_drag_drop {
        window_builder = window_builder.disable_drag_drop_handler();
    }

    if window_config.new_window {
        let app_handle = app.clone();
        let popup_config = config.clone();
        let popup_tauri_config = tauri_config.clone();
        window_builder = window_builder.on_new_window(move |target_url, features| {
            match open_requested_window(
                &app_handle,
                &popup_config,
                &popup_tauri_config,
                target_url,
                features,
            ) {
                Ok(window) => NewWindowResponse::Create { window },
                Err(error) => {
                    eprintln!("[Pake] Failed to open requested window: {error}");
                    NewWindowResponse::Deny
                }
            }
        });
    }

    // Add initialization scripts. Order matters: pakeConfig must land before
    // any script that reads it (e.g. fullscreen polyfill checks for an opt-out
    // flag), and toast must register `window.pakeToast` before Rust code
    // calls show_toast().
    window_builder = window_builder
        .initialization_script(&config_script)
        .initialization_script(include_str!("../inject/find.js"))
        .initialization_script(include_str!("../inject/toast.js"))
        .initialization_script(include_str!("../inject/fullscreen.js"))
        .initialization_script(include_str!("../inject/event.js"))
        .initialization_script(include_str!("../inject/style.js"))
        .initialization_script(include_str!("../inject/theme_refresh.js"))
        .initialization_script(include_str!("../inject/auth.js"))
        .initialization_script(include_str!("../inject/custom.js"));

    #[cfg(target_os = "windows")]
    let mut windows_browser_args = String::from("--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --disable-blink-features=AutomationControlled");

    #[cfg(target_os = "linux")]
    let mut linux_browser_args = String::from("--disable-blink-features=AutomationControlled");

    if window_config.ignore_certificate_errors {
        #[cfg(target_os = "windows")]
        {
            windows_browser_args.push_str(" --ignore-certificate-errors");
        }

        #[cfg(target_os = "linux")]
        {
            linux_browser_args.push_str(" --ignore-certificate-errors");
        }

        #[cfg(target_os = "macos")]
        {
            window_builder = window_builder.additional_browser_args("--ignore-certificate-errors");
        }
    }

    if window_config.enable_wasm {
        #[cfg(target_os = "windows")]
        {
            windows_browser_args.push_str(" --enable-features=SharedArrayBuffer");
            windows_browser_args.push_str(" --enable-unsafe-webgpu");
        }

        #[cfg(target_os = "linux")]
        {
            linux_browser_args.push_str(" --enable-features=SharedArrayBuffer");
            linux_browser_args.push_str(" --enable-unsafe-webgpu");
        }

        #[cfg(target_os = "macos")]
        {
            window_builder = window_builder
                .additional_browser_args("--enable-features=SharedArrayBuffer")
                .additional_browser_args("--enable-unsafe-webgpu");
        }
    }

    let mut parsed_proxy_url: Option<Url> = None;

    // Default to following the system theme (None), only force dark when explicitly set.
    // Computed once; the matching platform block below is the sole consumer.
    let theme = if window_config.dark_mode {
        Some(Theme::Dark)
    } else {
        None // Follow system theme
    };

    // Platform-specific configuration must be set before proxy on Windows/Linux
    #[cfg(target_os = "macos")]
    {
        let title_bar_style = if window_config.hide_title_bar {
            TitleBarStyle::Overlay
        } else {
            TitleBarStyle::Visible
        };
        window_builder = window_builder.title_bar_style(title_bar_style);
        window_builder = window_builder.theme(theme);
    }

    // Windows and Linux: set data_directory before proxy_url
    #[cfg(not(target_os = "macos"))]
    {
        window_builder = window_builder.data_directory(_data_dir).theme(theme);

        if !config.proxy_url.is_empty() {
            if let Ok(proxy_url) = Url::from_str(&config.proxy_url) {
                parsed_proxy_url = Some(proxy_url.clone());
                #[cfg(target_os = "windows")]
                {
                    if let Some(arg) = build_proxy_browser_arg(&proxy_url) {
                        windows_browser_args.push(' ');
                        windows_browser_args.push_str(&arg);
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            window_builder = window_builder.additional_browser_args(&windows_browser_args);
        }

        #[cfg(target_os = "linux")]
        {
            window_builder = window_builder.additional_browser_args(&linux_browser_args);
        }
    }

    // Set proxy after platform-specific configs (required for Windows/Linux)
    if parsed_proxy_url.is_none() && !config.proxy_url.is_empty() {
        if let Ok(proxy_url) = Url::from_str(&config.proxy_url) {
            parsed_proxy_url = Some(proxy_url);
        }
    }

    if let Some(proxy_url) = parsed_proxy_url {
        window_builder = window_builder.proxy_url(proxy_url);
        #[cfg(debug_assertions)]
        println!("Proxy configured: {}", config.proxy_url);
    }

    if let Some(features) = new_window_features {
        // Reuse only opener-provided position/size on macOS; sharing the opener
        // WKWebViewConfiguration triggers duplicate WKScriptMessageHandler
        // registrations on macOS 26+ and crashes the app (issue #1194).
        #[cfg(target_os = "macos")]
        {
            if let Some(position) = features.position() {
                window_builder = window_builder.position(position.x, position.y);
            }

            if let Some(size) = features.size() {
                window_builder = window_builder.inner_size(size.width, size.height);
            }

            window_builder = window_builder.focused(true);
        }

        #[cfg(not(target_os = "macos"))]
        {
            window_builder = window_builder.window_features(features).focused(true);
        }
    }

    // Capture webview-initiated downloads (blob:, data:, Content-Disposition,
    // etc.) and write them to the OS Downloads folder. This is essential for
    // sites with a strict Content-Security-Policy (e.g. Gemini): their
    // `connect-src` blocks Tauri's IPC origin, so downloads cannot be routed
    // through the JS bridge, and downloads triggered from a sandboxed iframe
    // can't reach the IPC either. Letting the browser download natively and
    // catching it here is independent of the page CSP and the IPC channel.
    {
        let download_handle = app.clone();
        window_builder = window_builder.on_download(move |_webview, event| match event {
            DownloadEvent::Requested { url, destination } => {
                match download_handle.path().download_dir() {
                    Ok(download_dir) => {
                        let filename = destination
                            .file_name()
                            .map(|name| name.to_string_lossy().to_string())
                            .filter(|name| !name.is_empty())
                            .or_else(|| {
                                url.path_segments()
                                    .and_then(|mut segments| segments.next_back())
                                    .map(|segment| segment.to_string())
                                    .filter(|segment| !segment.is_empty())
                            })
                            .unwrap_or_else(|| "download".to_string());

                        let target = download_dir.join(filename);
                        if let Some(path_str) = target.to_str() {
                            *destination = PathBuf::from(check_file_or_append(path_str));
                        }
                    }
                    Err(error) => {
                        eprintln!("[Pake] Failed to resolve download dir: {error}");
                    }
                }
                true
            }
            DownloadEvent::Finished {
                url: _,
                path: _,
                success,
            } => {
                if let Some(window) = download_handle.get_webview_window(MAIN_WINDOW_LABEL) {
                    let message_type = if success {
                        MessageType::Success
                    } else {
                        MessageType::Failure
                    };
                    show_toast(&window, &get_download_message_with_lang(message_type, None));
                }
                true
            }
            _ => true,
        });
    }

    window_builder = window_builder.on_navigation(|_| true);

    window_builder.build()
}

#[cfg(test)]
mod window_config_tests {
    use super::*;
    use crate::app::config::{FunctionON, UserAgent};

    fn sample_window(label: Option<&str>, url: &str) -> WindowConfig {
        WindowConfig {
            url: url.to_string(),
            hide_title_bar: false,
            fullscreen: false,
            maximize: false,
            width: 1200.0,
            height: 780.0,
            resizable: true,
            url_type: "web".to_string(),
            always_on_top: false,
            dark_mode: false,
            disabled_web_shortcuts: false,
            activation_shortcut: String::new(),
            hide_on_close: true,
            incognito: false,
            title: None,
            enable_wasm: false,
            enable_drag_drop: false,
            new_window: false,
            label: label.map(str::to_string),
            start_to_tray: false,
            force_internal_navigation: false,
            internal_url_regex: String::new(),
            enable_find: false,
            zoom: 100,
            min_width: 0.0,
            min_height: 0.0,
            ignore_certificate_errors: false,
        }
    }

    fn sample_config(windows: Vec<WindowConfig>) -> PakeConfig {
        PakeConfig {
            windows,
            user_agent: UserAgent {
                macos: String::new(),
                linux: String::new(),
                windows: String::new(),
            },
            system_tray: FunctionON {
                macos: false,
                linux: false,
                windows: false,
            },
            system_tray_path: String::new(),
            proxy_url: String::new(),
            multi_instance: false,
            multi_window: true,
        }
    }

    #[test]
    fn window_config_by_label_returns_matching_config() {
        let config = sample_config(vec![
            sample_window(Some("pake"), "https://my-web.com/dashboard"),
            sample_window(Some("camera"), "https://my-web.com/camera"),
        ]);

        let camera = config.window_config_by_label("camera").unwrap();
        assert_eq!(camera.url, "https://my-web.com/camera");
    }

    #[test]
    fn window_config_by_label_rejects_unknown_label() {
        let config = sample_config(vec![sample_window(
            Some("pake"),
            "https://my-web.com/dashboard",
        )]);

        assert!(config.window_config_by_label("unknown").is_err());
    }

    #[test]
    fn validate_window_labels_rejects_duplicates() {
        let config = sample_config(vec![
            sample_window(Some("camera"), "https://my-web.com/camera"),
            sample_window(Some("camera"), "https://my-web.com/monitor"),
        ]);

        assert!(config.validate_window_labels().is_err());
    }

    #[test]
    fn route_window_templates_excludes_main_window() {
        let config = sample_config(vec![
            sample_window(Some("pake"), "https://my-web.com/dashboard"),
            sample_window(Some("camera"), "https://my-web.com/camera"),
        ]);

        let templates = config.route_window_templates();
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].label.as_deref(), Some("camera"));
    }

    #[test]
    fn resolve_window_config_maps_route_instances_to_template() {
        let config = sample_config(vec![
            sample_window(Some("pake"), "https://my-web.com/"),
            sample_window(Some("live"), "https://my-web.com/live"),
        ]);

        let live = config.resolve_window_config("live-2").unwrap();
        assert_eq!(live.url, "https://my-web.com/live");
        assert!(config.is_route_instance_label("live-3"));
        assert!(!config.is_route_instance_label("pake-1"));
    }

    #[test]
    fn next_route_template_round_robins_templates() {
        let state = MultiWindowState::new(
            sample_config(vec![
                sample_window(Some("pake"), "https://my-web.com/"),
                sample_window(Some("live"), "https://my-web.com/live"),
            ]),
            Config::default(),
        );

        let templates = state.route_templates();
        assert_eq!(templates.len(), 1);
        assert_eq!(
            MultiWindowState::route_template_label(&templates[0]),
            "live"
        );
        assert_eq!(state.next_route_instance_label("live"), "live-1");
        assert_eq!(state.next_route_instance_label("live"), "live-2");
    }
}

#[cfg(all(test, target_os = "windows"))]
mod proxy_arg_tests {
    use super::*;

    fn parse(url: &str) -> Url {
        Url::from_str(url).unwrap()
    }

    #[test]
    fn http_url_with_explicit_port() {
        let arg = build_proxy_browser_arg(&parse("http://127.0.0.1:7890")).unwrap();
        assert_eq!(arg, "--proxy-server=http://127.0.0.1:7890");
    }

    #[test]
    fn http_url_uses_default_port_when_missing() {
        let arg = build_proxy_browser_arg(&parse("http://proxy.local")).unwrap();
        assert_eq!(arg, "--proxy-server=http://proxy.local:80");
    }

    #[test]
    fn socks5_url_uses_default_port_when_missing() {
        let arg = build_proxy_browser_arg(&parse("socks5://proxy.local")).unwrap();
        assert_eq!(arg, "--proxy-server=socks5://proxy.local:1080");
    }

    #[test]
    fn https_scheme_is_not_supported_yet() {
        // https proxies fall back to platform proxy_url; we only emit a CLI arg
        // for http/socks5 today.
        assert!(build_proxy_browser_arg(&parse("https://proxy.local:8443")).is_none());
    }
}
