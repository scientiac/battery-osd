use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box, Image, Label, Orientation};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

use crate::config::Config;

#[derive(Clone)]
pub struct OSDWindow {
    window: ApplicationWindow,
    icon: Image,
    label: Label,
}

impl OSDWindow {
    pub fn new(app: &Application, config: &Config) -> Self {
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

    pub fn show_message(&self, icon_name: &str, message: &str, level: &str) {
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

    pub fn hide(&self) {
        self.window.set_visible(false);
    }
}
