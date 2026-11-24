use crate::config::{Config, Palette, WeekStart};
use chrono::{Datelike, Local, NaiveDate, Timelike};
use eframe::egui::{
    self, Align2, Color32, Event, FontFamily, FontId, Key, MouseWheelUnit, Painter, Rect, Sense,
    Stroke, StrokeKind, Ui, Vec2, pos2, vec2,
};
use std::sync::Arc;

const MONTHS: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September",
    "October", "November", "December",
];
const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MIN_YEAR: i32 = 1;
const MAX_YEAR: i32 = 9999;
const SCROLL_STEP_POINTS: f32 = 40.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    Month,
    Months,
    Years,
}

pub struct Calendar {
    cfg: Config,
    pal: Palette,
    year: i32,
    month: u32,
    view: View,
    year_page: i32,
    was_focused: bool,
    scroll_acc: f32,
}

fn week_column_width(cfg: &Config) -> f32 {
    if cfg.show_week_numbers {
        (cfg.cell_width * 0.8).round()
    } else {
        0.0
    }
}

pub fn window_size(cfg: &Config) -> Vec2 {
    vec2(
        2.0 * cfg.padding + week_column_width(cfg) + 7.0 * cfg.cell_width,
        2.0 * cfg.padding + cfg.header_height + cfg.header_gap + 7.0 * cfg.cell_height,
    )
}

fn install_fonts(ctx: &egui::Context, cfg: &Config) {
    let mut defs = egui::FontDefinitions::default();
    let fallback = defs.families[&FontFamily::Proportional].clone();
    let mut db = fontdb::Database::new();
    db.load_system_fonts();

    for (name, weight) in [("regular", cfg.font_weight), ("bold", cfg.bold_weight)] {
        let query = fontdb::Query {
            families: &[fontdb::Family::Name(&cfg.font_family)],
            weight: fontdb::Weight(weight),
            ..Default::default()
        };
        let data = db.query(&query).and_then(|id| {
            db.with_face_data(id, |bytes, index| {
                egui::FontData::from_blob(Arc::new(bytes.to_vec()), index)
            })
        });
        let mut family = Vec::new();
        match data {
            Some(data) => {
                defs.font_data.insert(name.into(), Arc::new(data));
                family.push(name.to_string());
            }
            None if name == "regular" => {
                eprintln!("simplecal: font {:?} not found, using fallback", cfg.font_family)
            }
            None => {}
        }
        family.extend(fallback.iter().cloned());
        defs.families.insert(FontFamily::Name(name.into()), family);
    }
    ctx.set_fonts(defs);
}

fn shift_month(year: i32, month: u32, by: i32) -> (i32, u32) {
    let idx = (year * 12 + month as i32 - 1 + by).clamp(MIN_YEAR * 12, MAX_YEAR * 12 + 11);
    (idx.div_euclid(12), idx.rem_euclid(12) as u32 + 1)
}

fn grid_cell(area: Rect, rows: usize, cols: usize, row: usize, col: usize) -> Rect {
    let w = area.width() / cols as f32;
    let h = area.height() / rows as f32;
    Rect::from_min_size(pos2(area.min.x + col as f32 * w, area.min.y + row as f32 * h), vec2(w, h))
}

impl Calendar {
    pub fn new(cc: &eframe::CreationContext<'_>, cfg: Config) -> Self {
        install_fonts(&cc.egui_ctx, &cfg);
        let today = Local::now().date_naive();
        Self {
            pal: cfg.colors.palette(),
            cfg,
            year: today.year(),
            month: today.month(),
            view: View::Month,
            year_page: 0,
            was_focused: false,
            scroll_acc: 0.0,
        }
    }

    fn font(&self, size: f32, bold: bool) -> FontId {
        FontId::new(size, FontFamily::Name(if bold { "bold" } else { "regular" }.into()))
    }

    fn go_today(&mut self) {
        let today = Local::now().date_naive();
        self.year = today.year();
        self.month = today.month();
        self.view = View::Month;
    }

    fn open_years(&mut self) {
        self.year_page = (self.year - 4).clamp(MIN_YEAR, MAX_YEAR - 11);
        self.view = View::Years;
    }

    /// Small step: Left/Right arrows and the scroll wheel.
    fn step(&mut self, by: i32) {
        match self.view {
            View::Month => (self.year, self.month) = shift_month(self.year, self.month, by),
            View::Months => self.year = (self.year + by).clamp(MIN_YEAR, MAX_YEAR),
            View::Years => self.year_page = (self.year_page + 12 * by).clamp(MIN_YEAR, MAX_YEAR - 11),
        }
    }

