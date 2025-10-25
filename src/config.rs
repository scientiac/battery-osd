use gtk4::CssProvider;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub position: PositionConfig,
    #[serde(default = "default_critical")]
    pub critical_threshold: f64,
    #[serde(default = "default_low")]
    pub low_threshold: f64,
    #[serde(default = "default_healthy")]
    pub healthy_threshold: f64,
    #[serde(default = "default_battery_path")]
    pub battery_path: String,
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,
    #[serde(default)]
    pub commands: CommandConfig,
    #[serde(default)]
    pub timeouts: TimeoutConfig,
    #[serde(default)]
    pub disable: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TimeoutConfig {
    #[serde(default = "default_timeout")]
    pub charging: u64,
    #[serde(default = "default_timeout")]
    pub discharging: u64,
    #[serde(default = "default_timeout_critical")]
    pub critical: u64,
    #[serde(default = "default_timeout_critical")]
    pub low: u64,
    #[serde(default = "default_timeout")]
    pub full: u64,
    #[serde(default = "default_timeout")]
    pub healthy: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            charging: default_timeout(),
            discharging: default_timeout(),
            critical: default_timeout_critical(),
            low: default_timeout_critical(),
            full: default_timeout(),
            healthy: default_timeout(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct PositionConfig {
    #[serde(default = "default_horizontal")]
    pub horizontal: String,
    #[serde(default = "default_vertical")]
    pub vertical: String,
    #[serde(default)]
    pub padding_top: i32,
    #[serde(default)]
    pub padding_bottom: i32,
    #[serde(default)]
    pub padding_left: i32,
    #[serde(default)]
    pub padding_right: i32,
}

impl Default for PositionConfig {
    fn default() -> Self {
        Self {
            horizontal: default_horizontal(),
            vertical: default_vertical(),
            padding_top: 20,
            padding_bottom: 0,
            padding_left: 0,
            padding_right: 0,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct CommandConfig {
    #[serde(default)]
    pub on_charging: Option<String>,
    #[serde(default)]
    pub on_discharging: Option<String>,
    #[serde(default)]
    pub on_critical: Option<String>,
    #[serde(default)]
    pub on_low: Option<String>,
    #[serde(default)]
    pub on_full: Option<String>,
    #[serde(default)]
    pub on_healthy: Option<String>,
}

impl Default for CommandConfig {
    fn default() -> Self {
        Self {
            on_charging: None,
            on_discharging: None,
            on_critical: None,
            on_low: None,
            on_full: None,
            on_healthy: None,
        }
    }
}

fn default_timeout() -> u64 { 3000 }
fn default_timeout_critical() -> u64 { 12000 }
fn default_horizontal() -> String { "center".to_string() }
fn default_vertical() -> String { "top".to_string() }
fn default_critical() -> f64 { 10.0 }
fn default_low() -> f64 { 20.0 }
fn default_healthy() -> f64 { 80.0 }
fn default_battery_path() -> String { "/sys/class/power_supply/BAT0".to_string() }
fn default_poll_interval() -> u64 { 5 }

impl Default for Config {
    fn default() -> Self {
        Self {
            position: PositionConfig::default(),
            critical_threshold: default_critical(),
            low_threshold: default_low(),
            healthy_threshold: default_healthy(),
            battery_path: default_battery_path(),
            poll_interval_secs: default_poll_interval(),
            commands: CommandConfig::default(),
            timeouts: TimeoutConfig::default(),
            disable: Vec::new(),
        }
    }
}

pub fn load_config() -> Config {
    let config_path = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
        .join(".config/battery-osd/config.toml");
    
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        match toml::from_str(&content) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Failed to parse config: {}. Using defaults.", e);
                Config::default()
            }
        }
    } else {
        Config::default()
    }
}

pub fn load_css() {
    let css_path = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".to_string()))
        .join(".config/battery-osd/style.css");
    
    let css = if let Ok(content) = std::fs::read_to_string(&css_path) {
        content
    } else {
        include_str!("../style/style.css").to_string()
    };

    let provider = CssProvider::new();
    provider.load_from_data(&css);
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().unwrap(),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
