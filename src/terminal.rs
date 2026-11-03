use crate::persistence;
use crossterm::{
    cursor::Show,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    style::{Attribute, ResetColor, SetAttribute},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, Widget},
    Terminal,
};
use std::{
    io::{self, stdout},
    path::PathBuf,
    time::{Duration, Instant},
};
use termatrix::{
    model::Scene,
    settings::{Settings, MAX_TRACE_SPEED, MIN_TRACE_SPEED},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
fn restore_output(writer: &mut impl io::Write) -> io::Result<()> {
    execute!(
        writer,
        ResetColor,
        SetAttribute(Attribute::Reset),
        Show,
        LeaveAlternateScreen
    )
}
struct Restore;
impl Drop for Restore {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = restore_output(&mut stdout());
    }
}
const FIELDS: [&str; 15] = [
    "Min speed",
    "Max speed",
    "Speed bias",
    "Density",
    "Clumping",
    "Min length",
    "Max length",
    "Mutation rate",
    "Refresh cap",
    "Canvas-relative",
    "Palette",
    "Alphabet",
    "Individual glyph colors",
    "Highlight probability",
    "Chart compression",
];
const ALPHABETS: [&str; 14] = [
    "all",
    "alpha",
    "alpha_lower",
    "alpha_upper",
    "alphanum",
    "binary",
    "decimal",
    "hex",
    "hex_lower",
    "hex_upper",
    "math",
    "set",
    "special",
    "symbol",
];
const PALETTES: [(&str, &str); 11] = crate::palettes::BUILTINS;
fn value(s: &Settings, i: usize) -> String {
    match i {
        0 => format!("{:.2}", s.min_speed),
        1 => format!("{:.2}", s.max_speed),
        2 => format!("{:.1}", s.speed_bias),
        3 => format!("{:.1}", s.density),
        4 => format!("{:.1}", s.clumping),
        5 => s.min_length.to_string(),
        6 => s.max_length.to_string(),
        7 => format!("{:.2}", s.mutation_rate),
        8 => format!("{} Hz", s.max_refresh_rate),
        9 => s.canvas_relative.to_string(),
        10 => PALETTES
            .iter()
            .find(|(_, colors)| *colors == s.colors)
            .map(|(name, _)| name.to_string())
            .unwrap_or_else(|| format!("Custom ({})", s.colors)),
        12 => s.per_glyph_colors.to_string(),
        13 => format!("{:.0}%", s.highlight_probability * 100.),
        14 => format!(
            "{:.1}{}",
            s.chart_compression,
            if s.chart_compression == 0. {
                " (linear)"
            } else {
                ""
            }
        ),
        _ => s.glyph_set.clone(),
    }
}
fn edit(s: &mut Settings, i: usize, d: f64) {
    let mut n = s.clone();
    match i {
        0 => n.min_speed = (n.min_speed + d * 0.25).clamp(MIN_TRACE_SPEED, n.max_speed),
        1 => n.max_speed = (n.max_speed + d * 0.25).clamp(n.min_speed, MAX_TRACE_SPEED),
        2 => n.speed_bias = (n.speed_bias + d * 0.1).clamp(-1., 1.),
        3 => n.density = (n.density + d * 0.1).clamp(0., 10.),
        4 => n.clumping = (n.clumping + d * 0.1).clamp(0., 1.),
        5 => n.min_length = (n.min_length as i64 + d as i64).clamp(1, n.max_length as i64) as usize,
        6 => {
            n.max_length = (n.max_length as i64 + d as i64).clamp(n.min_length as i64, 512) as usize
        }
        7 => n.mutation_rate = (n.mutation_rate + d * 0.05).clamp(0., 10.),
        8 => n.max_refresh_rate = (n.max_refresh_rate as i64 + d as i64 * 5).clamp(1, 240) as u16,
        9 => n.canvas_relative = !n.canvas_relative,
        10 => {
            let p = PALETTES
                .iter()
                .position(|(_, colors)| *colors == n.colors)
                .unwrap_or(0);
            n.colors = PALETTES[(p as i64 + d as i64).rem_euclid(PALETTES.len() as i64) as usize]
                .1
                .into();
        }
        12 => n.per_glyph_colors = !n.per_glyph_colors,
        13 => n.highlight_probability = (n.highlight_probability + d * 0.05).clamp(0., 1.),
        14 => n.chart_compression = (n.chart_compression + d * 0.5).clamp(0., 10.),
        _ => {
            let p = ALPHABETS
                .iter()
                .position(|p| *p == n.glyph_set)
                .unwrap_or(0);
            n.glyph_set = ALPHABETS[(p as i64 + d as i64).rem_euclid(14) as usize].into();
        }
    }
    if n.validate().is_ok() {
        *s = n;
    }
}
// Dark-theme artistic palette. All foregrounds are explicit RGB, never ANSI names.
pub(crate) fn color(c: char) -> Color {
    use term_colors::{Color as Hue, Depth, Palette, Theme};
    static PALETTE: std::sync::OnceLock<Palette> = std::sync::OnceLock::new();
    let hue = match c {
        'r' => Hue::Red,
        'o' => Hue::Orange,
        'y' => Hue::Yellow,
        'g' => Hue::Green,
        'b' => Hue::Blue,
        'm' => Hue::Purple,
        's' => Hue::Grey,
        // Local additions outside the shared seven-hue palette.
        'G' | 'R' | 'S' | 'M' | 'Y' | 'O' | 'B' => {
            let [r, g, b] = Palette::theme(Theme::Light)
                .color(
                    match c {
                        'G' => Hue::Green,
                        'R' => Hue::Red,
                        'B' => Hue::Blue,
                        'M' => Hue::Purple,
                        'Y' => Hue::Yellow,
                        'O' => Hue::Orange,
                        _ => Hue::Grey,
                    },
                    Depth::TrueColor,
                )
                .swatch()
                .rgb;
            return Color::Rgb(r, g, b);
        }
        'c' => return Color::Rgb(51, 187, 204),
        'w' => return Color::Rgb(238, 238, 238),
        'k' => return Color::Rgb(34, 34, 34),
        _ => Hue::Green,
    };
    let [r, g, b] = PALETTE
        .get_or_init(|| Palette::theme(Theme::Dark))
        .color(hue, Depth::TrueColor)
        .swatch()
        .rgb;
    Color::Rgb(r, g, b)
}

