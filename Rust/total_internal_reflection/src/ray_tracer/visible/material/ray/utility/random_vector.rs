use nalgebra as na;
use rand as rnd;

pub type Pnt3 = na::Point3<f64>;
pub type UVec3 = na::UnitVector3<f64>;
pub type Vec3 = na::Vector3<f64>;

pub fn random_f64<T>(st_rng: &mut T) -> f64
where
    T: rnd::Rng,
{
    st_rng.gen::<f64>()
}

pub fn new_random_vec3<T>(st_rng: &mut T) -> Vec3
where
    T: rnd::Rng,
{
    na::vector![random_f64(st_rng), random_f64(st_rng), random_f64(st_rng)]
}
