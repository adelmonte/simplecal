# simplecal

A small popup calendar for Wayland bars, written in Rust with [egui](https://github.com/emilk/egui). It replaces [gsimplecal](https://github.com/dmedvinsky/gsimplecal) without GTK: one binary that only links against libc, with Wayland, xkbcommon and EGL loaded at runtime.

- Month grid with arrows to step through months and years
- Mouse wheel / touchpad scrolling through months
- Click the month name for a month picker, click the year for a year picker
- Click a greyed-out day to jump to that month; right-click anywhere to return to today
- Keyboard navigation
- Running it again closes the open instance, so a single bar button toggles it
- Closes when it loses focus (optional)
- Fully themeable from a TOML file: fonts, sizes, every color
- Optional week numbers and Monday-first weeks

## Install

### Arch Linux

A PKGBUILD is in [`packaging/`](packaging/PKGBUILD):

```sh
git clone https://github.com/adelmonte/simplecal
cd simplecal/packaging
makepkg -si
```

### From source

Needs Rust 1.85+.

```sh
cargo build --release
install -Dm755 target/release/simplecal ~/.local/bin/simplecal
```

At runtime it needs `libwayland-client`, `libxkbcommon` and an EGL driver (`libglvnd` + Mesa or similar).

## Usage

```
simplecal [-c|--config PATH] [--default-config] [--no-toggle]
```

| Option | |
|---|---|
| `-c`, `--config PATH` | Use a config file other than `~/.config/simplecal/config.toml` |
| `--default-config` | Print the default config and exit |
| `--no-toggle` | Open a new window even if one is already open |

### Waybar

```json
"clock": {
    "on-click": "simplecal"
}
```

### niri

Float the window in a corner next to the bar:

```kdl
window-rule {
    match app-id="simplecal"
    open-floating true
    default-floating-position x=0 y=0 relative-to="bottom-right"
}
```

### Keys

| Key | Action |
|---|---|
| `←` / `→`, `h` / `l`, wheel | Previous / next month (year in the month picker, page in the year picker) |
| `↑` / `↓`, `k` / `j`, `PgUp` / `PgDn` | Previous / next year |
| `Home`, `t`, `Space`, right-click | Today |
| `m` | Month picker |
| `y` | Year picker |
| `Esc`, `q` | Back out of a picker, or close |

## Configuration

Copy [`config.example.toml`](config.example.toml) to `~/.config/simplecal/config.toml` and edit it. Every key is optional. The defaults are a Nord theme with JetBrainsMono Nerd Font.

## License

GPL-3.0-or-later
