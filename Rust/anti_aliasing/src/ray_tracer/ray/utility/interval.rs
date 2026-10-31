pub struct Interval {
    pub min: f64,
    pub max: f64,
}

impl Interval {
    pub fn new(_min: f64, _max: f64) -> Interval {
        Interval{ min: _min, max: _max }
    }

    pub fn full_fill() -> Interval {
        Interval{ min: std::f64::NEG_INFINITY, max: std::f64::INFINITY }
    }

    pub fn contains(&self, x: f64) -> bool {
        self.min <= x && x <= self.max
    }

    pub fn surrounds(&self, x: f64) -> bool {
        x <= self.min || x >= self.max
    }

    pub fn clamp(&self, x: f64) -> f64 {
        if x < self.min {
            self.min
        } else if x >= self.max {
            self.max
        } else {
            x
        }
    }

    pub const EMPTY: Interval = Interval { min: std::f64::INFINITY, max: std::f64::NEG_INFINITY };

    pub const UNIVERSE: Interval = Interval { min: std::f64::NEG_INFINITY, max: std::f64::INFINITY };
}
