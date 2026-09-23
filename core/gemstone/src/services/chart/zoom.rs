use chrono::{DateTime, TimeDelta, Utc};

const MIN_VISIBLE_POINTS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct GemChartZoom {
    pub scale: f64,
}

impl GemChartZoom {
    pub fn identity() -> Self {
        Self { scale: 1.0 }
    }

    pub fn magnified(&self, magnification: f64, points: usize) -> Self {
        Self { scale: self.scale * magnification }.clamped(points)
    }

    pub fn clamped(&self, points: usize) -> Self {
        let maximum_scale = (points as f64 / MIN_VISIBLE_POINTS as f64).max(1.0);
        Self { scale: self.scale.clamp(1.0, maximum_scale) }
    }

    pub fn visible_start(&self, first: DateTime<Utc>, last: DateTime<Utc>) -> DateTime<Utc> {
        let visible_milliseconds = (last - first).num_milliseconds() as f64 / self.scale;
        last - TimeDelta::milliseconds(visible_milliseconds as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magnified() {
        let zoom = GemChartZoom::identity();

        assert_eq!(zoom.magnified(2.0, 60), GemChartZoom { scale: 2.0 });
        assert_eq!(zoom.magnified(2.0, 60).magnified(1.5, 60), GemChartZoom { scale: 3.0 });
        assert_eq!(zoom.magnified(0.5, 60), zoom, "zooming out past the whole period stays on the whole period");
        assert_eq!(zoom.magnified(100.0, 60), GemChartZoom { scale: 7.5 }, "zooming in stops at the minimum visible points");
        assert_eq!(zoom.magnified(100.0, 60).magnified(0.5, 60), GemChartZoom { scale: 3.75 }, "zooming back out responds at once from the limit");
        assert_eq!(zoom.magnified(3.0, 5), zoom, "a chart with fewer points than the minimum cannot zoom");
    }

    #[test]
    fn test_clamped() {
        assert_eq!(GemChartZoom { scale: 4.0 }.clamped(80), GemChartZoom { scale: 4.0 });
        assert_eq!(GemChartZoom { scale: 4.0 }.clamped(16), GemChartZoom { scale: 2.0 }, "a refresh with fewer points narrows the zoom it can hold");
        assert_eq!(GemChartZoom { scale: 0.5 }.clamped(80), GemChartZoom::identity());
    }

    #[test]
    fn test_visible_start() {
        let first = DateTime::from_timestamp(0, 0).unwrap();
        let last = DateTime::from_timestamp(1000, 0).unwrap();

        assert_eq!(GemChartZoom::identity().visible_start(first, last), first);
        assert_eq!(GemChartZoom { scale: 4.0 }.visible_start(first, last), DateTime::from_timestamp(750, 0).unwrap());
        assert_eq!(GemChartZoom { scale: 2.0 }.visible_start(last, last), last, "a single moment has nothing to zoom");
    }
}
