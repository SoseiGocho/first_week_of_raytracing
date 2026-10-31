use std::ops::Deref;

use image as im;
use image::Pixel;
use nalgebra as na;
use rand as rnd;

mod utility;
use utility::new_random_vec3_on_a_correct_hemisphere;
pub use utility::Interval;
pub use utility::Pnt3;
pub use utility::UVec3;
pub use utility::Vec3;

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

pub struct Ray {
    pub origin: Pnt3,
    pub direction: Vec3,
}

impl Ray {
    fn at(&self, t: f64) -> Pnt3 {
        self.origin + t * self.direction
    }

    pub fn color<T>(
        &self,
        world: &Visible,
        init_t: &Interval,
        max_depth: u32,
        st_rng: &mut T,
    ) -> Color
    where
        T: rnd::Rng,
    {
        let mut rays = Vec::<Option<Ray>>::with_capacity(max_depth as usize);
        let mut next_ray = Some(self);

        for depth_idx in 0..max_depth {
            rays.push(
                next_ray
                    .map(|r| {
                        world.hit(r, init_t).map(|x| {
                            let reflect_direction = new_random_vec3_on_a_correct_hemisphere(
                                &x.normal.unwrap_or_else(|x| x).into_inner(),
                                st_rng,
                            );
                            Ray {
                                origin: x.p,
                                direction: reflect_direction.into_inner(),
                            }
                        })
                    })
                    .flatten(),
            );
            next_ray = rays[depth_idx as usize].as_ref();
            if next_ray.is_none() {
                break;
            }
        }

        let celestial = {
            let last_ray = rays.last();
            let direction = match last_ray {
                Some(Some(r)) => r.direction,
                Some(None) => {
                    let num_of_rays = rays.len();
                    if num_of_rays > 1 {
                        rays[num_of_rays - 2].as_ref().unwrap().direction
                    } else {
                        self.direction
                    }
                }
                None => self.direction,
            };
            let unit_direction = UVec3::new_normalize(direction);
            let a = 0.5 * (unit_direction.y + 1.0);
            let color_vec: [f64; 3] =
                ((1.0 - a) * na::vector![1.0, 1.0, 1.0] + a * na::vector![0.5, 0.7, 1.0]).into();
            im::Rgb::from(color_vec)
        };

        rays.iter().rfold(celestial, |acc, x| match x {
            Some(_r) => acc.map(|c| 0.5f64 * c),
            None => acc,
        })

        // 本質的な問題は、FP的な書き方をしたそうにしているにも関わらず、
        // 再帰関数形ではなくループ形式でraysを構成する = 手続き的な手法を導入した
        // 自分にあって、そもそも手続き的な手法を導入したのは自分で
        // 手続き的な手法を始点にFP的な、Monadicな書き方がBorrow競合で書き辛い
        // と言い出すのは自業自得であり当たり前の話である
    }
}

pub const NORMAL_COLOR_RANGE: Interval = Interval {
    range: 0.0_f64..=1.0_f64,
};