#[cfg(test)]
fn brighten(c: Color) -> Color {
    match c {
        Color::Rgb(r, g, b) => {
            let lift = |v: u8| v + ((255 - v) as f64 * 0.35).round() as u8;
            Color::Rgb(lift(r), lift(g), lift(b))
        }
        _ => c,
    }
}
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum Theme {
    Auto,
    Dark,
    Light,
}
#[derive(Clone, Copy, Debug)]
struct Appearance {
    light: bool,
    depth: u16,
}
impl Appearance {
    fn detect(theme: Theme, term: &str, colorterm: &str, background: &str) -> Self {
        let light = match theme {
            Theme::Light => true,
            Theme::Dark => false,
            Theme::Auto => background
                .rsplit(';')
                .next()
                .and_then(|v| v.parse::<u8>().ok())
                .is_some_and(|v| matches!(v, 7 | 9..=15)),
        };
        let depth = if matches!(
            colorterm.to_ascii_lowercase().as_str(),
            "truecolor" | "24bit"
        ) || term.contains("direct")
        {
            24
        } else if term.contains("256color") {
            256
        } else {
            16
        };
        Self { light, depth }
    }
    #[cfg(test)]
    fn foreground(self, base: Color, progress: f64, accent: bool) -> Color {
        self.foreground_with(base, progress, accent, |rgb, trailing| {
            self.quantize(rgb, trailing)
        })
    }
    fn foreground_with(
        self,
        base: Color,
        progress: f64,
        accent: bool,
        mut reduced: impl FnMut([u8; 3], bool) -> Color,
    ) -> Color {
        let Color::Rgb(r, g, b) = base else {
            return base;
        };
        let head = if self.light { 0. } else { 255. };
        let p = progress.clamp(0., 1.);
        let channel = |v: u8| {
            let value = if accent {
                head * 0.75 + v as f64 * 0.25
            } else if p <= 0.3 {
                head + (v as f64 - head) * p / 0.3
            } else {
                // Retain 15% of the primary color, with a small contrast floor
                // even for a primary color matching the fade endpoint.
                let endpoint = if self.light { 243. } else { 12. };
                v as f64 + (endpoint - v as f64) * 0.85 * (p - 0.3) / 0.7
            };
            value.round() as u8
        };
        let rgb = [channel(r), channel(g), channel(b)];
        if self.depth == 24 {
            return Color::Rgb(rgb[0], rgb[1], rgb[2]);
        }
        reduced(rgb, p > 0.3)
    }
    fn quantize(self, rgb: [u8; 3], trailing: bool) -> Color {
        let count = if self.depth == 256 { 256 } else { 16 };
        let index = (0..count)
            // Quantization must not erase the faint trailing glyphs.
            .filter(|&i| !trailing || indexed_rgb(i as u8) != [if self.light { 255 } else { 0 }; 3])
            .min_by_key(|&i| {
                let c = indexed_rgb(i as u8);
                rgb.iter()
                    .zip(c)
                    .map(|(&a, b)| (a as i32 - b as i32).pow(2))
                    .sum::<i32>()
            })
            .unwrap();
        Color::Indexed(index as u8)
    }
}
/// Cache exact rounded fade colors, rather than sampling progress into buckets.
/// Owned by the renderer so entries survive frames and live palette changes.
struct RainColors {
    appearance: Appearance,
    reduced: std::cell::RefCell<std::collections::HashMap<([u8; 3], bool), Color>>,
}
impl RainColors {
    fn new(appearance: Appearance) -> Self {
        Self {
            appearance,
            reduced: Default::default(),
        }
    }
    fn foreground(&self, base: Color, progress: f64, accent: bool) -> Color {
        self.appearance
            .foreground_with(base, progress, accent, |rgb, trailing| {
                *self
                    .reduced
                    .borrow_mut()
                    .entry((rgb, trailing))
                    .or_insert_with(|| self.appearance.quantize(rgb, trailing))
            })
    }
}
fn indexed_rgb(i: u8) -> [u8; 3] {
    const BASIC: [[u8; 3]; 16] = [
        [0, 0, 0],
        [128, 0, 0],
        [0, 128, 0],
        [128, 128, 0],
        [0, 0, 128],
        [128, 0, 128],
        [0, 128, 128],
        [192, 192, 192],
        [128, 128, 128],
        [255, 0, 0],
        [0, 255, 0],
        [255, 255, 0],
        [0, 0, 255],
        [255, 0, 255],
        [0, 255, 255],
        [255, 255, 255],
    ];
    match i {
        0..=15 => BASIC[i as usize],
        16..=231 => {
            let n = i - 16;
            let level = |v: u8| if v == 0 { 0 } else { 55 + 40 * v };
            [level(n / 36), level(n / 6 % 6), level(n % 6)]
        }
        _ => [8 + 10 * (i - 232); 3],
    }
}
#[cfg(test)]
struct Rain<'a>(&'a Scene);
struct ThemedRain<'a>(&'a Scene, &'a RainColors);
#[cfg(test)]
impl Widget for Rain<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        ThemedRain(
            self.0,
            &RainColors::new(Appearance {
                light: false,
                depth: 24,
            }),
        )
        .render(area, buf);
    }
}
impl Widget for ThemedRain<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for sample in termatrix::projection::orthographic(self.0) {
            let x = area.x as usize + sample.column * 2;
            if x + 1 >= area.right() as usize || sample.row >= area.height as usize {
                continue;
            }
            let y = area.y + sample.row as u16;
            use termatrix::projection::Highlight;
            let emphasized = matches!(sample.highlight, Highlight::Head | Highlight::Accent);
            let mut style = Style::default().fg(self.1.foreground(
                color(sample.color),
                sample.progress,
                sample.highlight == Highlight::Accent,
            ));
            if emphasized {
                style = style.add_modifier(Modifier::BOLD);
            }
            buf.set_string(x as u16, y, "  ", Style::default());
            if sample.symbol.width().is_some_and(|w| w > 0 && w <= 2) {
                buf.set_string(x as u16, y, sample.symbol.to_string(), style);
            }
        }
    }
}
fn regions(a: Rect, p: bool) -> (Rect, Rect) {
    let h = if p { a.height.min(12) } else { 0 };
    (
        Rect::new(a.x, a.y, a.width, h),
        Rect::new(a.x, a.y + h, a.width, a.height - h),
    )
}
// Borders, keyboard help and save status reserve four rows.
fn settings_window(panel_height: u16, selected: usize) -> std::ops::Range<usize> {
    let count = (panel_height.saturating_sub(4) as usize).min(FIELDS.len());
    let first = selected.saturating_sub(count / 2).min(FIELDS.len() - count);
    first..first + count
}
#[derive(Default)]
enum SaveStatus {
    #[default]
    Config,
    Saved,
    Failed(String),
}
#[derive(Default)]
struct SaveState {
    unsaved: bool,
    changed: Option<Instant>,
    status: SaveStatus,
}
impl SaveState {
    fn mark_changed(&mut self) {
        self.unsaved = true;
        self.changed = Some(Instant::now());
    }
    fn attempt(&mut self, path: &std::path::Path, settings: &Settings) {
        self.status = match persistence::save(path, settings) {
            Ok(()) => {
                self.unsaved = false;
                SaveStatus::Saved
            }
            Err(error) => SaveStatus::Failed(error.to_string()),
        };
        self.changed = None;
    }
    fn finish(&self, path: &std::path::Path, settings: &Settings) -> io::Result<()> {
        if self.unsaved {
            persistence::save(path, settings).map_err(|error| {
                io::Error::other(format!(
                    "cannot save preferences to {}: {error}",
                    path.display()
                ))
            })?;
        }
        Ok(())
    }
}
/// Wrap help independently from setting values, preserving styled keys.
fn control_help(width: usize, paused: bool) -> Vec<Line<'static>> {
    let width = width.max(1);
    let mut lines = vec![Line::default()];
    let mut used: usize = 0;
    let mut commands = vec![
        ("?", " controls"),
        ("p", " profiles"),
        ("c", " palettes"),
        ("↑↓", " select"),
        ("←→", " adjust"),
        ("Space", " pause"),
        ("q", " quit"),
    ];
    if paused {
        commands.push(("Paused", ""));
    }
    for (key, description) in commands {
        let length = key.width() + description.width();
        if used > 0 {
            if used.saturating_add(3).saturating_add(length) <= width {
                lines
                    .last_mut()
                    .unwrap()
                    .spans
                    .push(Span::styled(" | ", Style::default().fg(color('s'))));
                used += 3;
            } else {
                lines.push(Line::default());
                used = 0;
            }
        }
        for (text, hue) in [(key, 'y'), (description, 's')] {
            for ch in text.chars() {
                let cells = ch.width().unwrap_or(0);
                if used + cells > width && used > 0 {
                    lines.push(Line::default());
                    used = 0;
                }
                lines.last_mut().unwrap().spans.push(Span::styled(
                    ch.to_string(),
                    Style::default().fg(color(hue)),
                ));
                used += cells;
            }
        }
    }
    lines
}
struct Controls<'a> {
    catalog: Option<&'a crate::palettes::Catalog>,
    settings: &'a Settings,
    selected: usize,
    paused: bool,
    path: &'a std::path::Path,
    status: &'a SaveStatus,
    pending: bool,
}
impl Controls<'_> {
    #[cfg(test)]
    fn lines(&self, height: u16) -> Vec<Line<'static>> {
        self.lines_for_size(height, usize::MAX)
    }
    fn lines_for_size(&self, height: u16, width: usize) -> Vec<Line<'static>> {
        let yellow = Style::default().fg(color('y'));
        let orange = Style::default().fg(color('o'));
        let mut lines = control_help(width, self.paused);
        let extra_help = lines.len().saturating_sub(1).min(u16::MAX as usize) as u16;
        for i in settings_window(height.saturating_sub(extra_help), self.selected) {
            let selected = i == self.selected;
            let mut spans = vec![
                Span::styled(if selected { "> " } else { "  " }, yellow),
                Span::styled(
                    format!("{}: ", FIELDS[i]),
                    if selected {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    },
                ),
            ];
            match i {
                9 | 12 => {
                    let enabled = if i == 9 {
                        self.settings.canvas_relative
                    } else {
                        self.settings.per_glyph_colors
                    };
                    spans.push(Span::styled(
                        if enabled { "On" } else { "Off" },
                        Style::default().fg(if enabled { color('g') } else { color('s') }),
                    ));
                }
                10 => {
                    let name = self
                        .catalog
                        .and_then(|catalog| catalog.label(&self.settings.colors))
                        .unwrap_or_else(|| value(self.settings, i));
                    spans.extend(crate::palettes::preview(&name, &self.settings.colors));
                }
                _ => spans.push(Span::styled(
                    value(self.settings, i),
                    match i {
                        8 => yellow,
                        11 => Style::default().fg(color('g')),
                        _ => orange,
                    },
                )),
            }
            lines.push(Line::from(spans).style(if selected {
                Style::default().bg(Color::Rgb(35, 35, 45))
            } else {
                Style::default()
            }));
        }
        let name = self
            .path
            .file_stem()
            .filter(|name| !name.is_empty())
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "default".into());
        let mut status = vec![
            Span::styled("Config Profile: ", Style::default().fg(color('m'))),
            Span::styled(name, yellow),
        ];
        if self.pending {
            status.push(Span::styled(" · Pending", yellow));
        } else {
            match self.status {
                SaveStatus::Config => {}
                SaveStatus::Saved => {
                    status.push(Span::styled(" · Saved", Style::default().fg(color('g'))))
                }
                SaveStatus::Failed(error) => {
                    status.push(Span::styled(
                        " · Save failed: ",
                        Style::default().fg(color('r')),
                    ));
                    status.push(Span::raw(error.clone()));
                }
            }
        }
        lines.push(Line::from(status));
        lines
    }
}
impl Widget for Controls<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.lines_for_size(area.height, area.width.saturating_sub(2) as usize))
            .style(Style::default().fg(color('w')).bg(Color::Rgb(17, 17, 17)))
            .block(
                Block::default()
                    .title(Line::styled(
                        "termatrix 3 · live settings",
                        Style::default().fg(color('m')),
                    ))
                    .border_style(Style::default().fg(color('s')))
                    .borders(Borders::ALL),
            )
            .render(area, buf);
    }
}
fn panel_regions(area: Rect) -> (Rect, Option<Rect>) {
    if area.height < 7 {
        return (area, None);
    }
    if area.width < 96 {
        let controls = Rect::new(area.x, area.y, area.width, area.height - 2);
        return (
            controls,
            Some(Rect::new(area.x, controls.bottom(), area.width, 2)),
        );
    }
    let controls = Rect::new(area.x, area.y, 72, area.height);
    (
        controls,
        Some(Rect::new(
            controls.right(),
            area.y,
            area.width - 72,
            area.height,
        )),
    )
}
fn plot_regions(area: Rect) -> Vec<Rect> {
    if area.width < 40 || area.height < 7 {
        return vec![];
    }
    let count = (area.width / 40).clamp(1, 3);
    let mut x = area.x;
    (0..count)
        .map(|i| {
            let width = if i == count - 1 {
                area.right() - x
            } else {
                area.width / count
            };
            let rect = Rect::new(x, area.y, width, area.height);
            x += width;
            rect
        })
        .collect()
}

