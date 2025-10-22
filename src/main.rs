use anyhow::Result;
use gtk4::prelude::*;
use gtk4::{glib, Application, ApplicationWindow, Box, CssProvider, Image, Label, Orientation};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::fs;
use std::process::Command;

#[derive(Debug, Deserialize, Clone)]
struct Config {
    #[serde(default)]
    position: PositionConfig,
    #[serde(default = "default_critical")]
    critical_threshold: f64,
    #[serde(default = "default_low")]
    low_threshold: f64,
    #[serde(default = "default_healthy")]
    healthy_threshold: f64,
    #[serde(default = "default_battery_path")]
    battery_path: String,
    #[serde(default = "default_poll_interval")]
    poll_interval_secs: u64,
    #[serde(default)]
    commands: CommandConfig,
    #[serde(default)]
    timeouts: TimeoutConfig,
    #[serde(default)]
    disable: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct TimeoutConfig {
    #[serde(default = "default_timeout")]
    charging: u64,
    #[serde(default = "default_timeout")]
    discharging: u64,
    #[serde(default = "default_timeout_critical")]
    critical: u64,
    #[serde(default = "default_timeout_critical")]
    low: u64,
    #[serde(default = "default_timeout")]
    full: u64,
    #[serde(default = "default_timeout")]
    healthy: u64,
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
struct PositionConfig {
    #[serde(default = "default_horizontal")]
    horizontal: String,
    #[serde(default = "default_vertical")]
    vertical: String,
    #[serde(default)]
    padding_top: i32,
    #[serde(default)]
    padding_bottom: i32,
    #[serde(default)]
    padding_left: i32,
    #[serde(default)]
    padding_right: i32,
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
struct CommandConfig {
    #[serde(default)]
    on_charging: Option<String>,
    #[serde(default)]
    on_discharging: Option<String>,
    #[serde(default)]
    on_critical: Option<String>,
    #[serde(default)]
    on_low: Option<String>,
    #[serde(default)]
    on_full: Option<String>,
    #[serde(default)]
    on_healthy: Option<String>,
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

#[derive(Debug, Clone, PartialEq)]
enum BatteryStatus {
    Charging,
    Discharging,
    Full,
    Unknown,
}

#[derive(Debug, Clone)]
struct BatteryInfo {
    capacity: f64,
    status: BatteryStatus,
}

impl BatteryInfo {
    fn read_from_sysfs(battery_path: &str) -> Result<Self> {
        let capacity_path = format!("{}/capacity", battery_path);
        let status_path = format!("{}/status", battery_path);

        let capacity_str = fs::read_to_string(&capacity_path)
            .map_err(|e| anyhow::anyhow!("Failed to read capacity from {}: {}", capacity_path, e))?;
        let capacity = capacity_str.trim().parse::<f64>()
            .map_err(|e| anyhow::anyhow!("Failed to parse capacity: {}", e))?;

        let status_str = fs::read_to_string(&status_path)
            .map_err(|e| anyhow::anyhow!("Failed to read status from {}: {}", status_path, e))?;
        let status = match status_str.trim() {
            "Charging" => BatteryStatus::Charging,
            "Discharging" => BatteryStatus::Discharging,
            "Full" => BatteryStatus::Full,
            _ => BatteryStatus::Unknown,
        };

        Ok(Self { capacity, status })
    }
}

#[derive(Clone)]
struct OSDWindow {
    window: ApplicationWindow,
    icon: Image,
    label: Label,
}

impl OSDWindow {
    fn new(app: &Application, config: &Config) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .build();

        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);

        // Clear all anchors first
        window.set_anchor(Edge::Left, false);
        window.set_anchor(Edge::Right, false);
        window.set_anchor(Edge::Top, false);
        window.set_anchor(Edge::Bottom, false);

        // Set all margins to 0 first
        window.set_margin(Edge::Left, 0);
        window.set_margin(Edge::Right, 0);
        window.set_margin(Edge::Top, 0);
        window.set_margin(Edge::Bottom, 0);

        // Set horizontal positioning
        match config.position.horizontal.as_str() {
            "left" => {
                window.set_anchor(Edge::Left, true);
                window.set_margin(Edge::Left, config.position.padding_left);
            }
            "right" => {
                window.set_anchor(Edge::Right, true);
                window.set_margin(Edge::Right, config.position.padding_right);
            }
            "center" | _ => {
                window.set_anchor(Edge::Left, true);
                window.set_anchor(Edge::Right, true);
            }
        }

        // Set vertical positioning
        match config.position.vertical.as_str() {
            "top" => {
                window.set_anchor(Edge::Top, true);
                window.set_margin(Edge::Top, config.position.padding_top);
            }
            "bottom" => {
                window.set_anchor(Edge::Bottom, true);
                window.set_margin(Edge::Bottom, config.position.padding_bottom);
            }
            _ => {
                window.set_anchor(Edge::Top, true);
                window.set_margin(Edge::Top, config.position.padding_top);
            }
        }

        let container = Box::new(Orientation::Horizontal, 10);
        container.set_halign(gtk4::Align::Center);
        container.set_valign(gtk4::Align::Center);
        container.add_css_class("osd-container");

        let icon = Image::from_icon_name("battery-symbolic");
        icon.set_pixel_size(24);
        icon.add_css_class("osd-icon");
        container.append(&icon);

        let label = Label::new(None);
        label.add_css_class("osd-label");
        container.append(&label);

        window.set_child(Some(&container));
        window.set_visible(false);

        Self { window, icon, label }
    }

    fn show_message(&self, icon_name: &str, message: &str, level: &str) {
        self.icon.set_icon_name(Some(icon_name));
        self.label.set_text(message);
        
        self.window.remove_css_class("critical");
        self.window.remove_css_class("low");
        self.window.remove_css_class("charging");
        self.window.remove_css_class("full");
        self.window.remove_css_class("healthy");
        self.window.remove_css_class("normal");
        self.window.add_css_class(level);
        self.window.set_visible(true);
    }

    fn hide(&self) {
        self.window.set_visible(false);
    }
}

struct BatteryMonitor {
    config: Config,
    last_state: Arc<Mutex<Option<BatteryInfo>>>,
    last_healthy_notified: Arc<Mutex<bool>>,
}

impl BatteryMonitor {
    fn new(config: Config) -> Self {
        Self {
            config,
            last_state: Arc::new(Mutex::new(None)),
            last_healthy_notified: Arc::new(Mutex::new(false)),
        }
    }

    fn execute_command(&self, command: &Option<String>) {
        if let Some(cmd) = command {
            std::thread::spawn({
                let cmd = cmd.clone();
                move || {
                    if let Err(e) = Command::new("sh")
                        .arg("-c")
                        .arg(&cmd)
                        .spawn()
                        .and_then(|mut child| child.wait())
                    {
                        eprintln!("Failed to execute command '{}': {}", cmd, e);
                    }
                }
            });
        }
    }

    fn is_disabled(&self, level: &str) -> bool {
        self.config.disable.iter().any(|disabled_level| {
            disabled_level.to_lowercase() == level.to_lowercase()
        })
    }

    fn check_battery(&self) -> Result<Option<(String, String, String, u64)>> {
        let battery_info = BatteryInfo::read_from_sysfs(&self.config.battery_path)?;
        
        let mut last = self.last_state.lock().unwrap();
        let mut last_healthy = self.last_healthy_notified.lock().unwrap();
        
        let should_show = if let Some(ref last_info) = *last {
            let state_changed = last_info.status != battery_info.status;
            
            let crossing_threshold = battery_info.status == BatteryStatus::Discharging && 
                ((battery_info.capacity <= self.config.critical_threshold && last_info.capacity > self.config.critical_threshold) ||
                 (battery_info.capacity <= self.config.low_threshold && last_info.capacity > self.config.low_threshold));
            
            let crossing_healthy = battery_info.status == BatteryStatus::Charging &&
                battery_info.capacity >= self.config.healthy_threshold &&
                last_info.capacity < self.config.healthy_threshold &&
                !*last_healthy;
            
            if battery_info.status == BatteryStatus::Discharging {
                *last_healthy = false;
            }
            
            if crossing_healthy {
                *last_healthy = true;
            }
            
            state_changed || crossing_threshold || crossing_healthy
        } else {
            true
        };

        *last = Some(battery_info.clone());

        if should_show {
            let capacity = battery_info.capacity as i32;
            let (icon, message, level, command, timeout) = match battery_info.status {
                BatteryStatus::Charging => {
                    if battery_info.capacity >= self.config.healthy_threshold {
                        ("battery-good-charging-symbolic", 
                         format!("Healthy {}%", capacity), 
                         "healthy", 
                         &self.config.commands.on_healthy,
                         self.config.timeouts.healthy)
                    } else {
                        ("battery-level-50-charging-symbolic", 
                         format!("Charging {}%", capacity), 
                         "charging", 
                         &self.config.commands.on_charging,
                         self.config.timeouts.charging)
                    }
                }
                BatteryStatus::Discharging => {
                    if battery_info.capacity <= self.config.critical_threshold {
                        ("battery-level-10-symbolic",
                         format!("Critical {}%", capacity),
                         "critical", 
                         &self.config.commands.on_critical,
                         self.config.timeouts.critical)
                    } else if battery_info.capacity <= self.config.low_threshold {
                        ("battery-level-20-symbolic",
                         format!("Low {}%", capacity),
                         "low", 
                         &self.config.commands.on_low,
                         self.config.timeouts.low)
                    } else {
                        ("battery-good-symbolic",
                         format!("Discharging {}%", capacity),
                         "normal", 
                         &self.config.commands.on_discharging,
                         self.config.timeouts.discharging)
                    }
                }
                BatteryStatus::Full => {
                    ("battery-full-symbolic",
                     format!("Full {}%", capacity),
                     "full", 
                     &self.config.commands.on_full,
                     self.config.timeouts.full)
                }
                BatteryStatus::Unknown => {
                    ("battery-missing-symbolic",
                     format!("Battery {}%", capacity),
                     "normal", 
                     &None,
                     self.config.timeouts.discharging)
                }
            };

            // Check if this notification is disabled
            if self.is_disabled(level) {
                return Ok(None);
            }

            self.execute_command(command);

            return Ok(Some((icon.to_string(), message, level.to_string(), timeout)));
        }

        Ok(None)
    }
}

fn load_config() -> Config {
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

fn load_css() {
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

fn main() -> Result<()> {
    let config = load_config();
    
    let app = Application::builder()
        .application_id("com.github.battery-osd")
        .build();

    app.connect_startup(|_| {
        load_css();
    });

    app.connect_activate(move |app| {
        let config = config.clone();
        let osd = OSDWindow::new(app, &config);
        let monitor = BatteryMonitor::new(config.clone());

        let poll_interval = config.poll_interval_secs;

        glib::timeout_add_seconds_local(poll_interval as u32, move || {
            match monitor.check_battery() {
                Ok(Some((icon, message, level, timeout))) => {
                    osd.show_message(&icon, &message, &level);
                    glib::timeout_add_local_once(
                        std::time::Duration::from_millis(timeout),
                        {
                            let osd = osd.clone();
                            move || osd.hide()
                        }
                    );
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!("Error checking battery: {}", e);
                }
            }
            glib::ControlFlow::Continue
        });
    });

    app.run_with_args(&Vec::<String>::new());
    Ok(())
}
