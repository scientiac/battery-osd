# battery-osd
An OSD for battery level alerts for your wayland compositor.

> I got tired of my laptop shutting off because I didn't know it had low battery so (A)I made this tiny utility.

## Images

<div style="display: flex; flex-wrap: wrap; justify-content: center; gap: 1rem;">
  <img src="./resources/charging.png" alt="Charging" width="180">
  <img src="./resources/healthy.png" alt="Healthy" width="180">
  <img src="./resources/discharging.png" alt="Discharging" width="180">
  <img src="./resources/low.png" alt="Low" width="180">
  <img src="./resources/critical.png" alt="Critical" width="180">
</div>

## Installation
1. Clone this repository.
2. Run `cargo build --release`
3. Copy/Move `./target/release/battery-osd` to your $PATH. 
4. Run it during startup.

## Configuration

Put your configuration file at `$HOME/.config/battery-osd/config.toml`.
And your styles file at `$HOME/.config/battery-osd/style.css`.

You can find the `config.toml` and `style.css` example at [./config](./config/)
