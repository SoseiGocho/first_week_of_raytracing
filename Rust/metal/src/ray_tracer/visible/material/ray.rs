pub mod utility;
pub use utility::{Interval, Pnt3, UVec3, Vec3};

pub struct Ray {
    pub origin: Pnt3,
    pub direction: Vec3,
}

impl Ray {
    pub fn at(&self, t: f64) -> Pnt3 {
        self.origin + t * self.direction
    }
}

pub const NORMAL_COLOR_RANGE: Interval = Interval {
    range: 0.0_f64..=1.0_f64,
};