fn chart_time_labels(compression: f64, width: u16) -> Vec<Span<'static>> {
    let count = if width >= 60 { 6 } else { 4 };
    (0..count)
        .map(|i| {
            let age = crate::metrics::position_age(i as f64 / (count - 1) as f64, compression);
            let label = if i == count - 1 {
                "now".into()
            } else if age < 1. {
                format!("-{age:.1}s")
            } else {
                let seconds = age.round() as u64;
                match (seconds / 60, seconds % 60) {
                    (0, s) => format!("-{s}s"),
                    (m, 0) => format!("-{m}m"),
                    (m, s) => format!("-{m}m{s}s"),
                }
            };
            Span::raw(label)
        })
        .collect()
}
fn density_chart<'a>(
    points: &'a [(f64, f64)],
    current: f64,
    compression: f64,
    width: u16,
) -> Chart<'a> {
    metric_chart(
        points,
        (
            format!("Visible glyph density {current:.1}%"),
            100.,
            true,
            'g',
        ),
        compression,
        width,
    )
}
fn metric_chart<'a>(
    points: &'a [(f64, f64)],
    (title, maximum, percent, hue): (String, f64, bool, char),
    compression: f64,
    width: u16,
) -> Chart<'a> {
    let value_start = title
        .find(|c: char| c.is_ascii_digit())
        .unwrap_or(title.len());
    let title = Line::from(vec![
        Span::styled(
            title[..value_start].to_owned(),
            Style::default().fg(color('m')),
        ),
        Span::styled(
            title[value_start..].to_owned(),
            Style::default().fg(color(if percent { 'o' } else { hue })),
        ),
    ]);
    Chart::new(vec![Dataset::default()
        .data(points)
        .marker(ratatui::symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(color(hue)))])
    .style(Style::default().fg(color('s')).bg(Color::Rgb(17, 17, 17)))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color('s')))
            .title(title),
    )
    .x_axis(
        Axis::default()
            .bounds([0., 1.])
            .labels(chart_time_labels(compression, width))
            .style(Style::default().fg(color('s'))),
    )
    .y_axis(
        Axis::default()
            .bounds([0., maximum])
            .labels(
                [0., maximum / 2., maximum]
                    .map(|v| format!("{v:.0}{}", if percent { "%" } else { "" })),
            )
            .style(Style::default().fg(color('s'))),
    )
}
fn performance(
    area: Rect,
    buf: &mut Buffer,
    density: &crate::metrics::DensityHistory,
    metrics: &crate::metrics::PerformanceHistory,
    now: f64,
    compression: f64,
    traces: usize,
) {
    let summary = Line::from(vec![
        Span::styled(format!("{traces} traces"), Style::default().fg(color('o'))),
        Span::raw(" | "),
        Span::styled(
            format!("{:.1} FPS", metrics.fps.current()),
            Style::default().fg(color('y')),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("{:.2} ms work", metrics.work.current()),
            Style::default().fg(color('b')),
        ),
    ]);
    let summary_area = Rect::new(area.x, area.y, area.width, 2.min(area.height));
    Paragraph::new(summary)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render(summary_area, buf);
    let plots = plot_regions(Rect::new(
        area.x,
        summary_area.bottom(),
        area.width,
        area.height - summary_area.height,
    ));
    for (i, rect) in plots.into_iter().enumerate() {
        if i == 1 {
            let points = density.points(now, compression);
            density_chart(&points, density.current(), compression, rect.width).render(rect, buf);
        } else {
            let (history, title, hue) = if i == 2 {
                (&metrics.traces, format!("Active traces {traces}"), 'o')
            } else {
                (
                    &metrics.work,
                    format!("Frame work {:.2} ms", metrics.work.current()),
                    'b',
                )
            };
            let points = history.points(now, compression);
            let peak = points.iter().map(|p| p.1).fold(1., f64::max);
            let maximum = (peak * 1.1).ceil();
            metric_chart(
                &points,
                (title, maximum, false, hue),
                compression,
                rect.width,
            )
            .render(rect, buf);
        }
    }
}
fn change_profile(
    settings: &mut Settings,
    path: &mut PathBuf,
    saving: &mut SaveState,
    destination: &std::path::Path,
    create: bool,
) -> io::Result<()> {
    if destination == path && !create {
        return Ok(());
    }
    // Validate the destination before writing or leaving the current profile.
    let next = if create {
        settings.clone()
    } else {
        persistence::load_required(destination)?
    };
    saving.finish(path, settings)?;
    if create {
        persistence::create(destination, &next)?;
    }
    *settings = next;
    *path = destination.into();
    *saving = SaveState::default();
    Ok(())
}
pub fn run(mut s: Settings, mut path: PathBuf, seed: u64, theme: Theme) -> io::Result<()> {
    let appearance = Appearance::detect(
        theme,
        &std::env::var("TERM").unwrap_or_default(),
        &std::env::var("COLORTERM").unwrap_or_default(),
        &std::env::var("COLORFGBG").unwrap_or_default(),
    );
    let catalog_path = directories::ProjectDirs::from("", "", "termatrix")
        .ok_or_else(|| io::Error::other("cannot locate palette configuration directory"))?
        .config_dir()
        .join("palettes.toml");
    let mut catalog = crate::palettes::Catalog::load(&catalog_path)?;
    let mut palette_manager: Option<crate::palettes::Manager> = None;
    let rain_colors = RainColors::new(appearance);
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = restore_output(&mut stdout());
        previous(info);
    }));
    enable_raw_mode()?;
    let _restore = Restore;
    execute!(stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    let mut scene = Scene::new(seed);
    let mut selected: usize = 0;
    let mut paused = false;
    let mut last = Instant::now();
    let mut next_frame = last;
    let mut saving = SaveState::default();
    let started = Instant::now();
    let mut density = crate::metrics::DensityHistory::default();
    let mut metrics = crate::metrics::PerformanceHistory::default();
    let mut pending_work = Duration::ZERO;
    let mut picker: Option<crate::profiles::Picker> = None;
    loop {
        let work_started = Instant::now();
        let now = work_started;
        let size = terminal.size()?;
        let (_, canvas) = regions(Rect::new(0, 0, size.width, size.height), s.panel);
        scene.resize(canvas.width as usize / 2, canvas.height as usize);
        if !paused {
            scene.advance(now.duration_since(last).as_secs_f64(), &s);
        }
        last = now;
        density.sample(now.duration_since(started).as_secs_f64(), &scene);
        if saving
            .changed
            .is_some_and(|t| now.duration_since(t) >= Duration::from_millis(400))
        {
            saving.attempt(&path, &s);
        }
        metrics.sample(
            now.duration_since(started).as_secs_f64(),
            scene.traces.len(),
        );
        if now >= next_frame {
            let mut preview = s.clone();
            if let Some(manager) = &palette_manager {
                if let Some(codes) = manager.preview_codes(&catalog) {
                    preview.colors = codes.into();
                }
            }
            scene.refresh_colors(&preview);
            let elapsed = now.duration_since(started).as_secs_f64();
            terminal.draw(|f| {
                let (panel, canvas) = regions(f.area(), s.panel);
                f.render_widget(ThemedRain(&scene, &rain_colors), canvas);
                if s.panel {
                    let (controls, chart) = panel_regions(panel);
                    if let Some(area) = chart {
                        performance(
                            area,
                            f.buffer_mut(),
                            &density,
                            &metrics,
                            elapsed,
                            s.chart_compression,
                            scene.traces.len(),
                        );
                    }
                    f.render_widget(
                        Controls {
                            catalog: Some(&catalog),
                            settings: &s,
                            selected,
                            paused,
                            path: &path,
                            status: &saving.status,
                            pending: saving.changed.is_some(),
                        },
                        controls,
                    );
                }
                if let Some(manager) = &palette_manager {
                    let a = f.area();
                    let width = a.width.min(76);
                    let height = a.height.min(22);
                    let area = Rect::new(
                        a.x + (a.width - width) / 2,
                        a.y + (a.height - height) / 2,
                        width,
                        height,
                    );
                    manager.render(area, f.buffer_mut(), &catalog);
                }
                if let Some(picker) = &picker {
                    let a = f.area();
                    let width = a.width.min(72);
                    let height = a.height.min(18);
                    let area = Rect::new(
                        a.x + (a.width - width) / 2,
                        a.y + (a.height - height) / 2,
                        width,
                        height,
                    );
                    picker.render(
                        area,
                        f.buffer_mut(),
                        crate::profiles::PickerColors {
                            title: color('m'),
                            key: color('y'),
                            text: color('w'),
                            profile: color('b'),
                            create: color('g'),
                            error: color('r'),
                        },
                    );
                }
            })?;
            metrics.frame((pending_work + work_started.elapsed()).as_secs_f64());
            pending_work = Duration::ZERO;
            next_frame = now + Duration::from_secs_f64(1. / s.max_refresh_rate as f64);
        } else {
            pending_work += work_started.elapsed();
        }
        if event::poll(
            next_frame
                .saturating_duration_since(Instant::now())
                .min(Duration::from_millis(10)),
        )? {
            let input = event::read()?;
            if let Event::Key(k) = input {
                if k.kind != KeyEventKind::Release
                    && palette_manager.is_some()
                    && !(k.code == KeyCode::Char('c')
                        && k.modifiers.contains(KeyModifiers::CONTROL))
                {
                    let before = s.colors.clone();
                    if palette_manager.as_mut().unwrap().key(
                        k,
                        &mut catalog,
                        &catalog_path,
                        &mut s.colors,
                    ) {
                        palette_manager = None;
                    }
                    if s.colors != before {
                        saving.mark_changed();
                    }
                    next_frame = Instant::now();
                    continue;
                }
                if k.kind != KeyEventKind::Release
                    && picker.is_some()
                    && !(k.code == KeyCode::Char('c')
                        && k.modifiers.contains(KeyModifiers::CONTROL))
                {
                    let action = picker.as_mut().unwrap().key(k);
                    match action {
                        Some(crate::profiles::Action::Close) => picker = None,
                        Some(action) => {
                            let (destination, create) = match action {
                                crate::profiles::Action::Load(p) => (p, false),
                                crate::profiles::Action::Create(p) => (p, true),
                                crate::profiles::Action::Close => unreachable!(),
                            };
                            match change_profile(
                                &mut s,
                                &mut path,
                                &mut saving,
                                &destination,
                                create,
                            ) {
                                Ok(()) => picker = None,
                                Err(e) => picker.as_mut().unwrap().error = Some(e.to_string()),
                            }
                        }
                        None => {}
                    }
                    next_frame = Instant::now();
                    continue;
                }
            }
            match input {
                Event::Key(k) if k.kind != KeyEventKind::Release => match k.code {
                    KeyCode::Esc | KeyCode::Char('q') => break,
                    KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Char('c') => {
                        palette_manager = Some(crate::palettes::Manager::open(&catalog, &s.colors));
                        next_frame = Instant::now();
                    }
                    KeyCode::Char('p') => {
                        match crate::profiles::Picker::open(&path) {
                            Ok(opened) => picker = Some(opened),
                            Err(e) => {
                                saving.status =
                                    SaveStatus::Failed(format!("Cannot list profiles: {e}"))
                            }
                        }
                        next_frame = Instant::now();
                    }
                    KeyCode::Char('?') => {
                        s.panel = !s.panel;
                        saving.mark_changed();
                        next_frame = Instant::now();
                    }
                    KeyCode::Char(' ') => paused = !paused,
                    KeyCode::Up if s.panel => {
                        selected = (selected + FIELDS.len() - 1) % FIELDS.len()
                    }
                    KeyCode::Down if s.panel => selected = (selected + 1) % FIELDS.len(),
                    KeyCode::Left | KeyCode::Right if s.panel => {
                        if selected == 10 {
                            s.colors = catalog
                                .cycle(&s.colors, if k.code == KeyCode::Right { 1 } else { -1 });
                        } else {
                            edit(
                                &mut s,
                                selected,
                                if k.code == KeyCode::Right { 1. } else { -1. },
                            );
                        }
                        saving.mark_changed();
                    }
                    _ => {}
                },
                Event::Resize(_, _) => {
                    terminal.clear()?;
                    next_frame = Instant::now();
                }
                _ => {}
            }
        }
    }
    saving.finish(&path, &s)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_switch_saves_pending_and_loads_or_copies_settings() {
        let dir = tempfile::tempdir().unwrap();
        let original = dir.path().join("default.toml");
        let destination = dir.path().join("night.toml");
        let mut settings = Settings {
            colors: "rR".into(),
            ..Settings::default()
        };
        let mut path = original.clone();
        let mut saving = SaveState::default();
        saving.mark_changed();
        change_profile(&mut settings, &mut path, &mut saving, &destination, true).unwrap();
        assert_eq!(persistence::load_required(&original).unwrap().colors, "rR");
        assert_eq!(
            persistence::load_required(&destination).unwrap().colors,
            "rR"
        );
        assert_eq!(path, destination);
        assert!(!saving.unsaved);
        settings.colors = "bB".into();
        saving.mark_changed();
        change_profile(&mut settings, &mut path, &mut saving, &original, false).unwrap();
        assert_eq!(settings.colors, "rR");
        assert_eq!(
            persistence::load_required(&destination).unwrap().colors,
            "bB"
        );
    }
    #[test]
    fn failed_profile_changes_preserve_active_path_settings_and_pending_save() {
        let dir = tempfile::tempdir().unwrap();
        let original = dir.path().join("default.toml");
        let invalid = dir.path().join("broken.toml");
        std::fs::write(&invalid, "bad = [").unwrap();
        let mut settings = Settings::default();
        let mut path = original.clone();
        let mut saving = SaveState::default();
        saving.mark_changed();
        for (destination, create) in [
            (&invalid, false),
            (&invalid, true),
            (&dir.path().join("missing.toml"), false),
        ] {
            assert!(
                change_profile(&mut settings, &mut path, &mut saving, destination, create).is_err()
            );
            assert_eq!(path, original);
            assert_eq!(settings.colors, "gG");
            assert!(saving.unsaved);
        }
        // Force the old profile save to fail without relying on permissions.
        let blocked = dir.path().join("blocked");
        std::fs::write(&blocked, "file").unwrap();
        path = blocked.join("default.toml");
        let destination = dir.path().join("valid.toml");
        persistence::save(&destination, &Settings::default()).unwrap();
        assert!(
            change_profile(&mut settings, &mut path, &mut saving, &destination, false).is_err()
        );
        assert_eq!(path, blocked.join("default.toml"));
        assert!(saving.unsaved);
        let new = dir.path().join("new.toml");
        assert!(change_profile(&mut settings, &mut path, &mut saving, &new, true).is_err());
        assert!(!new.exists());
    }
    #[test]
    fn performance_layout_uses_remaining_width_and_adds_plots() {
        for (width, expected) in [
            (24, 0),
            (39, 0),
            (40, 1),
            (79, 1),
            (80, 2),
            (119, 2),
            (120, 3),
            (200, 3),
        ] {
            let area = Rect::new(72, 2, width, 10);
            let plots = plot_regions(area);
            assert_eq!(plots.len(), expected);
            if let Some(last) = plots.last() {
                assert_eq!(last.right(), area.right());
                assert!(plots.iter().all(|r| r.width >= 40));
                for pair in plots.windows(2) {
                    assert_eq!(pair[0].right(), pair[1].x);
                }
            }
        }
        let mut scene = Scene::new(42);
        scene.resize(10, 10);
        let mut density = crate::metrics::DensityHistory::default();
        density.sample(0., &scene);
        let mut metrics = crate::metrics::PerformanceHistory::default();
        metrics.frame(0.002);
        metrics.sample(0.25, 12);
        for width in [24, 40, 80, 120] {
            let area = Rect::new(0, 0, width, 12);
            let mut buffer = Buffer::empty(area);
            performance(area, &mut buffer, &density, &metrics, 0.25, 6., 12);
            let text: String = buffer.content.iter().map(|c| c.symbol()).collect();
            assert!(text.contains("12 traces"));
            if width >= 40 {
                assert!(text.contains("4.0 FPS"));
            }
            assert_eq!(text.contains("Frame work 2.00 ms"), width >= 40);
            assert_eq!(text.contains("Visible glyph density"), width >= 80);
            assert_eq!(text.contains("Active traces 12"), width >= 120);
            if width >= 120 {
                assert!(
                    text.find("Frame work").unwrap() < text.find("Visible glyph density").unwrap()
                );
                assert!(
                    text.find("Visible glyph density").unwrap()
                        < text.find("Active traces").unwrap()
                );
            }
        }
    }
    #[test]
    fn chart_axis_labels_follow_compression_and_available_width() {
        let labels = |c, w| {
            chart_time_labels(c, w)
                .iter()
                .map(|s| s.content.to_string())
                .collect::<Vec<_>>()
        };
        assert_eq!(labels(0., 40), vec!["-10m", "-6m40s", "-3m20s", "now"]);
        assert_eq!(labels(6., 40), vec!["-10m", "-1m20s", "-10s", "now"]);
        assert_eq!(labels(6., 60).len(), 6);
        let mut settings = Settings::default();
        edit(&mut settings, 14, -1.);
        assert_eq!(settings.chart_compression, 5.5);
        for _ in 0..30 {
            edit(&mut settings, 14, -1.);
        }
        assert_eq!(value(&settings, 14), "0.0 (linear)");
    }
    #[test]
    fn refresh_cap_displays_hertz() {
        let settings = Settings::default();
        assert_eq!(value(&settings, 8), "60 Hz");
        let controls = Controls {
            catalog: None,
            settings: &settings,
            selected: 8,

            paused: false,
            path: std::path::Path::new("default.toml"),
            status: &SaveStatus::Config,
            pending: false,
        };
        let lines = controls.lines(12);
        let row = lines
            .iter()
            .find(|l| l.to_string().starts_with("> Refresh cap:"))
            .unwrap();
        assert_eq!(row.to_string(), "> Refresh cap: 60 Hz");
        assert_eq!(row.spans.last().unwrap().style.fg, Some(color('y')));
    }
    #[test]
    fn unchanged_and_successful_saves_do_not_write_again_on_exit() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("default.toml");
        let settings = Settings::default();
        let mut state = SaveState::default();
        state.finish(&path, &settings).unwrap();
        assert!(!path.exists());
        state.mark_changed();
        state.attempt(&path, &settings);
        assert!(matches!(state.status, SaveStatus::Saved));
        std::fs::remove_file(&path).unwrap();
        state.finish(&path, &settings).unwrap();
        assert!(!path.exists());
    }
    #[test]
    fn failed_autosave_is_retried_on_exit() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("default.toml");
        std::fs::create_dir(&path).unwrap();
        let settings = Settings::default();
        let mut state = SaveState::default();
        state.mark_changed();
        state.attempt(&path, &settings);
        assert!(matches!(state.status, SaveStatus::Failed(_)));
        assert!(state.finish(&path, &settings).is_err());
        std::fs::remove_dir(&path).unwrap();
        state.finish(&path, &settings).unwrap();
        assert!(path.is_file());
    }
    #[test]
    fn terminal_cleanup_resets_style_before_leaving_screen() {
        let mut bytes = Vec::new();
        restore_output(&mut bytes).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("\x1b[0m"));
        assert!(text.find("\x1b[0m").unwrap() < text.find("\x1b[?1049l").unwrap());
        assert!(text.contains("\x1b[?25h"));
    }
    #[test]
    fn rendered_frame_handles_partial_writes_and_output_failures() {
        use ratatui::backend::Backend;
        use std::io::Write;
        struct Writer {
            bytes: Vec<u8>,
            limit: usize,
            failure: Option<io::ErrorKind>,
            fail_flush: bool,
        }
        impl Write for Writer {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                if let Some(kind) = self.failure {
                    return Err(io::Error::from(kind));
                }
                let n = bytes.len().min(self.limit);
                self.bytes.extend_from_slice(&bytes[..n]);
                Ok(n)
            }
            fn flush(&mut self) -> io::Result<()> {
                if self.fail_flush {
                    Err(io::Error::other("flush failed"))
                } else {
                    Ok(())
                }
            }
        }
        let mut scene = Scene::new(42);
        scene.resize(4, 10);
        for _ in 0..200 {
            scene.advance(0.01, &Settings::default());
        }
        let area = Rect::new(0, 0, 8, 10);
        let mut frame = Buffer::empty(area);
        Rain(&scene).render(area, &mut frame);
        let cells = || {
            frame
                .content
                .iter()
                .enumerate()
                .map(|(i, c)| ((i % 8) as u16, (i / 8) as u16, c))
        };
        let mut full = Vec::new();
        CrosstermBackend::new(&mut full).draw(cells()).unwrap();
        let mut writer = Writer {
            bytes: vec![],
            limit: 1,
            failure: None,
            fail_flush: false,
        };
        CrosstermBackend::new(&mut writer).draw(cells()).unwrap();
        assert_eq!(writer.bytes, full);
        for kind in [io::ErrorKind::BrokenPipe, io::ErrorKind::Other] {
            writer.failure = Some(kind);
            assert_eq!(
                CrosstermBackend::new(&mut writer)
                    .draw(cells())
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
        writer.failure = None;
        writer.limit = 0;
        assert_eq!(
            CrosstermBackend::new(&mut writer)
                .draw(cells())
                .unwrap_err()
                .kind(),
            io::ErrorKind::WriteZero
        );
        writer.fail_flush = true;
        assert!(Backend::flush(&mut CrosstermBackend::new(&mut writer)).is_err());
    }
    #[test]
    fn narrow_foreground_occludes_wide_background_without_stale_cells() {
        let mut scene = Scene::new(42);
        scene.resize(1, 30);
        for _ in 0..200 {
            scene.advance(0.01, &Settings::default());
        }
        let mut back = scene.traces[0].clone();
        back.depth = 0.9;
        back.id = 1;
        back.glyphs[0].symbol = '界';
        let row = back.head_row() as u16;
        let mut front = back.clone();
        front.depth = 0.1;
        front.id = 2;
        front.glyphs.truncate(1);
        front.glyphs[0].symbol = 'A';
        scene.traces = vec![back.clone(), front];
        let area = Rect::new(0, 0, 2, 30);
        let mut buffer = Buffer::empty(area);
        Rain(&scene).render(area, &mut buffer);
        assert_eq!(buffer[(0, row)].symbol(), "A");
        assert_eq!(buffer[(1, row)].symbol(), " ");
        scene.traces = vec![back];
        Rain(&scene).render(area, &mut buffer);
        assert_eq!(buffer[(0, row)].symbol(), "界");
    }
    #[test]
    fn individual_profiles_have_two_distinct_shades() {
        for (name, codes, dark_rgb) in [
            ("Blue", "bB", (0, 119, 170)),
            ("Purple", "mM", (102, 34, 204)),
            ("Yellow", "yY", (204, 153, 0)),
            ("Orange", "oO", (187, 85, 0)),
        ] {
            let s = Settings {
                colors: codes.into(),
                ..Settings::default()
            };
            s.validate().unwrap();
            assert_eq!(value(&s, 10), name);
            let light = s.palette_color(0.25);
            let dark = s.palette_color(0.75);
            assert_ne!(color(light), color(dark));
            assert_eq!(color(dark), Color::Rgb(dark_rgb.0, dark_rgb.1, dark_rgb.2));
        }
    }
    #[test]
    fn named_color_profiles_are_selectable_and_valid() {
        for (name, colors) in [
            ("Green", "gG"),
            ("Blue", "bB"),
            ("Red", "rR"),
            ("Purple", "mM"),
            ("Yellow", "yY"),
            ("Orange", "oO"),
        ] {
            let mut s = Settings {
                colors: colors.into(),
                ..Settings::default()
            };
            s.validate().unwrap();
            assert_eq!(value(&s, 10), name);
            edit(&mut s, 10, 1.);
            edit(&mut s, 10, -1.);
            assert_eq!(s.colors, colors);
        }
        assert_eq!(color('R'), Color::Rgb(170, 17, 0));
        assert_ne!(color('R'), color('r'));
    }
    #[test]
    fn named_mixed_palettes_color_the_name_without_duplicate_labels() {
        for (name, codes) in [
            ("Rainbow", "roygbm"),
            ("Christmas", "rgw"),
            ("Orange-green-blue", "ogb"),
        ] {
            assert!(PALETTES.contains(&(name, codes)));
            let settings = Settings {
                colors: codes.into(),
                ..Settings::default()
            };
            let controls = Controls {
                catalog: None,
                settings: &settings,
                selected: 10,
                paused: false,
                path: std::path::Path::new("default.toml"),
                status: &SaveStatus::Config,
                pending: false,
            };
            let lines = controls.lines(12);
            let line = lines
                .iter()
                .find(|l| l.to_string().starts_with("> Palette:"))
                .unwrap();
            assert_eq!(line.to_string(), format!("> Palette: {name}"));
            for code in codes.chars() {
                assert!(line
                    .spans
                    .iter()
                    .skip(2)
                    .any(|span| span.style.fg == Some(color(code))));
            }
            if !name.contains('-') {
                for (span, code) in line.spans.iter().skip(2).zip(codes.chars().cycle()) {
                    assert_eq!(span.style.fg, Some(color(code)));
                }
            }
        }
    }
    #[test]
    fn shared_custom_palette_name_is_styled_in_controls_even_when_hidden() {
        let mut catalog = crate::palettes::Catalog::default();
        catalog.custom.push(crate::palettes::Palette {
            name: "Sunset".into(),
            colors: "rrroY".into(),
        });
        catalog.hidden.push("Sunset".into());
        let settings = Settings {
            colors: "rrroY".into(),
            ..Settings::default()
        };
        let controls = Controls {
            catalog: Some(&catalog),
            settings: &settings,
            selected: 10,
            paused: false,
            path: std::path::Path::new("default.toml"),
            status: &SaveStatus::Config,
            pending: false,
        };
        let lines = controls.lines(12);
        let palette = lines
            .iter()
            .find(|l| l.to_string().starts_with("> Palette:"))
            .unwrap();
        assert_eq!(palette.to_string(), "> Palette: Sunset");
        assert_eq!(palette.spans[2].style.fg, Some(color('r')));
        assert_eq!(palette.spans[5].style.fg, Some(color('o')));
        assert_eq!(palette.spans[6].style.fg, Some(color('Y')));
    }
    #[test]
    fn individual_palette_previews_show_one_primary_name() {
        for (name, codes) in PALETTES {
            if !matches!(codes, "gG" | "bB" | "sS" | "rR" | "mM" | "yY" | "oO") {
                continue;
            }
            let settings = Settings {
                colors: codes.into(),
                ..Settings::default()
            };
            let controls = Controls {
                catalog: None,
                settings: &settings,
                selected: 10,
                paused: false,
                path: std::path::Path::new("default.toml"),
                status: &SaveStatus::Config,
                pending: false,
            };
            let lines = controls.lines(12);
            let line = lines
                .iter()
                .find(|line| line.to_string().starts_with("> Palette:"))
                .unwrap();
            assert_eq!(line.to_string(), format!("> Palette: {name}"));
            assert_eq!(
                line.spans.last().unwrap().style.fg,
                Some(color(codes.chars().next().unwrap()))
            );
        }
    }
    #[test]
    fn grey_profile_and_preview() {
        let mut s = Settings {
            colors: "bB".into(),
            ..Settings::default()
        };
        edit(&mut s, 10, 1.);
        assert_eq!(s.colors, "sS");
        assert_eq!(color('s'), Color::Rgb(170, 170, 170));
        assert_eq!(color('S'), Color::Rgb(119, 119, 119));
        s.validate().unwrap();
        assert_eq!(value(&s, 10), "Grey");
        assert_eq!(s.palette_color(0.25), 's');
        assert_eq!(s.palette_color(0.75), 'S');
        let controls = Controls {
            catalog: None,
            settings: &s,
            selected: 10,

            paused: false,
            path: std::path::Path::new("default.toml"),
            status: &SaveStatus::Config,
            pending: false,
        };
        let lines = controls.lines(12);
        let palette = lines
            .iter()
            .find(|l| l.to_string().starts_with("> Palette:"))
            .unwrap();
        assert_eq!(palette.to_string(), "> Palette: Grey");
        assert!(palette
            .spans
            .iter()
            .any(|s| s.content == "Grey" && s.style.fg == Some(color('s'))));
    }
    #[test]
    fn chart_layout_and_braille_rendering() {
        for width in [0, 80, 111, 112, 160, 240] {
            let area = Rect::new(2, 3, width, 12);
            let (controls, chart) = panel_regions(area);
            let chart = chart.unwrap();
            if width < 96 {
                assert_eq!(controls.width, area.width);
                assert_eq!(chart.y, controls.bottom());
                assert_eq!(chart.height, 2);
            } else {
                assert_eq!(controls.width, 72);
                assert_eq!(chart.x, controls.right());
                assert_eq!(chart.right(), area.right());
            }
        }
        assert!(panel_regions(Rect::new(0, 0, 120, 6)).1.is_none());
        let area = Rect::new(0, 0, 60, 12);
        let mut buffer = Buffer::empty(area);
        density_chart(&[(0., 0.), (0.5, 70.), (1., 25.)], 25., 6., 60).render(area, &mut buffer);
        let text: String = buffer.content.iter().map(|c| c.symbol()).collect();
        assert!(text.contains("Visible glyph density 25.0%"));
        assert!(text.chars().any(|c| ('\u{2801}'..='\u{28ff}').contains(&c)));
    }
    #[test]
    fn speed_controls_clamp_at_compiled_limits() {
        assert_eq!(&FIELDS[..3], &["Min speed", "Max speed", "Speed bias"]);
        let mut s = Settings {
            min_speed: MIN_TRACE_SPEED,
            max_speed: MAX_TRACE_SPEED,
            ..Settings::default()
        };
        edit(&mut s, 0, -1.);
        edit(&mut s, 1, 1.);
        assert_eq!(s.min_speed, MIN_TRACE_SPEED);
        assert_eq!(s.max_speed, MAX_TRACE_SPEED);
    }
    #[test]
    fn rendered_tails_use_explicit_rgb_without_dim_and_preserve_accents() {
        let mut scene = Scene::new(42);
        scene.resize(1, 100);
        let settings = Settings {
            density: 1.,
            min_speed: 5.,
            max_speed: 5.,
            min_length: 8,
            max_length: 8,
            ..Settings::default()
        };
        for _ in 0..80 {
            scene.advance(0.05, &settings);
        }
        let trace = scene
            .traces
            .iter()
            .find(|t| t.glyphs.len() >= 6)
            .unwrap()
            .clone();
        scene.traces = vec![trace];
        scene.traces[0].glyphs.truncate(6);
        let area = Rect::new(0, 0, 2, 100);
        for code in "gGrRbBmMyYoOsScwk".chars() {
            for glyph in &mut scene.traces[0].glyphs {
                glyph.color = code;
                glyph.symbol = 'X';
                glyph.highlighted = false;
            }
            scene.traces[0].glyphs[4].highlighted = true;
            let mut buffer = Buffer::empty(area);
            Rain(&scene).render(area, &mut buffer);
            let row = scene.traces[0].head_row() as u16;
            let base = color(code);
            let appearance = Appearance {
                light: false,
                depth: 24,
            };
            assert_ne!(buffer[(0, row - 5)].fg, Color::Rgb(0, 0, 0));
            assert!(!buffer[(0, row - 5)].modifier.contains(Modifier::DIM));
            assert_eq!(
                buffer[(0, row - 1)].fg,
                appearance.foreground(base, 0.2, false)
            );
            assert_eq!(buffer[(0, row)].fg, Color::Rgb(255, 255, 255));
            assert!(buffer[(0, row)].modifier.contains(Modifier::BOLD));
            assert_eq!(
                buffer[(0, row - 4)].fg,
                appearance.foreground(base, 0.8, true)
            );
            assert!(buffer[(0, row - 4)].modifier.contains(Modifier::BOLD));
        }
    }
    #[test]
    fn rain_preserves_terminal_background_in_both_themes() {
        let scene = Scene::new(42);
        let area = Rect::new(0, 0, 4, 4);
        for light in [false, true] {
            for background in [Color::Reset, Color::Rgb(23, 31, 42)] {
                let mut buffer = Buffer::empty(area);
                buffer.set_style(area, Style::default().bg(background));
                ThemedRain(&scene, &RainColors::new(Appearance { light, depth: 24 }))
                    .render(area, &mut buffer);
                assert!(buffer.content.iter().all(|cell| cell.bg == background));
            }
        }
    }
    #[test]
    fn cached_fades_preserve_exact_colors_and_reuse_entries() {
        for light in [false, true] {
            for depth in [16, 256, 24] {
                let appearance = Appearance { light, depth };
                let cache = RainColors::new(appearance);
                for pass in 0..2 {
                    let before = cache.reduced.borrow().len();
                    for code in "bBcgGkmMrRwyYoOsS".chars() {
                        for length in [1, 2, 4, 24, 127, 512] {
                            for index in 0..length {
                                let p = index as f64 / (length - 1).max(1) as f64;
                                for accent in [false, true] {
                                    assert_eq!(
                                        cache.foreground(color(code), p, accent),
                                        appearance.foreground(color(code), p, accent)
                                    );
                                }
                            }
                        }
                    }
                    let after = cache.reduced.borrow().len();
                    if pass == 1 {
                        assert_eq!(before, after);
                    }
                    if depth == 24 {
                        assert_eq!(after, 0);
                    } else {
                        assert!(after > 0);
                    }
                }
            }
        }
    }
    #[test]
    fn fades_follow_theme_and_capability() {
        for light in [false, true] {
            let a = Appearance { light, depth: 24 };
            let head = if light { 0 } else { 255 };
            let tail = 255 - head;
            for code in "gGrRbBmMyYoOsScwk".chars() {
                let base = color(code);
                assert_eq!(a.foreground(base, 0., false), Color::Rgb(head, head, head));
                assert_eq!(a.foreground(base, 0.3, false), base);
                assert_ne!(a.foreground(base, 1., false), Color::Rgb(tail, tail, tail));
                assert_ne!(a.foreground(base, 0.15, false), base);
                assert_ne!(
                    a.foreground(base, 0.8, true),
                    a.foreground(base, 0.8, false)
                );
                for depth in [16, 256] {
                    for step in 0..=100 {
                        let c =
                            Appearance { light, depth }.foreground(base, step as f64 / 100., false);
                        assert!(matches!(c, Color::Indexed(i) if (i as u16) < depth));
                        if step == 100 {
                            let Color::Indexed(i) = c else { unreachable!() };
                            assert_ne!(indexed_rgb(i), [tail; 3]);
                        }
                    }
                }
            }
        }
        for (term, ct, depth) in [
            ("xterm", "", 16),
            ("xterm-256color", "", 256),
            ("xterm", "truecolor", 24),
            ("xterm-direct", "", 24),
        ] {
            assert_eq!(
                Appearance::detect(Theme::Auto, term, ct, "0;15").depth,
                depth
            );
        }
        assert!(Appearance::detect(Theme::Auto, "", "", "0;15").light);
        assert!(!Appearance::detect(Theme::Auto, "", "", "invalid").light);
        assert!(!Appearance::detect(Theme::Dark, "", "", "0;15").light);
        assert!(Appearance::detect(Theme::Light, "", "", "15;0").light);
    }
    #[test]
    fn green_shades_and_highlight_have_explicit_contrast() {
        assert_eq!(color('g'), Color::Rgb(102, 187, 51));
        assert_eq!(color('G'), Color::Rgb(51, 136, 0));
        assert_eq!(brighten(color('G')), Color::Rgb(122, 178, 89));
        assert_ne!(brighten(color('g')), color('g'));
    }
    #[test]
    fn complete_palette_uses_explicit_rgb() {
        for (code, rgb) in [
            ('r', (221, 68, 51)),
            ('o', (238, 136, 51)),
            ('y', (221, 204, 51)),
            ('g', (102, 187, 51)),
            ('b', (51, 170, 221)),
            ('m', (153, 85, 255)),
            ('s', (170, 170, 170)),
            ('c', (51, 187, 204)),
            ('w', (238, 238, 238)),
            ('k', (34, 34, 34)),
        ] {
            assert_eq!(color(code), Color::Rgb(rgb.0, rgb.1, rgb.2));
        }
    }
    #[test]
    fn yellow_uses_shared_dark_palette() {
        assert_eq!(color('y'), Color::Rgb(221, 204, 51));
        assert_ne!(color('y'), color('o'));
    }
    #[test]
    fn orange_uses_shared_dark_palette() {
        assert_eq!(color('o'), Color::Rgb(238, 136, 51));
        assert_ne!(color('o'), color('y'));
    }
    #[test]
    fn control_help_keeps_all_commands_visible_at_panel_widths() {
        let settings = Settings::default();
        for width in [32, 40, 60, 72, 80, 120] {
            for paused in [false, true] {
                let controls = Controls {
                    catalog: None,
                    settings: &settings,
                    selected: 14,
                    paused,
                    path: std::path::Path::new("default.toml"),
                    status: &SaveStatus::Saved,
                    pending: false,
                };
                let area = Rect::new(0, 0, width, 12);
                let mut buffer = Buffer::empty(area);
                controls.render(area, &mut buffer);
                let text = buffer
                    .content
                    .iter()
                    .map(|c| c.symbol())
                    .collect::<String>();
                for command in [
                    "? controls",
                    "p profiles",
                    "c palettes",
                    "↑↓ select",
                    "←→ adjust",
                    "Space pause",
                    "q quit",
                ] {
                    assert!(
                        text.contains(command),
                        "missing {command} at width {width}, paused={paused}"
                    );
                }
                assert!(text.contains("> Chart compression:"));
                assert!(text.contains("Config Profile: default"));
                if paused {
                    assert!(text.contains("Paused"));
                }
            }
        }
    }
    #[test]
    fn controls_validate() {
        let mut s = Settings::default();
        for i in 0..FIELDS.len() {
            for d in [-1., 1.] {
                for _ in 0..100 {
                    edit(&mut s, i, d);
                    s.validate().unwrap();
                }
            }
        }
    }
    #[test]
    fn tiny_layout() {
        for h in [0, 1, 7, 8, 11, 12, 13, 20, 24, 60] {
            let (p, c) = regions(Rect::new(2, 3, 10, h), true);
            assert_eq!(p.height, h.min(12));
            assert_eq!(p.height + c.height, h);
            assert_eq!(c.y, p.bottom());
            let (hidden, full) = regions(Rect::new(2, 3, 10, h), false);
            assert_eq!(hidden.height, 0);
            assert_eq!(full, Rect::new(2, 3, 10, h));
        }
    }
    #[test]
    fn panel_shows_eight_settings_and_keeps_selection_visible() {
        for height in 0..=12 {
            for selected in 0..FIELDS.len() {
                let window = settings_window(height, selected);
                assert_eq!(window.len(), height.saturating_sub(4) as usize);
                if !window.is_empty() {
                    assert!(window.contains(&selected));
                }
            }
        }
        assert_eq!(settings_window(12, 0), 0..8);
        assert_eq!(settings_window(12, 14), 7..15);
    }

    #[test]
    fn frame_contains_glyphs() {
        let mut scene = Scene::new(1);
        scene.resize(10, 10);
        for _ in 0..100 {
            scene.advance(0.02, &Settings::default());
        }
        let mut terminal = Terminal::new(ratatui::backend::TestBackend::new(20, 10)).unwrap();
        terminal
            .draw(|f| f.render_widget(Rain(&scene), f.area()))
            .unwrap();
        assert!(terminal
            .backend()
            .buffer()
            .content
            .iter()
            .any(|c| c.symbol() != " "));
    }
    #[test]
    fn control_styles_preserve_plain_text_and_selection() {
        let s = Settings::default();
        let controls = Controls {
            catalog: None,
            settings: &s,
            selected: 0,

            paused: true,
            path: std::path::Path::new("config/settings.toml"),
            status: &SaveStatus::Saved,
            pending: false,
        };
        let lines = controls.lines(12);
        assert_eq!(lines[1].to_string(), "> Min speed: 2.00");
        assert!(lines[0].to_string().ends_with("quit | Paused"));
        assert!(!lines[0].to_string().contains("traces"));
        assert_eq!(
            lines.last().unwrap().to_string(),
            "Config Profile: settings · Saved"
        );
        let area = Rect::new(0, 0, 120, 12);
        let mut styled = Buffer::empty(area);
        let mut plain = Buffer::empty(area);
        Paragraph::new(lines.clone()).render(area, &mut styled);
        let plain_lines: Vec<_> = lines.iter().map(|l| Line::raw(l.to_string())).collect();
        Paragraph::new(plain_lines).render(area, &mut plain);
        assert_eq!(
            styled
                .content
                .iter()
                .map(|c| c.symbol())
                .collect::<Vec<_>>(),
            plain.content.iter().map(|c| c.symbol()).collect::<Vec<_>>()
        );
        assert_eq!(styled[(13, 1)].fg, color('o'));
        assert_eq!(styled[(0, 1)].bg, Color::Rgb(35, 35, 45));
        assert!(styled[(2, 1)].modifier.contains(Modifier::BOLD));
    }
    #[test]
    fn profile_label_uses_filename_stem_without_exposing_directory() {
        let settings = Settings::default();
        for (path, name) in [
            ("profiles/default.toml", "default"),
            ("/private/config/Christmas rain.toml", "Christmas rain"),
            ("profiles/night.v2.toml", "night.v2"),
            ("custom", "custom"),
            ("profiles/雨.toml", "雨"),
            ("", "default"),
        ] {
            let controls = Controls {
                catalog: None,
                settings: &settings,
                selected: 0,
                paused: false,
                path: std::path::Path::new(path),
                status: &SaveStatus::Config,
                pending: false,
            };
            assert_eq!(
                controls.lines(12).last().unwrap().to_string(),
                format!("Config Profile: {name}")
            );
        }
    }
    #[test]
    fn palette_toggle_and_save_status_have_text_and_color() {
        let s = Settings {
            colors: "mgb".into(),
            per_glyph_colors: false,
            ..Settings::default()
        };
        let failure = SaveStatus::Failed("disk full".into());
        let mut controls = Controls {
            catalog: None,
            settings: &s,
            selected: 10,

            paused: false,
            path: std::path::Path::new("settings.toml"),
            status: &failure,
            pending: false,
        };
        let lines = controls.lines(12);
        let palette = lines
            .iter()
            .find(|l| l.to_string().starts_with("> Palette:"))
            .unwrap();
        assert_eq!(palette.to_string(), "> Palette: Purple-green-blue");
        for (name, fg) in [
            ("Purple", color('m')),
            ("green", color('g')),
            ("blue", color('b')),
        ] {
            assert!(palette
                .spans
                .iter()
                .any(|span| span.content == name && span.style.fg == Some(fg)));
        }
        assert!(lines
            .iter()
            .any(|l| l.to_string() == "  Individual glyph colors: Off"));
        assert_eq!(
            lines.last().unwrap().to_string(),
            "Config Profile: settings · Save failed: disk full"
        );
        assert_eq!(lines.last().unwrap().spans[2].style.fg, Some(color('r')));
        controls.pending = true;
        let pending = controls.lines(12);
        assert_eq!(
            pending.last().unwrap().to_string(),
            "Config Profile: settings · Pending"
        );
        assert_eq!(pending.last().unwrap().spans[2].style.fg, Some(color('y')));
    }
}
