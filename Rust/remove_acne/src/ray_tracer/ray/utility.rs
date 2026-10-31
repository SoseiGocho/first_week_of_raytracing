use nalgebra as na;
use rand as rnd;

mod interval;
pub use interval::Interval;

mod math;

mod random_vector;
use random_vector::random_f64;
pub use random_vector::Pnt3;
pub use random_vector::UVec3;
pub use random_vector::Vec3;

fn new_random_vec3_between<T>(range: &Interval, st_rng: &mut T) -> Vec3
where
    T: rnd::Rng,
{
    na::vector![
        st_rng.gen_range(range.range.clone()),
        st_rng.gen_range(range.range.clone()),
        st_rng.gen_range(range.range.clone()),
    ]
}

fn new_random_unit_vec3<T>(st_rng: &mut T) -> UVec3
where
    T: rnd::Rng,
{
    loop {
        let p = new_random_vec3_between(&Interval::UNIT, st_rng);
        let sqlength = p.norm_squared();
        if sqlength <= 1.0f64 && sqlength >= f64::EPSILON {
            return UVec3::new_normalize(p);
        }
    }
}

pub fn new_random_vec3_on_a_correct_hemisphere<T>(normal: &Vec3, st_rng: &mut T) -> UVec3
where
    T: rnd::Rng,
{
    let unit_p = new_random_unit_vec3(st_rng);
    if unit_p.dot(normal) > 0.0f64 {
        unit_p
    } else {
        -unit_p
    }
}
