mod app;
mod config;

use eframe::egui;
use std::path::PathBuf;

const USAGE: &str = "usage: simplecal [-c|--config PATH] [--default-config] [--no-toggle]

Opens the calendar. If another simplecal is already running, closes it
instead (so a bar button can toggle it), unless --no-toggle is given.";

fn main() {
    let mut config_path = None;
    let mut toggle = true;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-c" | "--config" => config_path = args.next().map(PathBuf::from),
            "--default-config" => {
                print!("{}", toml::to_string_pretty(&config::Config::default()).unwrap());
                return;
            }
            "--no-toggle" => toggle = false,
            "-h" | "--help" => {
                println!("{USAGE}");
                return;
            }
            other => {
                eprintln!("simplecal: unknown argument {other:?}\n{USAGE}");
                std::process::exit(2);
            }
        }
    }

    if toggle && close_running_instances() {
        return;
    }

    let cfg = config::load(&config_path.unwrap_or_else(config::default_path));
    let size = app::window_size(&cfg);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("simplecal")
            .with_app_id(cfg.app_id.clone())
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_max_inner_size(size)
            .with_resizable(false)
            .with_decorations(false),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "simplecal",
        options,
        Box::new(move |cc| Ok(Box::new(app::Calendar::new(cc, cfg)))),
    ) {
        eprintln!("simplecal: {e}");
        std::process::exit(1);
    }
}

/// Sends SIGTERM to every other simplecal process owned by this user.
fn close_running_instances() -> bool {
    let me = std::process::id();
    let Ok(my_comm) = std::fs::read_to_string("/proc/self/comm") else {
        return false;
    };
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return false;
    };
    let mut found = false;
    for entry in entries.flatten() {
        let Some(pid) = entry.file_name().to_str().and_then(|s| s.parse::<u32>().ok()) else {
            continue;
        };
        if pid == me {
            continue;
        }
        let dir = entry.path();
        if std::fs::read_to_string(dir.join("comm")).ok().as_deref() != Some(my_comm.as_str()) {
            continue;
        }
        // Only processes we own expose a readable exe link
        if std::fs::read_link(dir.join("exe")).is_err() {
            continue;
        }
        if unsafe { libc::kill(pid as libc::pid_t, libc::SIGTERM) } == 0 {
            found = true;
        }
    }
    found
}
