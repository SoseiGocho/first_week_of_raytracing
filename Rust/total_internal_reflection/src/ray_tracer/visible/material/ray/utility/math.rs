pub struct Degree {
    pub degree: f64,
}

pub struct Radian {
    pub radian: f64,
}

impl From<Degree> for Radian {
    fn from(value: Degree) -> Self {
        Radian {
            radian: value.degree * std::f64::consts::PI / 180.0,
        }
    }
}
