use std::iter;

use primitives::ChartDateValue;

pub fn reduced_points(values: &[ChartDateValue], count: usize) -> Vec<ChartDateValue> {
    let [first, inner @ .., last] = values else {
        return values.to_vec();
    };
    if values.len() <= count {
        return values.to_vec();
    }
    let bucket_count = count.saturating_sub(2);
    let buckets: Vec<&[ChartDateValue]> = (0..bucket_count).map(|index| &inner[index * inner.len() / bucket_count..(index + 1) * inner.len() / bucket_count]).collect();
    let next_points = buckets.iter().skip(1).map(|bucket| average(bucket)).chain(iter::once(Point::from(last)));
    let kept = buckets.iter().zip(next_points).scan(Point::from(first), |previous, (bucket, next)| {
        let area = |value: &ChartDateValue| triangle_area(*previous, Point::from(value), next);
        let point = bucket.iter().max_by(|a, b| area(a).total_cmp(&area(b)))?;
        *previous = Point::from(point);
        Some(point)
    });
    iter::once(first).chain(kept).chain(iter::once(last)).cloned().collect()
}

#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

impl From<&ChartDateValue> for Point {
    fn from(value: &ChartDateValue) -> Self {
        Self {
            x: value.date.timestamp_millis() as f64,
            y: value.value,
        }
    }
}

fn average(values: &[ChartDateValue]) -> Point {
    let count = values.len() as f64;
    Point {
        x: values.iter().map(|value| Point::from(value).x).sum::<f64>() / count,
        y: values.iter().map(|value| value.value).sum::<f64>() / count,
    }
}

fn triangle_area(a: Point, b: Point, c: Point) -> f64 {
    ((b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y)).abs() / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reduced_points() {
        let bumps: Vec<ChartDateValue> = [0.0, 1.0, 5.0, 2.0, 3.0, 9.0, 4.0, 6.0].into_iter().zip(0..).map(|(value, second)| ChartDateValue::mock(second, value)).collect();
        let spiked: Vec<ChartDateValue> = (0..600).map(|second| ChartDateValue::mock(second, if second == 333 { 1000.0 } else { 100.0 })).collect();
        let reduced = reduced_points(&spiked, 120);

        assert_eq!(
            reduced_points(&bumps, 4),
            vec![bumps[0].clone(), bumps[2].clone(), bumps[5].clone(), bumps[7].clone()],
            "each bucket keeps the point that bends the line most"
        );
        assert_eq!((reduced.len(), reduced.first(), reduced.last()), (120, spiked.first(), spiked.last()));
        assert_eq!(reduced.iter().filter(|value| value.value == 1000.0).count(), 1, "a spike survives");
        for step in 1..300 {
            let steps: Vec<ChartDateValue> = (0..300).map(|second| ChartDateValue::mock(second, if second < step { 1.0 } else { 2.0 })).collect();
            assert!(reduced_points(&steps, 120).is_sorted_by(|a, b| a.date < b.date), "a step at {step} keeps the points in order, each once");
        }
    }
}
