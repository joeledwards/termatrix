use crate::{alphabet::GlyphSet, settings::Settings};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::collections::VecDeque;

pub const MAX_TRACES: usize = 2048;
pub const MAX_GLYPHS: usize = 100_000;
#[derive(Clone, Debug, PartialEq)]
pub struct Glyph {
    pub symbol: char,
    pub highlighted: bool,
    highlight_percentile: f64,
    pub color: char,
    color_percentile: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Trace {
    pub id: u64,
    pub column: usize,
    /// Smaller depth is nearer. Equal depths are ordered by ID.
    pub depth: f64,
    pub head: f64,
    pub glyphs: VecDeque<Glyph>,
    pub max_length: usize,
    pub speed_percentile: f64,
    pub color: char,
    color_percentile: f64,
    emitted_row: i64,
}
impl Trace {
    pub fn head_row(&self) -> i64 {
        self.emitted_row
    }
}
pub struct Scene {
    pub traces: Vec<Trace>,
    pub columns: usize,
    pub height: usize,
    rng: StdRng,
    next_id: u64,
    accumulator: f64,
    spawn_credit: f64,
}
impl Scene {
    pub fn new(seed: u64) -> Self {
        Self {
            traces: vec![],
            columns: 0,
            height: 0,
            rng: StdRng::seed_from_u64(seed),
            next_id: 0,
            accumulator: 0.,
            spawn_credit: 0.,
        }
    }
    pub fn resize(&mut self, columns: usize, height: usize) {
        self.columns = columns.min(4096);
        self.height = height.min(4096);
        self.traces.retain(|t| {
            t.column < self.columns && t.emitted_row - (t.glyphs.len() as i64) < self.height as i64
        });
    }
    /// Fixed simulation quanta make seeded runs independent of frontend refresh cadence.
    /// Unexpected stalls consume at most 250ms per call; explicit pause passes no elapsed time.
    pub fn advance(&mut self, elapsed: f64, settings: &Settings) {
        if !elapsed.is_finite() || elapsed <= 0. || self.columns == 0 || self.height == 0 {
            return;
        }
        self.accumulator += elapsed.min(0.25);
        let alphabet = settings
            .glyph_set
            .parse::<GlyphSet>()
            .expect("validated settings")
            .to_alphabet();
        while self.accumulator + 1e-12 >= 0.01 {
            self.accumulator -= 0.01;
            self.step(settings, &alphabet);
        }
        self.refresh_colors(settings);
    }
    /// Re-resolve stable random color choices, including while paused.
    pub fn refresh_colors(&mut self, settings: &Settings) {
        for trace in &mut self.traces {
            trace.color = settings.palette_color(trace.color_percentile);
            for glyph in &mut trace.glyphs {
                glyph.highlighted = glyph.highlight_percentile < settings.highlight_probability;
                glyph.color = if settings.per_glyph_colors {
                    settings.palette_color(glyph.color_percentile)
                } else {
                    trace.color
                };
            }
        }
    }
    fn step(&mut self, s: &Settings, alphabet: &[char]) {
        let scale = if s.canvas_relative {
            self.height as f64 / 30.
        } else {
            1.
        };
        for t in &mut self.traces {
            t.head += s.speed(t.speed_percentile) * scale * 0.01;
            let row = t.head.floor() as i64;
            for _ in t.emitted_row..row {
                t.glyphs.push_front(Glyph {
                    symbol: alphabet[self.rng.gen_range(0..alphabet.len())],
                    highlighted: false,
                    highlight_percentile: self.rng.gen(),
                    color: 'g',
                    color_percentile: self.rng.gen(),
                });
                t.glyphs.truncate(t.max_length);
            }
            t.emitted_row = row;
            for glyph in &mut t.glyphs {
                if self.rng.gen_bool(1. - (-s.mutation_rate * 0.01).exp()) {
                    glyph.symbol = alphabet[self.rng.gen_range(0..alphabet.len())];
                }
            }
        }
        self.traces
            .retain(|t| t.emitted_row - (t.glyphs.len() as i64) + 1 < self.height as i64);
        let target = ((self.columns as f64 * s.density).round() as usize).min(MAX_TRACES);
        if self.traces.len() >= target {
            self.spawn_credit = 0.;
            return;
        }
        self.spawn_credit = (self.spawn_credit + (target as f64 * 0.5).max(1.) * 0.01).min(8.);
        while self.spawn_credit >= 1. && self.traces.len() < target {
            self.spawn_credit -= 1.;
            let glyph_count: usize = self.traces.iter().map(|t| t.max_length).sum();
            if glyph_count + s.max_length > MAX_GLYPHS {
                break;
            }
            let column = if !self.traces.is_empty() && self.rng.gen_bool(s.clumping) {
                let anchor = self.traces[self.rng.gen_range(0..self.traces.len())].column as i64;
                (anchor + self.rng.gen_range(-2..=2)).clamp(0, self.columns as i64 - 1) as usize
            } else {
                self.rng.gen_range(0..self.columns)
            };
            self.traces.push(Trace {
                id: self.next_id,
                column,
                depth: self.rng.gen(),
                head: 0.,
                glyphs: VecDeque::from([Glyph {
                    symbol: alphabet[self.rng.gen_range(0..alphabet.len())],
                    highlighted: false,
                    highlight_percentile: self.rng.gen(),
                    color: 'g',
                    color_percentile: self.rng.gen(),
                }]),
                max_length: self.rng.gen_range(s.min_length..=s.max_length),
                speed_percentile: self.rng.gen(),
                color: 'g',
                color_percentile: self.rng.gen(),
                emitted_row: 0,
            });
            self.next_id += 1;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cadence_independent() {
        let s = Settings::default();
        let mut a = Scene::new(42);
        let mut b = Scene::new(42);
        a.resize(40, 30);
        b.resize(40, 30);
        for _ in 0..100 {
            a.advance(0.02, &s);
        }
        for _ in 0..200 {
            b.advance(0.01, &s);
        }
        assert!(!a.traces.is_empty());
        assert_eq!(a.traces, b.traces);
    }
    #[test]
    fn empty_resize_density_and_lifecycle() {
        let mut s = Settings::default();
        let mut a = Scene::new(1);
        a.advance(1., &s);
        assert!(a.traces.is_empty());
        a.resize(4, 3);
        for _ in 0..100 {
            a.advance(0.02, &s);
        }
        assert!(!a.traces.is_empty());
        s.density = 0.;
        for _ in 0..1000 {
            a.advance(0.02, &s);
        }
        assert!(a.traces.is_empty());
        a.resize(0, 0);
        a.advance(1., &s);
    }
    #[test]
    fn population_respects_reserved_glyph_budget() {
        let mut scene = Scene::new(7);
        scene.resize(4096, 4096);
        let settings = Settings {
            density: 10.,
            min_length: 512,
            max_length: 512,
            min_speed: 0.1,
            max_speed: 0.1,
            ..Settings::default()
        };
        for _ in 0..40 {
            scene.advance(0.25, &settings);
        }
        assert!(!scene.traces.is_empty());
        assert!(scene.traces.len() <= MAX_TRACES);
        assert!(scene.traces.iter().map(|t| t.max_length).sum::<usize>() <= MAX_GLYPHS);
    }
    #[test]
    fn live_speed_changes_preserve_identity_and_percentile() {
        let mut scene = Scene::new(7);
        scene.resize(4, 100);
        let mut settings = Settings {
            mutation_rate: 0.,
            ..Settings::default()
        };
        for _ in 0..100 {
            scene.advance(0.02, &settings);
        }
        let before = scene.traces[0].clone();
        settings.density = 0.;
        settings.min_speed = 20.;
        settings.max_speed = 20.;
        scene.advance(0.1, &settings);
        let after = &scene.traces[0];
        assert_eq!(before.id, after.id);
        assert_eq!(before.speed_percentile, after.speed_percentile);
        assert!((after.head - before.head - 2.).abs() < 1e-9);
    }
    #[test]
    fn live_palettes_and_individual_colors_preserve_scene() {
        let mut scene = Scene::new(42);
        scene.resize(20, 100);
        let mut settings = Settings::default();
        for _ in 0..200 {
            scene.advance(0.02, &settings);
        }
        let positions: Vec<_> = scene.traces.iter().map(|t| (t.id, t.head)).collect();
        settings.colors = "o".into();
        settings.validate().unwrap();
        scene.refresh_colors(&settings);
        assert!(scene
            .traces
            .iter()
            .flat_map(|t| &t.glyphs)
            .all(|g| g.color == 'o'));
        settings.colors = "roygcbm".into();
        settings.per_glyph_colors = true;
        scene.refresh_colors(&settings);
        let colors: std::collections::BTreeSet<_> = scene
            .traces
            .iter()
            .flat_map(|t| &t.glyphs)
            .map(|g| g.color)
            .collect();
        assert!(colors.len() > 1);
        assert!(scene
            .traces
            .iter()
            .any(|t| t.glyphs.iter().any(|g| g.color != t.glyphs[0].color)));
        let snapshot = scene.traces.clone();
        scene.refresh_colors(&settings);
        assert_eq!(scene.traces, snapshot);
        settings.per_glyph_colors = false;
        scene.refresh_colors(&settings);
        assert!(scene
            .traces
            .iter()
            .all(|t| t.glyphs.iter().all(|g| g.color == t.color)));
        assert_eq!(
            positions,
            scene
                .traces
                .iter()
                .map(|t| (t.id, t.head))
                .collect::<Vec<_>>()
        );
    }
    #[test]
    fn highlight_probability_is_live_stable_and_projected() {
        let mut scene = Scene::new(42);
        scene.resize(20, 100);
        let mut settings = Settings::default();
        assert_eq!(settings.colors, "gG");
        assert!(settings.per_glyph_colors);
        for _ in 0..200 {
            scene.advance(0.02, &settings);
        }
        let original = scene.traces.clone();
        settings.highlight_probability = 0.;
        scene.refresh_colors(&settings);
        assert!(scene
            .traces
            .iter()
            .flat_map(|t| &t.glyphs)
            .all(|g| !g.highlighted));
        settings.highlight_probability = 1.;
        scene.refresh_colors(&settings);
        assert!(scene
            .traces
            .iter()
            .flat_map(|t| &t.glyphs)
            .all(|g| g.highlighted));
        let samples = crate::projection::orthographic(&scene);
        assert!(samples
            .iter()
            .any(|s| s.highlight == crate::projection::Highlight::Accent));
        assert!(samples
            .iter()
            .any(|s| s.highlight == crate::projection::Highlight::Head));
        settings.highlight_probability = Settings::default().highlight_probability;
        scene.refresh_colors(&settings);
        assert_eq!(scene.traces, original);
        let shades: std::collections::BTreeSet<_> = scene
            .traces
            .iter()
            .flat_map(|t| &t.glyphs)
            .map(|g| g.color)
            .collect();
        assert_eq!(shades, std::collections::BTreeSet::from(['g', 'G']));
    }
}
