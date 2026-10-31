use std::ops::RangeInclusive;

pub struct Interval {
    pub range: RangeInclusive<f64>,
}

impl Interval {
    pub fn full_fill() -> Interval {
        Interval {
            range: f64::NEG_INFINITY..=f64::INFINITY,
        }
    }

    pub fn contains(&self, x: f64) -> bool {
        self.range.contains(&x)
    }

    pub fn surrounds(&self, x: f64) -> bool {
        x <= *self.range.start() || x >= *self.range.end()
    }

    pub fn clamp(&self, x: f64) -> f64 {
        if x < *self.range.start() {
            *self.range.start()
        } else if x > *self.range.end() {
            *self.range.end()
        } else {
            x
        }
    }

    pub fn min(&self) -> f64 {
        *self.range.start()
    }

    pub fn max(&self) -> f64 {
        *self.range.end()
    }

    pub const EMPTY: Interval = Interval {
        range: f64::INFINITY..=f64::NEG_INFINITY,
    };

    pub const UNIT: Interval = Interval {
        range: -1.0f64..=1.0f64,
    };

    pub const UNIVERSE: Interval = Interval {
        range: f64::NEG_INFINITY..=f64::INFINITY,
    };
}
