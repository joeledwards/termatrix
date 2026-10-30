use serde::{Deserialize, Serialize};

/// Internal safety bounds for abstract trace speeds; not user settings.
pub const MIN_TRACE_SPEED: f64 = 0.01;
pub const MAX_TRACE_SPEED: f64 = 1000.;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub min_speed: f64,
    pub max_speed: f64,
    /// -1 favors slow, 0 is uniform, +1 favors fast.
    pub speed_bias: f64,
    pub density: f64,
    pub clumping: f64,
    pub min_length: usize,
    pub max_length: usize,
    pub mutation_rate: f64,
    pub colors: String,
    pub per_glyph_colors: bool,
    pub highlight_probability: f64,
    pub glyph_set: String,
    pub canvas_relative: bool,
    pub max_refresh_rate: u16,
    pub panel: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            min_speed: 2.,
            max_speed: 12.,
            speed_bias: 0.,
            density: 1.5,
            clumping: 0.,
            min_length: 4,
            max_length: 24,
            mutation_rate: 0.05,
            colors: "gG".into(),
            per_glyph_colors: true,
            highlight_probability: 0.12,
            glyph_set: "special".into(),
            canvas_relative: false,
            max_refresh_rate: 60,
            panel: true,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        let finite = [
            self.min_speed,
            self.max_speed,
            self.speed_bias,
            self.density,
            self.clumping,
            self.mutation_rate,
            self.highlight_probability,
        ]
        .iter()
        .all(|v| v.is_finite());
        if !finite
            || self.min_speed < MIN_TRACE_SPEED
            || self.min_speed > self.max_speed
            || self.max_speed > MAX_TRACE_SPEED
        {
            return Err(format!(
                "speed bounds must be finite: {MIN_TRACE_SPEED} <= min <= max <= {MAX_TRACE_SPEED}"
            ));
        }
        if !(-1. ..=1.).contains(&self.speed_bias)
            || !(0. ..=10.).contains(&self.density)
            || !(0. ..=1.).contains(&self.clumping)
            || !(0. ..=10.).contains(&self.mutation_rate)
            || !(0. ..=1.).contains(&self.highlight_probability)
        {
            return Err(
                "bias must be -1..1, density 0..10, clumping 0..1, mutation_rate 0..10, highlight_probability 0..1".into(),
            );
        }
        if self.min_length == 0
            || self.min_length > self.max_length
            || self.max_length > 512
            || !(1..=240).contains(&self.max_refresh_rate)
        {
            return Err("lengths must satisfy 1 <= min <= max <= 512; refresh rate 1..240".into());
        }
        if self.colors.is_empty() || !self.colors.chars().all(|c| "bcgGkmMrRwyYoOsS".contains(c)) {
            return Err("colors must be a nonempty sequence of bcgGkmMrRwyYoOsS".into());
        }
        self.glyph_set.parse::<crate::alphabet::GlyphSet>()?;
        Ok(())
    }
    pub fn palette_color(&self, percentile: f64) -> char {
        let index = ((percentile * self.colors.len() as f64) as usize).min(self.colors.len() - 1);
        self.colors.as_bytes()[index] as char
    }
    pub fn speed(&self, percentile: f64) -> f64 {
        let p = if self.speed_bias < 0. {
            percentile.powf(1. + -self.speed_bias * 4.)
        } else {
            1. - (1. - percentile).powf(1. + self.speed_bias * 4.)
        };
        self.min_speed + (self.max_speed - self.min_speed) * p
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn speed_hard_limits_are_internal_and_validated() {
        let mut s = Settings {
            min_speed: MIN_TRACE_SPEED,
            max_speed: MAX_TRACE_SPEED,
            ..Settings::default()
        };
        s.validate().unwrap();
        let saved = toml::to_string(&s).unwrap();
        assert!(!saved.contains("allowed_"));
        s.min_speed = MIN_TRACE_SPEED / 2.;
        assert!(s.validate().is_err());
        s.min_speed = MIN_TRACE_SPEED;
        s.max_speed = MAX_TRACE_SPEED + 1.;
        assert!(s.validate().is_err());
    }
    #[test]
    fn fresh_defaults_use_special_and_saved_choice_wins() {
        assert_eq!(Settings::default().glyph_set, "special");
        assert_eq!(toml::from_str::<Settings>("").unwrap().glyph_set, "special");
        assert_eq!(
            toml::from_str::<Settings>("glyph_set = 'binary'")
                .unwrap()
                .glyph_set,
            "binary"
        );
    }
    #[test]
    fn config_round_trip_and_validation() {
        let s = Settings::default();
        s.validate().unwrap();
        let restored: Settings = toml::from_str(&toml::to_string(&s).unwrap()).unwrap();
        assert_eq!(restored.max_speed, s.max_speed);
        for value in [f64::NAN, -1., 2000.] {
            let mut bad = s.clone();
            bad.min_speed = value;
            assert!(bad.validate().is_err());
        }
        let mut equal = s.clone();
        equal.min_speed = equal.max_speed;
        equal.validate().unwrap();
        assert_eq!(equal.speed(0.), equal.speed(1.));
    }
    #[test]
    fn bias_preserves_bounds_and_order() {
        let mut s = Settings::default();
        let uniform = s.speed(0.5);
        s.speed_bias = -1.;
        assert!(s.speed(0.5) < uniform);
        s.speed_bias = 1.;
        assert!(s.speed(0.5) > uniform);
        for bias in [-1., 0., 1.] {
            s.speed_bias = bias;
            assert_eq!(s.speed(0.), s.min_speed);
            assert_eq!(s.speed(1.), s.max_speed);
            assert!(s.speed(0.2) < s.speed(0.8));
        }
    }
}
