use nalgebra as na;
use rand as rnd;

mod interval;
pub use interval::Interval;

pub mod math;

mod random_vector;
pub use random_vector::Pnt3;
pub use random_vector::UVec3;
pub use random_vector::Vec3;

// 区間乱数を生成するのにclampしていたバカタレが居るらしい

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

pub fn new_random_unit_vec3<T>(st_rng: &mut T) -> UVec3
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

pub fn gamma(linear_color_elem: f64) -> f64 {
    linear_color_elem.powf(1.0f64 / 2.2f64)
}

pub fn near_zero(v: &Vec3) -> bool {
    const S: f64 = 1e-8f64;
    v.x.abs() < S && v.y.abs() < S && v.z.abs() < S
}

pub fn reflect(v: &Vec3, n: &Vec3) -> Vec3 {
    v - 2.0f64 * v.dot(n) * n
}

pub fn refract(uv: &UVec3, n: &UVec3, etai_over_etat: f64) -> Vec3 {
    let cos_theta = -uv.dot(n);
    let r_out_parallel = etai_over_etat * (uv.into_inner() + cos_theta * n.into_inner());
    let r_out_perp = -(1.0f64 - r_out_parallel.norm_squared()).sqrt() * n.into_inner();
    r_out_parallel + r_out_perp
}

pub fn reflectance(cos_theta: f64, refraction_index: f64) -> f64 {
    let r0 = (1.0f64 - refraction_index) / (1.0f64 + refraction_index);
    let r0_squared = r0 * r0;
    r0_squared + (1.0f64 - r0_squared) * (1.0f64 - cos_theta).powi(5)
}

pub fn new_random_vec3_in_unit_disk<T>(st_rng: &mut T) -> Vec3
where
    T: rnd::Rng,
{
    loop {
        let p = na::vector![st_rng.gen::<f64>(), st_rng.gen::<f64>(), 0.0f64];
        if p.norm_squared() < 1.0f64 {
            return p;
        }
    }
}
