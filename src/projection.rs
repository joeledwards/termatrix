//! Orthographic projection in logical glyph lanes. No terminal or graphics types.
use crate::model::Scene;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Highlight {
    Head,
    Body,
    Accent,
    Tail,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub column: usize,
    pub row: usize,
    pub symbol: char,
    pub color: char,
    pub highlight: Highlight,
    pub trace_id: u64,
    /// Normalized distance behind the head, independent of viewport clipping.
    pub progress: f64,
}
/// Returns visible samples after depth occlusion. Hidden traces are never modified.
/// A sample occupies a whole logical lane; frontends allocate its glyph footprint.
pub fn orthographic(scene: &Scene) -> Vec<Sample> {
    let mut visible = std::collections::BTreeMap::new();
    let mut traces: Vec<_> = scene.traces.iter().collect();
    traces.sort_by(|a, b| b.depth.total_cmp(&a.depth).then(a.id.cmp(&b.id)));
    for t in traces {
        if t.column >= scene.columns {
            continue;
        }
        for (i, g) in t.glyphs.iter().enumerate() {
            let row = t.head_row() - i as i64;
            if row < 0 || row >= scene.height as i64 {
                continue;
            }
            visible.insert(
                (row as usize, t.column),
                Sample {
                    column: t.column,
                    row: row as usize,
                    symbol: g.symbol,
                    color: g.color,
                    highlight: if i == 0 {
                        Highlight::Head
                    } else if g.highlighted {
                        Highlight::Accent
                    } else if i * 2 >= t.glyphs.len() {
                        Highlight::Tail
                    } else {
                        Highlight::Body
                    },
                    trace_id: t.id,
                    progress: i as f64 / t.glyphs.len().saturating_sub(1).max(1) as f64,
                },
            );
        }
    }
    visible.into_values().collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;
    #[test]
    fn occlusion_is_non_destructive_and_reveals_background() {
        let mut scene = Scene::new(42);
        scene.resize(1, 30);
        let s = Settings {
            density: 10.,
            ..Settings::default()
        };
        for _ in 0..50 {
            scene.advance(0.02, &s);
        }
        let mut back = scene.traces[0].clone();
        back.depth = 0.9;
        back.id = 1;
        let mut front = back.clone();
        front.depth = 0.1;
        front.id = 2;
        front.glyphs[0].symbol = 'X';
        back.glyphs[0].symbol = 'B';
        scene.traces = vec![back.clone(), front];
        let before = scene.traces.clone();
        assert_eq!(
            orthographic(&scene)
                .iter()
                .find(|s| s.row == back.head_row() as usize)
                .unwrap()
                .symbol,
            'X'
        );
        assert_eq!(scene.traces, before);
        scene.traces.pop();
        assert_eq!(
            orthographic(&scene)
                .iter()
                .find(|s| s.row == back.head_row() as usize)
                .unwrap()
                .symbol,
            'B'
        );
    }
}
