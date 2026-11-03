//! Profile discovery and keyboard state; switching is coordinated by the terminal.
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy)]
pub struct PickerColors {
    pub title: Color,
    pub key: Color,
    pub text: Color,
    pub profile: Color,
    pub create: Color,
    pub error: Color,
}
pub enum Action {
    Close,
    Load(PathBuf),
    Create(PathBuf),
}
pub struct Picker {
    directory: PathBuf,
    entries: Vec<PathBuf>,
    selected: usize,
    name: Option<String>,
    pub error: Option<String>,
}
fn directory(current: &Path) -> PathBuf {
    current
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .into()
}
fn label(path: &Path) -> String {
    path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .chars()
        .map(|c| if c.is_control() { '�' } else { c })
        .collect()
}
pub fn new_path(directory: &Path, name: &str) -> io::Result<PathBuf> {
    let upper = name.to_ascii_uppercase();
    if name.is_empty()
        || name.chars().count() > 64
        || name.trim() != name
        || !name
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_'))
        || matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ["COM", "LPT"].iter().any(|prefix| {
            upper
                .strip_prefix(prefix)
                .is_some_and(|n| n.len() == 1 && matches!(n.as_bytes()[0], b'1'..=b'9'))
        })
    {
        return Err(io::Error::other(
            "Use 1–64 letters, digits, spaces, - or _; avoid reserved device names.",
        ));
    }
    Ok(directory.join(format!("{name}.toml")))
}
impl Picker {
    pub fn open(current: &Path) -> io::Result<Self> {
        let directory = directory(current);
        let mut entries = Vec::new();
        match fs::read_dir(&directory) {
            Ok(files) => {
                for file in files {
                    let file = file?;
                    if file.file_type()?.is_file()
                        && file.path().extension().is_some_and(|e| e == "toml")
                    {
                        entries.push(file.path());
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        entries.sort_by_key(|p| label(p));
        let selected = entries
            .iter()
            .position(|p| p.file_name() == current.file_name())
            .unwrap_or(0);
        Ok(Self {
            directory,
            entries,
            selected,
            name: None,
            error: None,
        })
    }
    pub fn key(&mut self, key: KeyEvent) -> Option<Action> {
        if key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
        {
            return None;
        }
        if let Some(name) = &mut self.name {
            match key.code {
                KeyCode::Esc => {
                    self.name = None;
                    self.error = None;
                }
                KeyCode::Backspace => {
                    name.pop();
                    self.error = None;
                }
                KeyCode::Char(c) if !c.is_control() && name.chars().count() < 64 => {
                    name.push(c);
                    self.error = None;
                }
                KeyCode::Enter => match new_path(&self.directory, name) {
                    Ok(path) => return Some(Action::Create(path)),
                    Err(e) => self.error = Some(e.to_string()),
                },
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Esc | KeyCode::Char('p') => return Some(Action::Close),
                KeyCode::Up => {
                    self.selected = (self.selected + self.entries.len()) % (self.entries.len() + 1)
                }
                KeyCode::Down => self.selected = (self.selected + 1) % (self.entries.len() + 1),
                KeyCode::Char('n') => {
                    self.name = Some(String::new());
                    self.error = None;
                }
                KeyCode::Enter if self.selected == self.entries.len() => {
                    self.name = Some(String::new());
                    self.error = None;
                }
                KeyCode::Enter => return Some(Action::Load(self.entries[self.selected].clone())),
                _ => {}
            }
        }
        None
    }
    pub fn render(&self, area: Rect, buf: &mut Buffer, colors: PickerColors) {
        Clear.render(area, buf);
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Config Profiles")
            .style(Style::default().fg(colors.title).bg(Color::Rgb(17, 17, 17)));
        let inner = block.inner(area);
        block.render(area, buf);
        let mut hints = Vec::new();
        for (key, description) in [
            ("↑↓", " select · "),
            ("Enter", " load · "),
            ("n", " new · "),
            ("Esc", " back"),
        ] {
            hints.push(Span::styled(key, Style::default().fg(colors.key)));
            hints.push(Span::styled(description, Style::default().fg(colors.text)));
        }
        let mut lines = vec![Line::from(hints)];
        if let Some(name) = &self.name {
            lines.push(Line::from(vec![
                Span::styled("New profile", Style::default().fg(colors.create)),
                Span::styled(
                    " copies current settings. ",
                    Style::default().fg(colors.text),
                ),
                Span::styled("Enter", Style::default().fg(colors.key)),
                Span::styled(" saves.", Style::default().fg(colors.text)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Name: ", Style::default().fg(colors.text)),
                Span::styled(name.clone(), Style::default().fg(colors.profile)),
                Span::styled("▏", Style::default().fg(colors.key)),
            ]));
        } else {
            let count = inner.height.saturating_sub(2) as usize;
            let first = self.selected.saturating_sub(count.saturating_sub(1));
            for i in (first..=self.entries.len()).take(count) {
                let (name, color) = self.entries.get(i).map_or_else(
                    || ("+ Create new profile…".into(), colors.create),
                    |p| (label(p), colors.profile),
                );
                let line = Line::from(vec![
                    Span::styled(
                        if i == self.selected { "> " } else { "  " },
                        Style::default().fg(colors.key),
                    ),
                    Span::styled(name, Style::default().fg(color)),
                ]);
                lines.push(if i == self.selected {
                    line.style(Style::default().bg(Color::Rgb(35, 35, 45)))
                } else {
                    line
                });
            }
        }
        if let Some(error) = &self.error {
            lines.push(Line::styled(
                format!("Error: {error}"),
                Style::default().fg(colors.error),
            ));
        }
        Paragraph::new(lines)
            .style(Style::default().fg(colors.text))
            .render(inner, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const COLORS: PickerColors = PickerColors {
        title: Color::Magenta,
        key: Color::Yellow,
        text: Color::White,
        profile: Color::Blue,
        create: Color::Green,
        error: Color::Red,
    };
    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }
    #[test]
    fn picker_renders_selected_profile_creation_and_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("night.toml");
        fs::write(&path, "").unwrap();
        let mut picker = Picker::open(&path).unwrap();
        let area = Rect::new(0, 0, 72, 10);
        let render = |picker: &Picker| {
            let mut buf = Buffer::empty(area);
            picker.render(area, &mut buf, COLORS);
            buf
        };
        let text = |buf: &Buffer| buf.content.iter().map(|c| c.symbol()).collect::<String>();
        let assert_color = |buf: &Buffer, token: &str, color: Color| {
            assert!(
                buf.content.windows(token.chars().count()).any(|cells| {
                    cells.iter().map(|c| c.symbol()).collect::<String>() == token
                        && cells.iter().all(|c| c.fg == color)
                }),
                "missing styled token {token:?}"
            );
        };
        let list = render(&picker);
        assert_color(&list, "Config Profiles", Color::Magenta);
        for x in 0..area.width {
            assert_eq!(list[(x, 0)].fg, COLORS.title);
            assert_eq!(list[(x, area.height - 1)].fg, COLORS.title);
        }
        for y in 0..area.height {
            assert_eq!(list[(0, y)].fg, COLORS.title);
            assert_eq!(list[(area.width - 1, y)].fg, COLORS.title);
        }
        assert!(text(&list).contains("> night"));
        assert_color(&list, "Enter", Color::Yellow);
        assert_color(&list, " select", Color::White);
        assert_color(&list, "night", Color::Blue);
        assert_color(&list, "+ Create new profile…", Color::Green);
        assert!(list
            .content
            .iter()
            .any(|c| c.symbol() == "n" && c.fg == Color::Blue && c.bg == Color::Rgb(35, 35, 45)));
        picker.key(key(KeyCode::Char('n')));
        picker.key(key(KeyCode::Char('x')));
        let creation = render(&picker);
        assert!(text(&creation).contains("Name: x"));
        assert_color(&creation, "Name: ", Color::White);
        assert_color(&creation, "x", Color::Blue);
        assert_color(&creation, "New profile", Color::Green);
        assert_color(&creation, "Enter", Color::Yellow);
        picker.error = Some("Already exists".into());
        let buf = render(&picker);
        assert!(text(&buf).contains("Error: Already exists"));
        assert!(buf
            .content
            .iter()
            .any(|c| c.symbol() == "E" && c.fg == Color::Red));
    }
    #[test]
    fn profile_names_are_portable_and_cannot_escape_directory() {
        for name in ["night", "Christmas rain", "雨", "blue-green_2"] {
            assert_eq!(
                new_path(Path::new("profiles"), name).unwrap(),
                Path::new("profiles").join(format!("{name}.toml"))
            );
        }
        for name in [
            "",
            ".",
            "..",
            "../escape",
            "/tmp/test",
            "a\\b",
            "a.toml",
            " x",
            "x ",
            "a\n",
            "CON",
            "nul",
            "COM1",
            "LPT9",
        ] {
            assert!(new_path(Path::new("profiles"), name).is_err(), "{name:?}");
        }
        assert!(new_path(Path::new("profiles"), &"a".repeat(65)).is_err());
    }
    #[test]
    fn discovery_filters_sorts_and_selects_current_with_scrolling() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["z.toml", "a.toml", "ignored.txt"] {
            fs::write(dir.path().join(name), "").unwrap();
        }
        fs::create_dir(dir.path().join("directory.toml")).unwrap();
        let mut picker = Picker::open(&dir.path().join("z.toml")).unwrap();
        assert_eq!(picker.entries.len(), 2);
        assert_eq!(picker.selected, 1);
        assert!(
            matches!(picker.key(key(KeyCode::Enter)), Some(Action::Load(p)) if p.ends_with("z.toml"))
        );
        picker.key(key(KeyCode::Down));
        picker.key(key(KeyCode::Down));
        assert_eq!(picker.selected, 0);
        picker.key(key(KeyCode::Up));
        assert_eq!(picker.selected, 2);
        for width in [0, 10, 72] {
            let area = Rect::new(0, 0, width, 5);
            picker.render(area, &mut Buffer::empty(area), COLORS);
        }
    }
    #[test]
    fn empty_directory_creation_editing_errors_and_cancellation() {
        let dir = tempfile::tempdir().unwrap();
        let current = dir.path().join("missing/default.toml");
        let mut picker = Picker::open(&current).unwrap();
        assert!(picker.entries.is_empty());
        picker.key(key(KeyCode::Up));
        picker.key(key(KeyCode::Enter));
        assert!(picker.key(key(KeyCode::Enter)).is_none());
        assert!(picker.error.is_some());
        for c in "nightx".chars() {
            picker.key(key(KeyCode::Char(c)));
        }
        picker.key(key(KeyCode::Backspace));
        assert!(
            matches!(picker.key(key(KeyCode::Enter)), Some(Action::Create(p)) if p == current.parent().unwrap().join("night.toml"))
        );
        picker.key(key(KeyCode::Esc));
        assert!(picker.name.is_none());
        assert!(matches!(picker.key(key(KeyCode::Esc)), Some(Action::Close)));
        picker.key(key(KeyCode::Char('n')));
        assert!(picker.name.is_some());
        assert!(picker
            .key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL))
            .is_none());
        assert_eq!(picker.name.as_deref(), Some(""));
    }
}
