use std::ops::RangeInclusive;

use chrono::{DateTime, TimeDelta, Utc};

pub const MIN_VISIBLE_POINTS: usize = 14;

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct GemChartZoom {
    pub scale: f64,
    pub offset: f64,
}

impl Default for GemChartZoom {
    fn default() -> Self {
        Self { scale: 1.0, offset: 0.0 }
    }
}

impl GemChartZoom {
    pub fn magnified(&self, magnification: f64, anchor: f64, points: usize) -> Self {
        let anchor = match self.offset > 0.0 {
            true => anchor.clamp(0.0, 1.0),
            false => 1.0,
        };
        let scale = Self { scale: self.scale * magnification, ..*self }.clamped(points).scale;
        let focus = self.offset + (1.0 - anchor) / self.scale;
        Self {
            scale,
            offset: focus - (1.0 - anchor) / scale,
        }
        .clamped(points)
    }

    pub fn panned(&self, fraction: f64, points: usize) -> Self {
        Self {
            offset: self.offset + fraction / self.scale,
            ..*self
        }
        .clamped(points)
    }

    pub fn clamped(&self, points: usize) -> Self {
        let maximum_scale = (points as f64 / MIN_VISIBLE_POINTS as f64).max(1.0);
        let scale = self.scale.clamp(1.0, maximum_scale);
        Self {
            scale,
            offset: self.offset.clamp(0.0, 1.0 - 1.0 / scale),
        }
    }

    pub fn is_zoomed(&self) -> bool {
        self.scale > 1.0
    }

    pub fn window(&self, first: DateTime<Utc>, last: DateTime<Utc>) -> RangeInclusive<DateTime<Utc>> {
        let span = (last - first).num_milliseconds() as f64;
        let end = last - TimeDelta::milliseconds((span * self.offset) as i64);
        (end - TimeDelta::milliseconds((span / self.scale) as i64))..=end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magnified() {
        let unzoomed = GemChartZoom::default();
        let panned = GemChartZoom { scale: 2.0, offset: 0.25 };

        assert_eq!(unzoomed.magnified(2.0, 1.0, 70), GemChartZoom { scale: 2.0, offset: 0.0 });
        assert_eq!(unzoomed.magnified(100.0, 1.0, 70), GemChartZoom { scale: 5.0, offset: 0.0 }, "zooming in stops with fourteen points on screen");
        assert_eq!(unzoomed.magnified(0.5, 1.0, 70), unzoomed, "zooming out stops at the whole period");
        assert_eq!(unzoomed.magnified(2.0, 0.5, 70), GemChartZoom { scale: 2.0, offset: 0.0 }, "the newest point stays pinned while it is on screen");
        assert_eq!(panned.magnified(2.0, 0.5, 70), GemChartZoom { scale: 4.0, offset: 0.375 }, "once panned back, the point between the fingers stays put");
        assert_eq!(panned.magnified(2.0, 1.4, 70), panned.magnified(2.0, 1.0, 70), "fingers past the plot zoom around its edge");
    }

    #[test]
    fn test_panned() {
        let zoomed = GemChartZoom { scale: 2.0, offset: 0.0 };

        assert_eq!(zoomed.panned(0.5, 70), GemChartZoom { scale: 2.0, offset: 0.25 });
        assert_eq!(zoomed.panned(3.0, 70), GemChartZoom { scale: 2.0, offset: 0.5 }, "panning stops at the oldest point");
        assert_eq!(zoomed.panned(-1.0, 70), zoomed, "panning stops at the newest point");
    }

    #[test]
    fn test_clamped() {
        assert_eq!(GemChartZoom { scale: 4.0, offset: 0.7 }.clamped(28), GemChartZoom { scale: 2.0, offset: 0.5 });
        assert_eq!(GemChartZoom { scale: 0.5, offset: -0.2 }.clamped(80), GemChartZoom::default());
    }

    #[test]
    fn test_window() {
        let at = |seconds: i64| DateTime::from_timestamp(seconds, 0).unwrap();

        assert_eq!(GemChartZoom { scale: 4.0, offset: 0.0 }.window(at(0), at(1000)), at(750)..=at(1000));
        assert_eq!(GemChartZoom { scale: 4.0, offset: 0.25 }.window(at(0), at(1000)), at(500)..=at(750));
    }
}
