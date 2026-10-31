use std::rc::Rc;

use image as im;
use rand as rnd;

pub mod ray;
use ray::utility::{near_zero, new_random_unit_vec3, reflect};
pub use ray::{Pnt3, Ray, UVec3};

pub type Color = im::Rgb<f64>;

pub struct HitRecord {
    pub t: f64,
    pub p: Pnt3,
    pub material: Rc<Material>,
    pub normal: Result<UVec3, UVec3>,
}

pub trait MaterialProtocol {
    fn scatter<T: rnd::Rng>(
        &self,
        ray: &Ray,
        rec: &HitRecord,
        st_rng: &mut T,
    ) -> Option<(Color, Ray)>;
}

pub struct Lambertian {
    pub albedo: Color,
}

impl MaterialProtocol for Lambertian {
    fn scatter<T>(&self, _ray: &Ray, rec: &HitRecord, st_rng: &mut T) -> Option<(Color, Ray)>
    where
        T: rnd::Rng,
    {
        let scatter_direction = rec.normal.unwrap_or_else(|x| x).into_inner()
            + new_random_unit_vec3(st_rng).into_inner();
        let normalized_scatter = if near_zero(&scatter_direction) {
            rec.normal.unwrap_or_else(|x| x).into_inner()
        } else {
            scatter_direction
        };
        Some((
            self.albedo,
            Ray {
                origin: rec.p,
                direction: normalized_scatter,
            },
        ))
    }
}

pub struct Metal {
    albedo: Color,
    fuzzyness: f64,
}

impl Metal {
    pub const fn new(color: &Color, fuzzy: f64) -> Metal {
        Metal {
            albedo: *color,
            fuzzyness: if fuzzy - 1.0f64 < f64::EPSILON {
                1.0f64
            } else {
                fuzzy
            },
        }
    }
}

impl MaterialProtocol for Metal {
    fn scatter<T>(&self, ray: &Ray, rec: &HitRecord, st_rng: &mut T) -> Option<(Color, Ray)>
    where
        T: rnd::Rng,
    {
        let reflected = UVec3::new_normalize(reflect(
            &ray.direction,
            &rec.normal.unwrap_or_else(|x| x).into_inner(),
        ))
        .into_inner()
            + self.fuzzyness * new_random_unit_vec3(st_rng).into_inner();
        let scattered = Ray {
            origin: rec.p,
            direction: reflected,
        };
        if scattered.direction.dot(&rec.normal.unwrap_or_else(|x| x)) > 0.0f64 {
            Some((self.albedo, scattered))
        } else {
            None
        }
    }
}

pub enum Material {
    Lambertian(Lambertian),
    Metal(Metal),
}

impl MaterialProtocol for Material {
    fn scatter<T: rand::Rng>(
        &self,
        ray: &Ray,
        rec: &HitRecord,
        st_rng: &mut T,
    ) -> Option<(Color, Ray)> {
        match self {
            Material::Lambertian(lambertian) => lambertian.scatter(ray, rec, st_rng),
            Material::Metal(metal) => metal.scatter(ray, rec, st_rng),
        }
    }
}
