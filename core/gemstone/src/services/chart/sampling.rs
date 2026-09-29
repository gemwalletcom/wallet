use std::iter;

use primitives::ChartDateValue;

pub fn sampled_values(values: &[ChartDateValue], count: usize) -> Vec<ChartDateValue> {
    if values.len() <= count {
        return values.to_vec();
    }
    let [first, inner @ .., last] = values else {
        return values.to_vec();
    };
    let buckets = count.saturating_sub(2);
    let edge = |index: usize| index * inner.len() / buckets;
    let bucket = |index: usize| &inner[edge(index)..edge(index + 1)];
    let next = |index: usize| match index + 1 < buckets {
        true => centroid(bucket(index + 1)),
        false => Point::from(last),
    };
    let picked = (0..buckets).scan(first, |previous, index| {
        let from = Point::from(*previous);
        let to = next(index);
        let area = |value: &ChartDateValue| triangle_area(from, Point::from(value), to);
        *previous = bucket(index).iter().max_by(|a, b| area(a).total_cmp(&area(b)))?;
        Some(*previous)
    });
    iter::once(first).chain(picked).chain(iter::once(last)).cloned().collect()
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

fn centroid(values: &[ChartDateValue]) -> Point {
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
    fn test_sampled_values() {
        let bumps: Vec<ChartDateValue> = [0.0, 1.0, 5.0, 2.0, 3.0, 9.0, 4.0, 6.0].into_iter().zip(0..).map(|(value, second)| ChartDateValue::mock(second, value)).collect();
        let spiked: Vec<ChartDateValue> = (0..600).map(|second| ChartDateValue::mock(second, if second == 333 { 1000.0 } else { 100.0 })).collect();
        let sampled = sampled_values(&spiked, 120);

        assert_eq!(
            sampled_values(&bumps, 4),
            vec![bumps[0].clone(), bumps[2].clone(), bumps[5].clone(), bumps[7].clone()],
            "each bucket keeps the point spanning the largest triangle"
        );
        assert_eq!((sampled.len(), sampled.first(), sampled.last()), (120, spiked.first(), spiked.last()));
        assert_eq!(sampled.iter().filter(|value| value.value == 1000.0).count(), 1, "a spike survives the sampling");
        for step in 1..300 {
            let steps: Vec<ChartDateValue> = (0..300).map(|second| ChartDateValue::mock(second, if second < step { 1.0 } else { 2.0 })).collect();
            assert!(sampled_values(&steps, 120).is_sorted_by(|a, b| a.date < b.date), "a step at {step} keeps the points in order, each once");
        }
    }
}
