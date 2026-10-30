//! Bounded wall-time history, sampled independently of display refresh.
use std::collections::VecDeque;
use termatrix::{model::Scene, projection::orthographic};
pub const WINDOW: f64 = 600.;
const INTERVAL: f64 = 0.2;
const CAPACITY: usize = 3001;
/// Normalized horizontal position: 0 is ten minutes ago, 1 is now.
/// Compression is validated in settings (0..10); zero is exactly linear.
pub fn age_position(age: f64, compression: f64) -> f64 {
    let fraction = (age / WINDOW).clamp(0., 1.);
    if fraction == 0. {
        return 1.;
    }
    if fraction == 1. {
        return 0.;
    }
    if compression < 1e-8 {
        1. - fraction
    } else {
        1. - (compression.exp_m1() * fraction).ln_1p() / compression
    }
}
/// Actual age at an evenly spaced chart coordinate, used for truthful axis labels.
pub fn position_age(position: f64, compression: f64) -> f64 {
    let fraction = 1. - position.clamp(0., 1.);
    if compression < 1e-8 {
        WINDOW * fraction
    } else {
        WINDOW * (compression * fraction).exp_m1() / compression.exp_m1()
    }
}
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
        self.record(now, density);
    }
    fn record(&mut self, now: f64, value: f64) {
        self.samples.push_back((now, value));
        while self.samples.len() > CAPACITY
            || self.samples.front().is_some_and(|(t, _)| *t < now - WINDOW)
        {
            self.samples.pop_front();
        }
    }
    pub fn points(&self, now: f64, compression: f64) -> Vec<(f64, f64)> {
        self.samples
            .iter()
            .filter(|(t, _)| *t >= now - WINDOW)
            .map(|(t, v)| (age_position(now - t, compression), *v))
            .collect()
    }
    pub fn current(&self) -> f64 {
        self.samples.back().map_or(0., |(_, v)| *v)
    }
}
/// Render cadence and active work, aggregated between telemetry samples.
#[derive(Default)]
pub struct PerformanceHistory {
    pub traces: DensityHistory,
    pub fps: DensityHistory,
    pub work: DensityHistory,
    last_sample: f64,
    frames: usize,
    work_seconds: f64,
}
impl PerformanceHistory {
    pub fn frame(&mut self, work_seconds: f64) {
        self.frames += 1;
        self.work_seconds += work_seconds;
    }
    pub fn sample(&mut self, now: f64, traces: usize) {
        let elapsed = now - self.last_sample;
        if elapsed < INTERVAL {
            return;
        }
        self.traces.record(now, traces as f64);
        self.fps.record(now, self.frames as f64 / elapsed);
        self.work.record(
            now,
            if self.frames == 0 {
                0.
            } else {
                self.work_seconds * 1000. / self.frames as f64
            },
        );
        self.last_sample = now;
        self.frames = 0;
        self.work_seconds = 0.;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use termatrix::settings::Settings;
    #[test]
    fn performance_aggregates_frames_over_wall_time_and_keeps_shared_history() {
        let mut metrics = PerformanceHistory::default();
        metrics.frame(0.002);
        metrics.frame(0.004);
        metrics.sample(0.1, 10);
        assert!(metrics.fps.points(0.1, 6.).is_empty());
        metrics.sample(0.25, 12);
        assert_eq!(metrics.fps.current(), 8.);
        assert_eq!(metrics.work.current(), 3.);
        assert_eq!(metrics.traces.current(), 12.);
        metrics.sample(0.5, 7);
        assert_eq!(metrics.fps.current(), 0.);
        assert_eq!(metrics.work.current(), 0.);
        for i in 3..4000 {
            metrics.sample(i as f64 * 0.25, i);
        }
        let traces = metrics.traces.points(1000., 6.);
        let fps = metrics.fps.points(1000., 6.);
        let work = metrics.work.points(1000., 6.);
        assert!(traces.len() <= CAPACITY);
        assert!(traces.len() < 2500);
        assert_eq!(traces.len(), fps.len());
        assert_eq!(fps.len(), work.len());
        for ((t, f), w) in traces.iter().zip(&fps).zip(&work) {
            assert_eq!(t.0, f.0);
            assert_eq!(f.0, w.0);
        }
    }
    #[test]
    fn compression_preserves_endpoints_order_and_expands_recent_history() {
        for compression in [0., 1e-10, 1., 6., 10.] {
            assert_eq!(age_position(0., compression), 1.);
            assert_eq!(age_position(WINDOW, compression), 0.);
            let mut previous = 1.;
            for age in 0..=600 {
                let x = age_position(age as f64, compression);
                assert!(x <= previous);
                previous = x;
                assert!((position_age(x, compression) - age as f64).abs() < 1e-6);
            }
        }
        assert_eq!(age_position(300., 0.), 0.5);
        assert!(1. - age_position(10., 6.) > 0.3);
        assert!(1. - age_position(10., 10.) > 1. - age_position(10., 6.));
    }
    #[test]
    fn live_compression_keeps_history_and_density_values() {
        let scene = Scene::new(42);
        let mut history = DensityHistory::default();
        for t in 0..=600 {
            history.sample(t as f64, &scene);
        }
        let linear = history.points(600., 0.);
        let compressed = history.points(600., 6.);
        assert_eq!(linear.len(), 601);
        assert_eq!(linear.len(), compressed.len());
        assert_eq!(
            linear.iter().map(|p| p.1).collect::<Vec<_>>(),
            compressed.iter().map(|p| p.1).collect::<Vec<_>>()
        );
        assert!(compressed[300].0 < linear[300].0);
        assert_eq!(history.points(600., 0.), linear);
    }
    #[test]
    fn sampling_is_bounded_independent_and_handles_empty_canvas() {
        let scene = Scene::new(42);
        let mut history = DensityHistory::default();
        history.sample(0., &scene);
        history.sample(0.1, &scene);
        assert_eq!(history.points(0.1, 6.).len(), 1);
        assert_eq!(history.current(), 0.);
        for i in 1..10000 {
            history.sample(i as f64 * 0.25, &scene);
        }
        let points = history.points(2499.75, 6.);
        assert!(points.len() <= CAPACITY);
        assert!(points
            .iter()
            .all(|(x, y)| (0. ..=1.).contains(x) && *y == 0.));
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
