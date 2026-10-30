//! Bounded wall-time history, sampled independently of display refresh.
use std::collections::VecDeque;
use termatrix::{model::Scene, projection::orthographic};
const WINDOW: f64 = 60.;
const INTERVAL: f64 = 0.2;
const CAPACITY: usize = 301;
#[derive(Default)]
pub struct DensityHistory {
    samples: VecDeque<(f64, f64)>,
    next_sample: f64,
}
impl DensityHistory {
    pub fn sample(&mut self, now: f64, scene: &Scene) {
        if now < self.next_sample {
            return;
        }
        self.next_sample = now + INTERVAL;
        let positions = scene.columns * scene.height;
        let density = if positions == 0 {
            0.
        } else {
            orthographic(scene).len() as f64 / positions as f64 * 100.
        };
        self.samples.push_back((now, density));
        while self.samples.len() > CAPACITY
            || self.samples.front().is_some_and(|(t, _)| *t < now - WINDOW)
        {
            self.samples.pop_front();
        }
    }
    pub fn points(&self, now: f64) -> Vec<(f64, f64)> {
        self.samples
            .iter()
            .filter(|(t, _)| *t >= now - WINDOW)
            .map(|(t, v)| (t - now, *v))
            .collect()
    }
    pub fn current(&self) -> f64 {
        self.samples.back().map_or(0., |(_, v)| *v)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use termatrix::settings::Settings;
    #[test]
    fn sampling_is_bounded_independent_and_handles_empty_canvas() {
        let scene = Scene::new(42);
        let mut history = DensityHistory::default();
        history.sample(0., &scene);
        history.sample(0.1, &scene);
        assert_eq!(history.points(0.1).len(), 1);
        assert_eq!(history.current(), 0.);
        for i in 1..1000 {
            history.sample(i as f64 * 0.25, &scene);
        }
        let points = history.points(249.75);
        assert!(points.len() <= 301);
        assert!(points
            .iter()
            .all(|(x, y)| (-60. ..=0.).contains(x) && *y == 0.));
    }
    #[test]
    fn overlapping_traces_count_only_once() {
        let mut scene = Scene::new(7);
        scene.resize(1, 10);
        let settings = Settings {
            density: 1.,
            min_speed: 0.1,
            max_speed: 0.1,
            ..Settings::default()
        };
        for _ in 0..8 {
            scene.advance(0.25, &settings);
        }
        assert_eq!(scene.traces.len(), 1);
        let mut history = DensityHistory::default();
        history.sample(0., &scene);
        assert_eq!(history.current(), 10.);
        let mut duplicate = scene.traces[0].clone();
        duplicate.id += 1;
        scene.traces.push(duplicate);
        history.sample(1., &scene);
        assert_eq!(history.current(), 10.);
        scene.resize(0, 0);
        history.sample(2., &scene);
        assert_eq!(history.current(), 0.);
    }
}
