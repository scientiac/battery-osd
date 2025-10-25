use anyhow::Result;
use std::fs;
use std::process::Command;
use std::sync::{Arc, Mutex};

use crate::config::Config;
use crate::types::{BatteryInfo, BatteryStatus};

impl BatteryInfo {
    pub fn read_from_sysfs(battery_path: &str) -> Result<Self> {
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

pub struct BatteryMonitor {
    config: Config,
    last_state: Arc<Mutex<Option<BatteryInfo>>>,
    last_healthy_notified: Arc<Mutex<bool>>,
}

impl BatteryMonitor {
    pub fn new(config: Config) -> Self {
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

    pub fn check_battery(&self) -> Result<Option<(String, String, String, u64)>> {
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