    /// Large step: Up/Down and Page Up/Down.
    fn big_step(&mut self, by: i32) {
        match self.view {
            View::Month | View::Months => self.year = (self.year + by).clamp(MIN_YEAR, MAX_YEAR),
            View::Years => self.step(by),
        }
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
        let focused = ctx.input(|i| i.viewport().focused);
        if focused == Some(true) {
            self.was_focused = true;
        } else if focused == Some(false) && self.was_focused && self.cfg.close_on_unfocus {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        let (events, right_click) = ctx.input(|i| (i.events.clone(), i.pointer.secondary_clicked()));
        if right_click {
            self.go_today();
        }
        for event in events {
            match event {
                Event::MouseWheel { unit, delta, .. } => {
                    let dy = if delta.y != 0.0 { delta.y } else { delta.x };
                    match unit {
                        MouseWheelUnit::Point => {
                            self.scroll_acc += dy;
                            while self.scroll_acc >= SCROLL_STEP_POINTS {
                                self.scroll_acc -= SCROLL_STEP_POINTS;
                                self.step(-1);
                            }
                            while self.scroll_acc <= -SCROLL_STEP_POINTS {
                                self.scroll_acc += SCROLL_STEP_POINTS;
                                self.step(1);
                            }
                        }
                        _ if dy > 0.0 => self.step(-1),
                        _ if dy < 0.0 => self.step(1),
                        _ => {}
                    }
                }
                Event::Key { key, pressed: true, .. } => match key {
                    Key::ArrowLeft | Key::H => self.step(-1),
                    Key::ArrowRight | Key::L => self.step(1),
                    Key::ArrowUp | Key::K | Key::PageUp => self.big_step(-1),
                    Key::ArrowDown | Key::J | Key::PageDown => self.big_step(1),
                    Key::Home | Key::T | Key::Space => self.go_today(),
                    Key::M => self.view = View::Months,
                    Key::Y => self.open_years(),
                    Key::Escape | Key::Q => match self.view {
                        View::Month => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                        View::Months => self.view = View::Month,
                        View::Years => self.view = View::Months,
                    },
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn hover_fill(&self, painter: &Painter, rect: Rect, hovered: bool) {
        if hovered {
            painter.rect_filled(rect, self.cfg.hover_radius, self.pal.hover);
        }
    }

    fn arrow_button(&self, ui: &Ui, rect: Rect, id: &str, left: bool) -> bool {
        let resp = ui.interact(rect, ui.scope_id().with(id), Sense::click());
        let painter = ui.painter();
        self.hover_fill(painter, rect.shrink(2.0), resp.hovered());
        let color = if resp.hovered() { self.pal.arrow_hover } else { self.pal.arrow };
        let c = rect.center();
        let s = (self.cfg.header_font_size * 0.28).max(3.0);
        let dx = if left { s * 0.5 } else { -s * 0.5 };
        painter.line(
            vec![pos2(c.x + dx, c.y - s), pos2(c.x - dx, c.y), pos2(c.x + dx, c.y + s)],
            Stroke::new(1.5, color),
        );
        resp.clicked()
    }

    fn label_button(&self, ui: &Ui, rect: Rect, id: &str, text: &str) -> bool {
        let resp = ui.interact(rect, ui.scope_id().with(id), Sense::click());
        let painter = ui.painter();
        self.hover_fill(painter, rect.shrink(2.0), resp.hovered());
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            text,
            self.font(self.cfg.header_font_size, false),
            self.pal.header_text,
        );
        resp.clicked()
    }

    fn text_width(&self, painter: &Painter, text: &str) -> f32 {
        painter
            .layout_no_wrap(text.into(), self.font(self.cfg.header_font_size, false), Color32::WHITE)
            .size()
            .x
    }

    /// Lays out `‹ label ›` with a fixed-width label slot so the arrows never move.
    fn arrow_group(&self, header: Rect, x: f32, slot: f32) -> (Rect, Rect, Rect) {
        let aw = header.height();
        let left = Rect::from_min_size(pos2(x, header.min.y), vec2(aw, header.height()));
        let label = Rect::from_min_size(pos2(left.max.x, header.min.y), vec2(slot, header.height()));
        let right = Rect::from_min_size(pos2(label.max.x, header.min.y), vec2(aw, header.height()));
        (left, label, right)
    }

    fn draw_header(&mut self, ui: &Ui, header: Rect) {
        let painter = ui.painter();
        painter.rect_filled(header, 0.0, self.pal.header_background);
        painter.rect_stroke(header, 0.0, Stroke::new(1.0, self.pal.header_border), StrokeKind::Inside);
        let aw = header.height();
        let pad = 6.0;

        match self.view {
            View::Month => {
                let month_slot = MONTHS
                    .iter()
                    .map(|m| self.text_width(painter, m))
                    .fold(0.0, f32::max)
                    + pad;
                let year_slot = self.text_width(painter, "0000") + pad;

                let (l, label, r) = self.arrow_group(header, header.min.x, month_slot);
                if self.arrow_button(ui, l, "month-prev", true) {
                    self.step(-1);
                }
                if self.label_button(ui, label, "month-label", MONTHS[self.month as usize - 1]) {
                    self.view = View::Months;
                }
                if self.arrow_button(ui, r, "month-next", false) {
                    self.step(1);
                }

                let x = header.max.x - 2.0 * aw - year_slot;
                let (l, label, r) = self.arrow_group(header, x, year_slot);
                if self.arrow_button(ui, l, "year-prev", true) {
                    self.big_step(-1);
                }
                if self.label_button(ui, label, "year-label", &self.year.to_string()) {
                    self.open_years();
                }
                if self.arrow_button(ui, r, "year-next", false) {
                    self.big_step(1);
                }
            }
            View::Months | View::Years => {
                let text = if self.view == View::Months {
                    self.year.to_string()
                } else {
                    format!("{} – {}", self.year_page, self.year_page + 11)
                };
                let slot = self.text_width(painter, "0000 – 0000") + 2.0 * pad;
                let x = header.center().x - aw - slot / 2.0;
                let (l, label, r) = self.arrow_group(header, x, slot);
                if self.arrow_button(ui, l, "nav-prev", true) {
                    self.step(-1);
                }
                if self.label_button(ui, label, "nav-label", &text) {
                    if self.view == View::Months {
                        self.open_years();
                    } else {
                        self.view = View::Months;
                    }
                }
                if self.arrow_button(ui, r, "nav-next", false) {
                    self.step(1);
                }
            }
        }
    }

    fn draw_month(&mut self, ui: &Ui, body: Rect, today: NaiveDate) {
        let cfg = &self.cfg;
        let pal = &self.pal;
        let painter = ui.painter();
        let wk = week_column_width(cfg);
        let first_day = match cfg.week_start {
            WeekStart::Sunday => 0,
            WeekStart::Monday => 1,
        };

        let band = Rect::from_min_size(body.min, vec2(body.width(), cfg.cell_height));
        painter.rect_filled(band, 0.0, pal.weekday_background);
        let cols = Rect::from_min_max(pos2(body.min.x + wk, body.min.y), body.max);
        for col in 0..7 {
            let cell = Rect::from_min_size(
                pos2(cols.min.x + col as f32 * cfg.cell_width, band.min.y),
                vec2(cfg.cell_width, cfg.cell_height),
            );
            painter.text(
                cell.center(),
                Align2::CENTER_CENTER,
                WEEKDAYS[(col + first_day) % 7],
                self.font(cfg.weekday_font_size, false),
                pal.weekday_text,
            );
        }

        let first = NaiveDate::from_ymd_opt(self.year, self.month, 1).unwrap();
        let offset = (first.weekday().num_days_from_sunday() as i64 - first_day as i64).rem_euclid(7);
        let start = first - chrono::Duration::days(offset);
        let mut target = None;

        for row in 0..6 {
            let y = band.max.y + row as f32 * cfg.cell_height;
            let row_start = start + chrono::Duration::days(row * 7);

            if cfg.show_week_numbers {
                let iso = (row_start + chrono::Duration::days(if first_day == 0 { 1 } else { 0 })).iso_week();
                let cell = Rect::from_min_size(pos2(body.min.x, y), vec2(wk, cfg.cell_height));
                painter.text(
                    cell.center(),
                    Align2::CENTER_CENTER,
                    iso.week().to_string(),
                    self.font(cfg.font_size * 0.8, false),
                    pal.week_number_text,
                );
            }

            for col in 0..7 {
                let date = row_start + chrono::Duration::days(col);
                let cell = Rect::from_min_size(
                    pos2(cols.min.x + col as f32 * cfg.cell_width, y),
                    vec2(cfg.cell_width, cfg.cell_height),
                );
                let resp = ui.interact(cell, ui.scope_id().with(("day", row, col)), Sense::click());
                let in_month = date.month() == self.month;
                let is_today = in_month && date == today;
                let weekend = date.weekday().num_days_from_monday() >= 5;

                if is_today {
                    painter.rect_filled(cell, cfg.today_radius, pal.today_background);
                } else {
                    self.hover_fill(painter, cell.shrink(1.0), resp.hovered());
                }
                let color = match () {
                    _ if is_today => pal.today_text,
                    _ if !in_month => pal.other_month_text,
                    _ if weekend => pal.weekend_text,
                    _ => pal.text,
                };
                painter.text(
                    cell.center(),
                    Align2::CENTER_CENTER,
                    date.day().to_string(),
                    self.font(cfg.font_size, is_today),
                    color,
                );
                if resp.clicked() && !in_month {
                    target = Some((date.year(), date.month()));
                }
            }
        }

        if let Some((y, m)) = target {
            self.year = y;
            self.month = m;
        }
    }

    /// Shared 4x3 grid used by the month and year pickers.
    fn draw_picker(&self, ui: &Ui, body: Rect, labels: &[String], current: usize, today: Option<usize>) -> Option<usize> {
        let painter = ui.painter();
        let mut picked = None;
        for (i, label) in labels.iter().enumerate() {
            let cell = grid_cell(body, 4, 3, i / 3, i % 3).shrink(3.0);
            let resp = ui.interact(cell, ui.scope_id().with(("pick", i)), Sense::click());
            let is_today = today == Some(i);
            if is_today {
                painter.rect_filled(cell, self.cfg.today_radius, self.pal.today_background);
            } else {
                self.hover_fill(painter, cell, resp.hovered());
            }
            if i == current {
                painter.rect_stroke(
                    cell,
                    self.cfg.hover_radius,
                    Stroke::new(1.0, self.pal.selected_border),
                    StrokeKind::Inside,
                );
            }
            painter.text(
                cell.center(),
                Align2::CENTER_CENTER,
                label,
                self.font(self.cfg.font_size, is_today),
                if is_today { self.pal.today_text } else { self.pal.text },
            );
            if resp.clicked() {
                picked = Some(i);
            }
        }
        picked
    }

    fn draw_months(&mut self, ui: &Ui, body: Rect, today: NaiveDate) {
        let labels: Vec<String> = MONTHS.iter().map(|m| m[..3].to_string()).collect();
        let today_idx = (today.year() == self.year).then(|| today.month0() as usize);
        if let Some(i) = self.draw_picker(ui, body, &labels, self.month as usize - 1, today_idx) {
            self.month = i as u32 + 1;
            self.view = View::Month;
        }
    }

    fn draw_years(&mut self, ui: &Ui, body: Rect, today: NaiveDate) {
        let labels: Vec<String> = (0..12).map(|i| (self.year_page + i).to_string()).collect();
        let idx = |y: i32| (y >= self.year_page && y < self.year_page + 12).then(|| (y - self.year_page) as usize);
        let current = idx(self.year).unwrap_or(usize::MAX);
        if let Some(i) = self.draw_picker(ui, body, &labels, current, idx(today.year())) {
            self.year = self.year_page + i as i32;
            self.view = View::Month;
        }
    }
}

impl eframe::App for Calendar {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.pal.background.to_normalized_gamma_f32()
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_input(&ctx);

        let now = Local::now();
        let today = now.date_naive();
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 0.0, self.pal.background);

        let inner = rect.shrink(self.cfg.padding);
        let header = Rect::from_min_size(inner.min, vec2(inner.width(), self.cfg.header_height));
        let body = Rect::from_min_max(pos2(inner.min.x, header.max.y + self.cfg.header_gap), inner.max);

        self.draw_header(ui, header);
        match self.view {
            View::Month => self.draw_month(ui, body, today),
            View::Months => self.draw_months(ui, body, today),
            View::Years => self.draw_years(ui, body, today),
        }

        let until_midnight = 86_400 - now.num_seconds_from_midnight() as u64;
        ctx.request_repaint_after(std::time::Duration::from_secs(until_midnight + 1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shifts_months_across_years() {
        assert_eq!(shift_month(2026, 11, 1), (2026, 12));
        assert_eq!(shift_month(2026, 12, 1), (2027, 1));
        assert_eq!(shift_month(2026, 1, -1), (2025, 12));
        assert_eq!(shift_month(2026, 3, -27), (2023, 12));
        assert_eq!(shift_month(MIN_YEAR, 1, -1), (MIN_YEAR, 1));
        assert_eq!(shift_month(MAX_YEAR, 12, 1), (MAX_YEAR, 12));
    }
}
