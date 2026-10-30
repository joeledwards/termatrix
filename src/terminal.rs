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
use unicode_width::UnicodeWidthChar;
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
const FIELDS: [&str; 14] = [
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
const PALETTES: [(&str, &str); 11] = [
    ("Green", "gG"),
    ("Rainbow", "roygcbm"),
    ("Purple-green-blue", "mgb"),
    ("Orange-green-blue", "ogb"),
    ("Christmas", "rgw"),
    ("Blue", "bB"),
    ("Grey", "sS"),
    ("Red", "rR"),
    ("Purple", "mM"),
    ("Yellow", "yY"),
    ("Orange", "oO"),
];
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
        8 => s.max_refresh_rate.to_string(),
        9 => s.canvas_relative.to_string(),
        10 => PALETTES
            .iter()
            .find(|(_, colors)| *colors == s.colors)
            .map(|(name, _)| name.to_string())
            .unwrap_or_else(|| format!("Custom ({})", s.colors)),
        12 => s.per_glyph_colors.to_string(),
        13 => format!("{:.0}%", s.highlight_probability * 100.),
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
fn color(c: char) -> Color {
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

fn brighten(c: Color) -> Color {
    match c {
        Color::Rgb(r, g, b) => {
            let lift = |v: u8| v + ((255 - v) as f64 * 0.35).round() as u8;
            Color::Rgb(lift(r), lift(g), lift(b))
        }
        _ => c,
    }
}
struct Rain<'a>(&'a Scene);
impl Widget for Rain<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for sample in termatrix::projection::orthographic(self.0) {
            let x = area.x as usize + sample.column * 2;
            if x + 1 >= area.right() as usize || sample.row >= area.height as usize {
                continue;
            }
            let y = area.y + sample.row as u16;
            use termatrix::projection::Highlight;
            let style = match sample.highlight {
                Highlight::Head => Style::default().fg(color('w')).add_modifier(Modifier::BOLD),
                Highlight::Tail => Style::default()
                    .fg(color(sample.color))
                    .add_modifier(Modifier::DIM),
                Highlight::Accent => Style::default()
                    .fg(brighten(color(sample.color)))
                    .add_modifier(Modifier::BOLD),
                Highlight::Body => Style::default().fg(color(sample.color)),
            };
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
struct Controls<'a> {
    settings: &'a Settings,
    selected: usize,
    traces: usize,
    paused: bool,
    path: &'a std::path::Path,
    status: &'a SaveStatus,
    pending: bool,
}
impl Controls<'_> {
    fn lines(&self, height: u16) -> Vec<Line<'static>> {
        let grey = Style::default().fg(color('s'));
        let yellow = Style::default().fg(color('y'));
        let orange = Style::default().fg(color('o'));
        let mut hints = Vec::new();
        for (key, meaning) in [
            ("?", " controls | "),
            ("↑↓", " select "),
            ("←→", " adjust | "),
            ("Space", " pause | "),
            ("q", " quit | "),
        ] {
            hints.push(Span::styled(key, yellow));
            hints.push(Span::styled(meaning, grey));
        }
        hints.push(Span::styled(self.traces.to_string(), orange));
        hints.push(Span::styled(" traces", grey));
        if self.paused {
            hints.push(Span::styled(" | Paused", yellow));
        }
        let mut lines = vec![Line::from(hints)];
        for i in settings_window(height, self.selected) {
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
                    spans.push(Span::raw(format!("{} · ", value(self.settings, i))));
                    for (index, c) in self.settings.colors.chars().enumerate() {
                        if index > 0 {
                            spans.push(Span::styled(" · ", grey));
                        }
                        let name = match c {
                            'b' => "Blue",
                            'B' => "Dark blue",
                            'c' => "Cyan",
                            'g' => "Green",
                            'G' => "Dark green",
                            'k' => "Black",
                            'm' => "Purple",
                            'M' => "Dark purple",
                            'r' => "Red",
                            'R' => "Dark red",
                            'w' => "White",
                            's' => "Light grey",
                            'S' => "Dark grey",
                            'y' => "Yellow",
                            'Y' => "Dark yellow",
                            'o' => "Orange",
                            'O' => "Dark orange",
                            _ => "?",
                        };
                        spans.push(Span::styled(name, Style::default().fg(color(c))));
                    }
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
        let (label, style) = if self.pending {
            ("Pending: ", yellow)
        } else {
            match self.status {
                SaveStatus::Config => ("Config: ", grey),
                SaveStatus::Saved => ("Saved: ", Style::default().fg(color('g'))),
                SaveStatus::Failed(error) => {
                    lines.push(Line::from(vec![
                        Span::styled("Save failed: ", Style::default().fg(color('r'))),
                        Span::raw(error.clone()),
                    ]));
                    return lines;
                }
            }
        };
        let mut status = vec![Span::styled(label, style)];
        if let Some(parent) = self.path.parent().filter(|p| !p.as_os_str().is_empty()) {
            let mut prefix = parent.to_string_lossy().into_owned();
            if !prefix.ends_with(std::path::MAIN_SEPARATOR) {
                prefix.push(std::path::MAIN_SEPARATOR);
            }
            status.push(Span::styled(prefix, Style::default().fg(color('m'))));
        }
        if let Some(name) = self.path.file_name() {
            status.push(Span::styled(name.to_string_lossy().into_owned(), yellow));
        }
        lines.push(Line::from(status));
        lines
    }
}
impl Widget for Controls<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.lines(area.height))
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
    if area.width < 112 || area.height < 7 {
        return (area, None);
    }
    let chart_width = (area.width / 3).clamp(40, 60);
    let controls = Rect::new(area.x, area.y, area.width - chart_width, area.height);
    (
        controls,
        Some(Rect::new(
            controls.right(),
            area.y,
            chart_width,
            area.height,
        )),
    )
}
fn density_chart<'a>(points: &'a [(f64, f64)], current: f64) -> Chart<'a> {
    Chart::new(vec![Dataset::default()
        .data(points)
        .marker(ratatui::symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(color('g')))])
    .style(Style::default().fg(color('s')).bg(Color::Rgb(17, 17, 17)))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color('s')))
            .title(Line::from(vec![
                Span::styled("Visible glyph density ", Style::default().fg(color('m'))),
                Span::styled(format!("{current:.1}%"), Style::default().fg(color('o'))),
            ])),
    )
    .x_axis(
        Axis::default()
            .bounds([-60., 0.])
            .labels(["-60s", "-30s", "now"])
            .style(Style::default().fg(color('s'))),
    )
    .y_axis(
        Axis::default()
            .bounds([0., 100.])
            .labels(["0%", "50%", "100%"])
            .style(Style::default().fg(color('s'))),
    )
}
pub fn run(mut s: Settings, path: PathBuf, seed: u64) -> io::Result<()> {
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
    loop {
        let now = Instant::now();
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
        if now >= next_frame {
            scene.refresh_colors(&s);
            let elapsed = now.duration_since(started).as_secs_f64();
            let points = density.points(elapsed);
            terminal.draw(|f| {
                let (panel, canvas) = regions(f.area(), s.panel);
                f.render_widget(Rain(&scene), canvas);
                if s.panel {
                    let (controls, chart) = panel_regions(panel);
                    if let Some(area) = chart {
                        f.render_widget(density_chart(&points, density.current()), area);
                    }
                    f.render_widget(
                        Controls {
                            settings: &s,
                            selected,
                            traces: scene.traces.len(),
                            paused,
                            path: &path,
                            status: &saving.status,
                            pending: saving.changed.is_some(),
                        },
                        controls,
                    );
                }
            })?;
            next_frame = now + Duration::from_secs_f64(1. / s.max_refresh_rate as f64);
        }
        if event::poll(
            next_frame
                .saturating_duration_since(Instant::now())
                .min(Duration::from_millis(10)),
        )? {
            match event::read()? {
                Event::Key(k) if k.kind != KeyEventKind::Release => match k.code {
                    KeyCode::Esc | KeyCode::Char('q') => break,
                    KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => break,
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
                        edit(
                            &mut s,
                            selected,
                            if k.code == KeyCode::Right { 1. } else { -1. },
                        );
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
            settings: &s,
            selected: 10,
            traces: 0,
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
        assert_eq!(
            palette.to_string(),
            "> Palette: Grey · Light grey · Dark grey"
        );
        assert!(palette
            .spans
            .iter()
            .any(|s| s.content == "Light grey" && s.style.fg == Some(color('s'))));
    }
    #[test]
    fn chart_layout_and_braille_rendering() {
        for width in [0, 80, 111, 112, 160, 240] {
            let area = Rect::new(2, 3, width, 12);
            let (controls, chart) = panel_regions(area);
            if width < 112 {
                assert_eq!(controls, area);
                assert!(chart.is_none());
            } else {
                let chart = chart.unwrap();
                assert!(controls.width >= 72);
                assert_eq!(chart.x, controls.right());
                assert_eq!(chart.right(), area.right());
            }
        }
        assert!(panel_regions(Rect::new(0, 0, 120, 6)).1.is_none());
        let area = Rect::new(0, 0, 60, 12);
        let mut buffer = Buffer::empty(area);
        density_chart(&[(-60., 0.), (-30., 70.), (0., 25.)], 25.).render(area, &mut buffer);
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
        assert_eq!(settings_window(12, 13), 6..14);
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
            settings: &s,
            selected: 0,
            traces: 12,
            paused: true,
            path: std::path::Path::new("config/settings.toml"),
            status: &SaveStatus::Saved,
            pending: false,
        };
        let lines = controls.lines(12);
        assert_eq!(lines[1].to_string(), "> Min speed: 2.00");
        assert!(lines[0].to_string().ends_with("12 traces | Paused"));
        assert_eq!(
            lines.last().unwrap().to_string(),
            "Saved: config/settings.toml"
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
    fn palette_toggle_and_save_status_have_text_and_color() {
        let s = Settings {
            colors: "mgb".into(),
            per_glyph_colors: false,
            ..Settings::default()
        };
        let failure = SaveStatus::Failed("disk full".into());
        let mut controls = Controls {
            settings: &s,
            selected: 10,
            traces: 0,
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
        assert_eq!(
            palette.to_string(),
            "> Palette: Purple-green-blue · Purple · Green · Blue"
        );
        for (name, fg) in [
            ("Purple", color('m')),
            ("Green", color('g')),
            ("Blue", color('b')),
        ] {
            assert!(palette
                .spans
                .iter()
                .any(|span| span.content == name && span.style.fg == Some(fg)));
        }
        assert!(lines
            .iter()
            .any(|l| l.to_string() == "  Individual glyph colors: Off"));
        assert_eq!(lines.last().unwrap().to_string(), "Save failed: disk full");
        assert_eq!(lines.last().unwrap().spans[0].style.fg, Some(color('r')));
        controls.pending = true;
        let pending = controls.lines(12);
        assert_eq!(
            pending.last().unwrap().to_string(),
            "Pending: settings.toml"
        );
        assert_eq!(pending.last().unwrap().spans[0].style.fg, Some(color('y')));
    }
}

#[cfg(test)]
#[path = "preview.rs"]
mod preview;
