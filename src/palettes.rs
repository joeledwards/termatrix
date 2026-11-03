//! Shared palette catalog and transactional manager. Profiles retain raw color codes.
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::Path,
};

pub const BUILTINS: [(&str, &str); 11] = [
    ("Green", "gG"),
    ("Rainbow", "roygbm"),
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
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Palette {
    pub name: String,
    pub colors: String,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Catalog {
    pub custom: Vec<Palette>,
    pub hidden: Vec<String>,
}
impl Catalog {
    pub fn entries(&self) -> Vec<Palette> {
        BUILTINS
            .iter()
            .map(|(name, colors)| Palette {
                name: (*name).into(),
                colors: (*colors).into(),
            })
            .chain(self.custom.iter().cloned())
            .collect()
    }
    pub fn visible(&self, name: &str) -> bool {
        !self.hidden.iter().any(|n| n == name)
    }
    pub fn validate(&self) -> io::Result<()> {
        if self.custom.len() > 128 {
            return Err(io::Error::other("At most 128 custom palettes"));
        }
        let entries = self.entries();
        let mut names = std::collections::HashSet::new();
        for p in &entries {
            if p.name.is_empty()
                || p.name.trim() != p.name
                || p.name.chars().count() > 64
                || p.name.chars().any(char::is_control)
                || !names.insert(p.name.to_lowercase())
            {
                return Err(io::Error::other(
                    "Use a unique name of 1–64 printable characters, without surrounding spaces",
                ));
            }
            if p.colors.is_empty()
                || p.colors.len() > 256
                || !p.colors.chars().all(|c| "bBcgGkmMrRwyYoOsS".contains(c))
            {
                return Err(io::Error::other("Use 1–256 color codes: bBcgGkmMrRwyYoOsS"));
            }
        }
        let mut hidden = std::collections::HashSet::new();
        if self
            .hidden
            .iter()
            .any(|n| !entries.iter().any(|p| p.name == *n) || !hidden.insert(n))
        {
            return Err(io::Error::other(
                "Hidden palette names must exist and be unique",
            ));
        }
        if !entries.iter().any(|p| self.visible(&p.name)) {
            return Err(io::Error::other("Keep at least one palette visible"));
        }
        Ok(())
    }
    pub fn load(path: &Path) -> io::Result<Self> {
        let catalog: Self = match fs::read_to_string(path) {
            Ok(s) => toml::from_str(&s).map_err(io::Error::other)?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => Self::default(),
            Err(e) => return Err(e),
        };
        catalog.validate()?;
        Ok(catalog)
    }
    pub fn save(&self, path: &Path) -> io::Result<()> {
        self.validate()?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        fs::create_dir_all(parent)?;
        let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
        tmp.write_all(
            toml::to_string_pretty(self)
                .map_err(io::Error::other)?
                .as_bytes(),
        )?;
        tmp.as_file().sync_all()?;
        tmp.persist(path).map_err(|e| e.error)?;
        Ok(())
    }
    pub fn cycle(&self, current: &str, direction: i32) -> String {
        let entries: Vec<_> = self
            .entries()
            .into_iter()
            .filter(|p| self.visible(&p.name))
            .collect();
        let index = entries.iter().position(|p| p.colors == current);
        let next = match index {
            Some(i) => (i as i32 + direction).rem_euclid(entries.len() as i32) as usize,
            None if direction < 0 => entries.len() - 1,
            None => 0,
        };
        entries[next].colors.clone()
    }
    pub fn label(&self, codes: &str) -> Option<String> {
        self.entries()
            .into_iter()
            .find(|p| p.colors == codes)
            .map(|p| p.name)
    }
}

/// Color the label itself. Explicit hue names get whole-word coloring; other
/// labels distribute the weighted colors across letters.
pub fn preview(name: &str, codes: &str) -> Vec<Span<'static>> {
    let codes: Vec<_> = codes.chars().collect();
    if codes.is_empty() {
        return vec![Span::raw(name.to_owned())];
    }
    let single = codes.iter().all(|c| c.eq_ignore_ascii_case(&codes[0]));
    if single {
        return vec![Span::styled(
            name.to_owned(),
            Style::default().fg(crate::terminal::color(codes[0])),
        )];
    }
    let parts: Vec<_> = name.split('-').collect();
    if parts.len() == codes.len() {
        let mut spans = Vec::new();
        for (i, (part, c)) in parts.iter().zip(&codes).enumerate() {
            if i > 0 {
                spans.push(Span::raw("-"));
            }
            spans.push(Span::styled(
                (*part).to_owned(),
                Style::default().fg(crate::terminal::color(*c)),
            ));
        }
        return spans;
    }
    name.chars()
        .enumerate()
        .map(|(i, c)| {
            Span::styled(
                c.to_string(),
                Style::default().fg(crate::terminal::color(codes[i % codes.len()])),
            )
        })
        .collect()
}
struct Editor {
    original: Option<usize>,
    name: String,
    codes: String,
    codes_field: bool,
}
pub struct Manager {
    selected: usize,
    editor: Option<Editor>,
    deleting: bool,
    pub error: Option<String>,
}
impl Manager {
    pub fn open(catalog: &Catalog, codes: &str) -> Self {
        Self {
            selected: catalog
                .entries()
                .iter()
                .position(|p| p.colors == codes)
                .unwrap_or(0),
            editor: None,
            deleting: false,
            error: None,
        }
    }
    pub fn preview_codes<'a>(&'a self, catalog: &'a Catalog) -> Option<&'a str> {
        if let Some(editor) = &self.editor {
            let codes = editor.codes.as_str();
            return (!codes.is_empty() && codes.chars().all(|c| "bBcgGkmMrRwyYoOsS".contains(c)))
                .then_some(codes);
        }
        if self.selected < BUILTINS.len() {
            Some(BUILTINS[self.selected].1)
        } else {
            catalog
                .custom
                .get(self.selected - BUILTINS.len())
                .map(|p| p.colors.as_str())
        }
    }
    /// Returns true when closing. Apply mutations only after durable persistence.
    pub fn key(
        &mut self,
        key: KeyEvent,
        catalog: &mut Catalog,
        path: &Path,
        active: &mut String,
    ) -> bool {
        if key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
        {
            return false;
        }
        self.error = None;
        let mut next = catalog.clone();
        if let Some(editor) = &mut self.editor {
            match key.code {
                KeyCode::Esc => {
                    self.editor = None;
                    return false;
                }
                KeyCode::Tab | KeyCode::Up | KeyCode::Down => {
                    editor.codes_field = !editor.codes_field
                }
                KeyCode::Backspace => {
                    if editor.codes_field {
                        editor.codes.pop();
                    } else {
                        editor.name.pop();
                    }
                }
                KeyCode::Char(c) if !c.is_control() => {
                    if editor.codes_field && editor.codes.len() < 256 {
                        editor.codes.push(c);
                    } else if !editor.codes_field && editor.name.chars().count() < 64 {
                        editor.name.push(c);
                    }
                }
                KeyCode::Enter => {
                    let p = Palette {
                        name: editor.name.clone(),
                        colors: editor.codes.clone(),
                    };
                    if let Some(i) = editor.original {
                        let old = next.custom[i].name.clone();
                        for hidden in &mut next.hidden {
                            if *hidden == old {
                                *hidden = p.name.clone();
                            }
                        }
                        next.custom[i] = p;
                    } else {
                        next.custom.push(p);
                    }
                    match next.save(path) {
                        Ok(()) => {
                            self.selected =
                                BUILTINS.len() + editor.original.unwrap_or(next.custom.len() - 1);
                            *catalog = next;
                            self.editor = None;
                        }
                        Err(e) => self.error = Some(e.to_string()),
                    }
                }
                _ => {}
            }
            return false;
        }
        let entries = catalog.entries();
        let entry = &entries[self.selected];
        if self.deleting {
            self.deleting = false;
            if key.code != KeyCode::Char('y') {
                return false;
            }
            next.custom.remove(self.selected - BUILTINS.len());
            next.hidden.retain(|name| *name != entry.name);
        } else {
            match key.code {
                KeyCode::Esc | KeyCode::Char('c') => return true,
                KeyCode::Up => {
                    self.selected = (self.selected + entries.len() - 1) % entries.len();
                    return false;
                }
                KeyCode::Down => {
                    self.selected = (self.selected + 1) % entries.len();
                    return false;
                }
                KeyCode::Enter => {
                    *active = entry.colors.clone();
                    return true;
                }
                KeyCode::Char('n') => {
                    self.editor = Some(Editor {
                        original: None,
                        name: String::new(),
                        codes: active.clone(),
                        codes_field: false,
                    });
                    return false;
                }
                KeyCode::Char('e') if self.selected >= BUILTINS.len() => {
                    self.editor = Some(Editor {
                        original: Some(self.selected - BUILTINS.len()),
                        name: entry.name.clone(),
                        codes: entry.colors.clone(),
                        codes_field: false,
                    });
                    return false;
                }
                KeyCode::Char('d') if self.selected >= BUILTINS.len() => {
                    self.deleting = true;
                    return false;
                }
                KeyCode::Char(' ') => {
                    if next.visible(&entry.name) {
                        next.hidden.push(entry.name.clone());
                    } else {
                        next.hidden.retain(|n| *n != entry.name);
                    }
                }
                _ => return false,
            }
        }
        match next.save(path) {
            Ok(()) => {
                *catalog = next;
                self.selected = self.selected.min(catalog.entries().len() - 1);
            }
            Err(e) => self.error = Some(e.to_string()),
        }
        false
    }
    pub fn render(&self, area: Rect, buf: &mut Buffer, catalog: &Catalog) {
        Clear.render(area, buf);
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Palettes")
            .style(
                Style::default()
                    .fg(crate::terminal::color('w'))
                    .bg(Color::Rgb(17, 17, 17)),
            );
        let inner = block.inner(area);
        block.render(area, buf);
        let hint = |text: &str| {
            Line::styled(
                text.to_owned(),
                Style::default().fg(crate::terminal::color('y')),
            )
        };
        let mut lines = vec![
            hint("↑↓ select · Enter use · Space show/hide"),
            hint("n new · e edit · d delete · Esc back"),
        ];
        if let Some(e) = &self.editor {
            lines.push(hint("Tab switch field · Enter save · Esc cancel"));
            lines.push(Line::raw(format!(
                "{}Name: {}",
                if e.codes_field { "  " } else { "> " },
                e.name
            )));
            lines.push(Line::raw(format!(
                "{}Colors: {}",
                if e.codes_field { "> " } else { "  " },
                e.codes
            )));
            lines.push(hint("Codes: bB c gG k mM rR w yY oO sS (repeat to weight)"));
            lines.push(Line::from(preview(&e.name, &e.codes)));
        } else {
            let entries = catalog.entries();
            let count = inner.height.saturating_sub(4) as usize;
            let first = self.selected.saturating_sub(count.saturating_sub(1));
            for (i, p) in entries.iter().enumerate().skip(first).take(count) {
                let mut spans = vec![Span::raw(format!(
                    "{}[{}] ",
                    if i == self.selected { "> " } else { "  " },
                    if catalog.visible(&p.name) { "x" } else { " " }
                ))];
                spans.extend(preview(&p.name, &p.colors));
                lines.push(Line::from(spans).style(if i == self.selected {
                    Style::default().bg(Color::Rgb(35, 35, 45))
                } else {
                    Style::default()
                }));
            }
        }
        if self.deleting {
            lines.push(hint(
                "Delete custom palette? y confirms; any other key cancels",
            ));
        }
        if let Some(error) = &self.error {
            lines.push(Line::styled(
                error.clone(),
                Style::default().fg(crate::terminal::color('r')),
            ));
        }
        Paragraph::new(lines).render(inner, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }
    fn send(
        m: &mut Manager,
        c: &mut Catalog,
        path: &Path,
        active: &mut String,
        code: KeyCode,
    ) -> bool {
        m.key(key(code), c, path, active)
    }
    fn type_text(m: &mut Manager, c: &mut Catalog, path: &Path, active: &mut String, text: &str) {
        for ch in text.chars() {
            send(m, c, path, active, KeyCode::Char(ch));
        }
    }
    #[test]
    fn catalog_defaults_roundtrip_validation_and_weighted_colors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("palettes.toml");
        let mut c = Catalog::load(&path).unwrap();
        assert!(!path.exists());
        assert_eq!(c.entries().len(), 11);
        c.custom.push(Palette {
            name: "雨 Sunset".into(),
            colors: "rrroY".into(),
        });
        c.hidden.push("Green".into());
        c.save(&path).unwrap();
        let restored = Catalog::load(&path).unwrap();
        assert_eq!(restored.custom[0].colors, "rrroY");
        assert!(!restored.visible("Green"));
        for name in ["", " Green", "green", "bad\nname"] {
            let mut invalid = c.clone();
            invalid.custom.push(Palette {
                name: name.into(),
                colors: "g".into(),
            });
            assert!(invalid.validate().is_err());
        }
        for colors in ["", "z", "λ", "g\n"] {
            let mut invalid = c.clone();
            invalid.custom[0].colors = colors.into();
            assert!(invalid.validate().is_err());
        }
        let mut invalid = c.clone();
        invalid.hidden.push("Missing".into());
        assert!(invalid.validate().is_err());
        fs::write(&path, "unknown = true").unwrap();
        assert!(Catalog::load(&path).is_err());
    }
    #[test]
    fn cycling_skips_hidden_but_profiles_keep_their_codes() {
        let mut c = Catalog::default();
        c.hidden.push("Green".into());
        assert_eq!(c.cycle("gG", 1), "roygbm");
        assert_eq!(c.cycle("roygbm", -1), "oO");
        let saved: termatrix::settings::Settings = toml::from_str("colors = 'gG'").unwrap();
        assert_eq!(saved.colors, "gG");
        assert_eq!(c.label(&saved.colors).as_deref(), Some("Green"));
        c.custom.push(Palette {
            name: "Weighted".into(),
            colors: "rrg".into(),
        });
        assert_eq!(c.cycle("oO", 1), "rrg");
        assert_eq!(c.cycle("rrg", 1), "roygbm");
    }
    #[test]
    fn manager_create_edit_delete_cancel_and_profile_preservation() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("palettes.toml");
        let mut c = Catalog::default();
        let mut active = "gG".to_string();
        let mut m = Manager::open(&c, &active);
        send(&mut m, &mut c, &path, &mut active, KeyCode::Char('n'));
        type_text(&mut m, &mut c, &path, &mut active, "Night");
        send(&mut m, &mut c, &path, &mut active, KeyCode::Tab);
        for _ in 0..2 {
            send(&mut m, &mut c, &path, &mut active, KeyCode::Backspace);
        }
        type_text(&mut m, &mut c, &path, &mut active, "rrb");
        assert_eq!(m.preview_codes(&c), Some("rrb"));
        send(&mut m, &mut c, &path, &mut active, KeyCode::Enter);
        assert!(m.error.is_none());
        assert_eq!(c.custom[0].colors, "rrb");
        assert_eq!(active, "gG");
        assert!(send(&mut m, &mut c, &path, &mut active, KeyCode::Enter));
        assert_eq!(active, "rrb");
        send(&mut m, &mut c, &path, &mut active, KeyCode::Char(' '));
        assert!(!c.visible("Night"));
        send(&mut m, &mut c, &path, &mut active, KeyCode::Char('e'));
        type_text(&mut m, &mut c, &path, &mut active, " sky");
        send(&mut m, &mut c, &path, &mut active, KeyCode::Enter);
        assert_eq!(c.custom[0].name, "Night sky");
        assert!(!c.visible("Night sky"));
        send(&mut m, &mut c, &path, &mut active, KeyCode::Char('d'));
        send(&mut m, &mut c, &path, &mut active, KeyCode::Esc);
        assert_eq!(c.custom.len(), 1);
        send(&mut m, &mut c, &path, &mut active, KeyCode::Char('d'));
        send(&mut m, &mut c, &path, &mut active, KeyCode::Char('y'));
        assert!(c.custom.is_empty());
        assert_eq!(active, "rrb");
        assert!(c.hidden.is_empty());
        assert!(Catalog::load(&path).unwrap().custom.is_empty());
        send(&mut m, &mut c, &path, &mut active, KeyCode::Char('n'));
        type_text(&mut m, &mut c, &path, &mut active, "Cancel");
        send(&mut m, &mut c, &path, &mut active, KeyCode::Esc);
        assert!(m.editor.is_none());
        assert!(send(&mut m, &mut c, &path, &mut active, KeyCode::Esc));
    }
    #[test]
    fn last_visible_and_save_failures_preserve_catalog_and_cleanup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("catalog");
        fs::create_dir(&path).unwrap();
        let mut c = Catalog::default();
        let mut active = "gG".into();
        let mut m = Manager::open(&c, "gG");
        send(&mut m, &mut c, &path, &mut active, KeyCode::Char(' '));
        assert!(m.error.is_some());
        assert!(c.hidden.is_empty());
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
        let path = dir.path().join("palettes.toml");
        c.hidden = c.entries().iter().skip(1).map(|p| p.name.clone()).collect();
        c.save(&path).unwrap();
        let before = fs::read(&path).unwrap();
        send(&mut m, &mut c, &path, &mut active, KeyCode::Char(' '));
        assert!(m.error.as_ref().unwrap().contains("at least one"));
        assert!(c.visible("Green"));
        assert_eq!(fs::read(&path).unwrap(), before);
        send(&mut m, &mut c, &path, &mut active, KeyCode::Char('n'));
        type_text(&mut m, &mut c, &path, &mut active, "Green");
        send(&mut m, &mut c, &path, &mut active, KeyCode::Enter);
        assert!(m.error.is_some());
        assert!(c.custom.is_empty());
    }
    #[test]
    fn failed_creation_edit_and_deletion_preserve_saved_and_active_colors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("catalog-directory");
        fs::create_dir(&path).unwrap();
        let mut c = Catalog::default();
        let mut active = "gG".to_string();
        let mut m = Manager::open(&c, &active);
        send(&mut m, &mut c, &path, &mut active, KeyCode::Char('n'));
        type_text(&mut m, &mut c, &path, &mut active, "New");
        send(&mut m, &mut c, &path, &mut active, KeyCode::Enter);
        assert!(m.error.is_some());
        assert!(m.editor.is_some());
        assert!(c.custom.is_empty());
        send(&mut m, &mut c, &path, &mut active, KeyCode::Esc);
        c.custom.push(Palette {
            name: "Saved".into(),
            colors: "rrg".into(),
        });
        m = Manager::open(&c, "rrg");
        send(&mut m, &mut c, &path, &mut active, KeyCode::Char('e'));
        type_text(&mut m, &mut c, &path, &mut active, " edited");
        send(&mut m, &mut c, &path, &mut active, KeyCode::Enter);
        assert!(m.error.is_some());
        assert_eq!(c.custom[0].name, "Saved");
        send(&mut m, &mut c, &path, &mut active, KeyCode::Esc);
        send(&mut m, &mut c, &path, &mut active, KeyCode::Char('d'));
        send(&mut m, &mut c, &path, &mut active, KeyCode::Char('y'));
        assert!(m.error.is_some());
        assert_eq!(c.custom.len(), 1);
        assert_eq!(active, "gG");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
        let path = dir.path().join("palettes.toml");
        c.hidden = BUILTINS.iter().map(|(name, _)| (*name).into()).collect();
        c.save(&path).unwrap();
        send(&mut m, &mut c, &path, &mut active, KeyCode::Char('d'));
        send(&mut m, &mut c, &path, &mut active, KeyCode::Char('y'));
        assert!(m.error.as_ref().unwrap().contains("at least one"));
        assert_eq!(Catalog::load(&path).unwrap().custom.len(), 1);
    }
    #[test]
    fn manager_navigation_rendering_editing_and_builtin_protection() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("palettes.toml");
        let mut c = Catalog::default();
        let mut active = "gG".into();
        let mut m = Manager::open(&c, "gG");
        for code in [KeyCode::Char('d'), KeyCode::Char('e')] {
            send(&mut m, &mut c, &path, &mut active, code);
        }
        assert!(!m.deleting);
        assert!(m.editor.is_none());
        send(&mut m, &mut c, &path, &mut active, KeyCode::Up);
        assert_eq!(m.selected, 10);
        assert_eq!(m.preview_codes(&c), Some("oO"));
        send(&mut m, &mut c, &path, &mut active, KeyCode::Down);
        assert_eq!(m.selected, 0);
        for (w, h) in [(0, 0), (4, 3), (76, 22)] {
            let area = Rect::new(0, 0, w, h);
            m.render(area, &mut Buffer::empty(area), &c);
        }
        let area = Rect::new(0, 0, 76, 22);
        let mut buf = Buffer::empty(area);
        m.render(area, &mut buf, &c);
        let text = buf
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(text.contains("> [x] Green"));
        assert!(buf
            .content
            .iter()
            .any(|cell| cell.symbol() == "G" && cell.fg == crate::terminal::color('g')));
        send(&mut m, &mut c, &path, &mut active, KeyCode::Char('n'));
        m.key(
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            &mut c,
            &path,
            &mut active,
        );
        assert_eq!(m.editor.as_ref().unwrap().name, "");
        type_text(&mut m, &mut c, &path, &mut active, "Preview");
        send(&mut m, &mut c, &path, &mut active, KeyCode::Tab);
        type_text(&mut m, &mut c, &path, &mut active, "z");
        assert_eq!(m.preview_codes(&c), None);
        m.error = Some("Invalid color".into());
        m.render(area, &mut buf, &c);
        let text = buf
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(text.contains("Name: Preview"));
        assert!(text.contains("Invalid color"));
    }
}
