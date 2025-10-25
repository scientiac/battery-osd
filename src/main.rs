use anyhow::Result;
use gtk4::prelude::*;
use gtk4::{glib, Application};

mod types;
mod config;
mod battery;
mod ui;

use config::{load_config, load_css};
use battery::BatteryMonitor;
use ui::OSDWindow;

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
