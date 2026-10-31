use image as im;
use nalgebra as na;

mod interval;
pub use interval::Interval;

pub type Pnt3 = na::Point3<f64>;
pub type Vec3 = na::Vector3<f64>;
pub type UVec3 = na::UnitVector3<f64>;
pub type Color = im::Rgb<f64>;


pub struct HitRecord {
    pub t: f64,
    pub p: Pnt3,
    pub normal: Result<UVec3, UVec3>,
}

trait RayHittable {
    fn hit(&self, ray: &Ray, t: &Interval) -> Option<HitRecord>;
}
pub struct Sphere {
    pub center: Pnt3,
    pub radius: f64,
}

impl RayHittable for Sphere {
    fn hit(&self, ray: &Ray, t: &Interval) -> Option<HitRecord> {
        let oc = ray.origin - &self.center;
        let a = ray.direction.dot(&ray.direction);
        let half_b = oc.dot(&ray.direction);
        let c = oc.dot(&oc) - &self.radius * &self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None
        }

        let sqrtd = discriminant.sqrt();
        let minus_root = (-half_b - sqrtd) / a;
        let nearest_root =
            if t.surrounds(minus_root) {
                let plus_root =  (-half_b + sqrtd) / a;
                if t.surrounds(plus_root) {
                    return None
                } else {
                    plus_root
                }
            } else {
                minus_root
            };

        let hit_point = ray.at(nearest_root);
        let unit_normal = UVec3::new_normalize((hit_point - self.center) / self.radius);
        Some(HitRecord{
            t: nearest_root,
            p: hit_point,
            normal:
                if ray.direction.dot(&unit_normal) < 0.0 {
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
        self.into_iter().fold(
            (t.max, None),
            |acc, x| {
                let op_rec = x.hit(ray, &Interval::new(t.min, acc.0));
                op_rec.and_then(|x| {
                    Some((x.t, Some(x)))
                }).unwrap_or_else(|| {
                    (acc.0, acc.1)
                })
        }).1
    }
}


pub struct Ray {
    pub origin: Pnt3,
    pub direction: Vec3,
}

impl Ray {
    fn at(&self, t:f64) -> Pnt3 {
        self.origin + t * self.direction
    }

    pub fn color(&self, world: &Visible, init_t: &Interval) -> Color {
        world.hit(self, init_t).and_then(|x| {
            let color_vec: [f64; 3] =
                (0.5 * (
                    x.normal.unwrap_or_else(
                        |x| { return x }
                    ).into_inner()
                + na::vector![1.0, 1.0, 1.0])).into();
            Some(im::Rgb::from(color_vec))
        }).unwrap_or_else(|| {
            let unit_direction = UVec3::new_normalize(self.direction);
            let a = 0.5 * (unit_direction.y + 1.0);
            let color_vec: [f64; 3] =
                ((1.0 - a) * na::vector![1.0, 1.0, 1.0] + a * na::vector![0.5, 0.7, 1.0]).into();
            im::Rgb::from(color_vec)
        })
    }
}
