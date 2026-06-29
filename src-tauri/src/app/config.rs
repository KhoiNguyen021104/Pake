use serde::{Deserialize, Serialize};

pub const MAIN_WINDOW_LABEL: &str = "pake";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WindowConfig {
    pub url: String,
    pub hide_title_bar: bool,
    pub fullscreen: bool,
    pub maximize: bool,
    pub width: f64,
    pub height: f64,
    pub resizable: bool,
    pub url_type: String,
    pub always_on_top: bool,
    pub dark_mode: bool,
    pub disabled_web_shortcuts: bool,
    pub activation_shortcut: String,
    pub hide_on_close: bool,
    pub incognito: bool,
    pub title: Option<String>,
    pub enable_wasm: bool,
    pub enable_drag_drop: bool,
    #[serde(default)]
    pub new_window: bool,
    #[serde(default)]
    pub label: Option<String>,
    pub start_to_tray: bool,
    #[serde(default)]
    pub force_internal_navigation: bool,
    #[serde(default)]
    pub internal_url_regex: String,
    #[serde(default)]
    pub enable_find: bool,
    #[serde(default = "default_zoom")]
    pub zoom: u32,
    #[serde(default)]
    pub min_width: f64,
    #[serde(default)]
    pub min_height: f64,
    #[serde(default)]
    pub ignore_certificate_errors: bool,
}

fn default_zoom() -> u32 {
    100
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlatformSpecific<T> {
    pub macos: T,
    pub linux: T,
    pub windows: T,
}

impl<T> PlatformSpecific<T> {
    pub const fn get(&self) -> &T {
        #[cfg(target_os = "macos")]
        let platform = &self.macos;
        #[cfg(target_os = "linux")]
        let platform = &self.linux;
        #[cfg(target_os = "windows")]
        let platform = &self.windows;

        platform
    }
}

impl<T> PlatformSpecific<T>
where
    T: Copy,
{
    pub const fn copied(&self) -> T {
        *self.get()
    }
}

pub type UserAgent = PlatformSpecific<String>;
pub type FunctionON = PlatformSpecific<bool>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PakeConfig {
    pub windows: Vec<WindowConfig>,
    pub user_agent: UserAgent,
    pub system_tray: FunctionON,
    pub system_tray_path: String,
    pub proxy_url: String,
    #[serde(default)]
    pub multi_instance: bool,
    #[serde(default)]
    pub multi_window: bool,
}

impl PakeConfig {
    pub fn show_system_tray(&self) -> bool {
        self.system_tray.copied()
    }

    pub fn validate_window_labels(&self) -> Result<(), String> {
        let mut seen = std::collections::HashSet::new();
        for window in &self.windows {
            let label = window
                .label
                .as_deref()
                .filter(|value| !value.is_empty())
                .unwrap_or(MAIN_WINDOW_LABEL);

            if !seen.insert(label.to_string()) {
                return Err(format!("Duplicate window label '{label}' in pake.json"));
            }
        }
        Ok(())
    }

    pub fn window_config_by_label(&self, label: &str) -> Result<&WindowConfig, String> {
        self.windows
            .iter()
            .find(|window| {
                window
                    .label
                    .as_deref()
                    .filter(|value| !value.is_empty())
                    .unwrap_or(MAIN_WINDOW_LABEL)
                    == label
            })
            .ok_or_else(|| format!("Unknown window label '{label}'"))
    }

    pub fn main_window_label(&self) -> &str {
        self.windows
            .first()
            .and_then(|window| window.label.as_deref())
            .filter(|label| !label.is_empty())
            .unwrap_or(MAIN_WINDOW_LABEL)
    }

    pub fn route_window_templates(&self) -> Vec<&WindowConfig> {
        let main_label = self.main_window_label();
        self.windows
            .iter()
            .filter(|window| {
                window
                    .label
                    .as_deref()
                    .filter(|value| !value.is_empty())
                    .unwrap_or(MAIN_WINDOW_LABEL)
                    != main_label
            })
            .collect()
    }

    pub fn has_route_templates(&self) -> bool {
        !self.route_window_templates().is_empty()
    }

    pub fn is_route_instance_label(&self, label: &str) -> bool {
        self.route_window_templates().iter().any(|template| {
            let Some(template_label) = template.label.as_deref().filter(|value| !value.is_empty())
            else {
                return false;
            };

            label == template_label
                || label
                    .strip_prefix(template_label)
                    .map(|suffix| suffix.starts_with('-') && !suffix[1..].is_empty())
                    .unwrap_or(false)
        })
    }

    pub fn resolve_window_config(&self, label: &str) -> Result<&WindowConfig, String> {
        if let Ok(config) = self.window_config_by_label(label) {
            return Ok(config);
        }

        for template in self.route_window_templates() {
            let Some(template_label) = template.label.as_deref().filter(|value| !value.is_empty())
            else {
                continue;
            };

            if label == template_label
                || label
                    .strip_prefix(template_label)
                    .map(|suffix| suffix.starts_with('-') && !suffix[1..].is_empty())
                    .unwrap_or(false)
            {
                return Ok(template);
            }
        }

        Err(format!("Unknown window label '{label}'"))
    }
}
