use std::sync::Arc;

use image as im;
use image::Pixel;
use nalgebra as na;
use rand as rnd;

mod material;
pub use material::{Color, Dielectrics, Lambertian, Material, Metal};
use material::{HitRecord, MaterialProtocol};

pub use material::ray;
pub use ray::{Interval, Pnt3, Ray, UVec3, Vec3};

pub trait RayHittable {
    fn hit(&self, ray: &Ray, t: &Interval) -> Option<HitRecord>;
    fn color_rec<T>(
        &self,
        ray: &Ray,
        init_t: &Interval,
        current_depth: u32,
        max_depth: u32,
        st_rng: &mut T,
    ) -> Color
    where
        T: rnd::Rng,
    {
        let void_point: Color = im::Rgb::from([0.0f64, 0.0f64, 0.0f64]);
        self.hit(ray, init_t).map_or_else(
            || {
                let unit_direction = UVec3::new_normalize(ray.direction);
                let a = 0.5 * (unit_direction.y + 1.0);
                let color_vec: [f64; 3] = ((1.0 - a) * na::vector![1.0, 1.0, 1.0]
                    + a * na::vector![0.5, 0.7, 1.0])
                .into();
                im::Rgb::from(color_vec)
            },
            |x| {
                if current_depth >= max_depth {
                    void_point
                } else {
                    if let Some((attenuation, next_ray)) = x.material.scatter(ray, &x, st_rng) {
                        Color::from(
                            Vec3::from_column_slice(&attenuation.0)
                                .component_mul(&Vec3::from_column_slice(
                                    &self
                                        .color_rec(
                                            &next_ray,
                                            init_t,
                                            current_depth + 1,
                                            max_depth,
                                            st_rng,
                                        )
                                        .0,
                                ))
                                .data
                                .0[0],
                        )
                    } else {
                        void_point
                    }
                }
            },
        )
    }
    fn color<T>(&self, ray: &Ray, init_t: &Interval, max_depth: u32, st_rng: &mut T) -> Color
    where
        T: rnd::Rng,
    {
        self.color_rec::<T>(ray, init_t, 0, max_depth, st_rng)
    }
}
pub struct Sphere {
    pub center: Pnt3,
    pub radius: f64,
    pub material: Arc<Material>,
}

impl RayHittable for Sphere {
    fn hit(&self, ray: &Ray, t: &Interval) -> Option<HitRecord> {
        let oc = ray.origin - self.center;
        let a = ray.direction.dot(&ray.direction);
        let half_b = oc.dot(&ray.direction);
        let c = oc.dot(&oc) - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();
        let minus_root = (-half_b - sqrtd) / a;
        let nearest_root = if t.surrounds(minus_root) {
            let plus_root = (-half_b + sqrtd) / a;
            if t.surrounds(plus_root) {
                return None;
            } else {
                plus_root
            }
        } else {
            minus_root
        };

        let hit_point = ray.at(nearest_root);
        let unit_normal = UVec3::new_normalize((hit_point - self.center) / self.radius);
        Some(HitRecord {
            t: nearest_root,
            p: hit_point,
            material: self.material.clone(),
            normal: if ray.direction.dot(&unit_normal) < 0.0 {
                Ok(unit_normal)
            } else {
                Err(-unit_normal)
            },
        })
    }
}

pub enum Visible {
    Sphere(Sphere),
    World(Vec<Visible>),
}

impl RayHittable for Visible {
    fn hit(&self, ray: &Ray, t: &Interval) -> Option<HitRecord> {
        match self {
            Visible::Sphere(sphere) => sphere.hit(ray, t),
            Visible::World(visible) => visible.hit(ray, t),
        }
    }
}

impl RayHittable for Vec<Visible> {
    fn hit(&self, ray: &Ray, t: &Interval) -> Option<HitRecord> {
        self.iter()
            .fold((*t.range.end(), None), |acc, x| {
                x.hit(
                    ray,
                    &Interval {
                        range: *t.range.start()..=acc.0,
                    },
                )
                .map(|x| (x.t, Some(x)))
                .unwrap_or(acc)
            })
            .1
    }
}
