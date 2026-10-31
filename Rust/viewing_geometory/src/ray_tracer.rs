use std::ops::DerefMut;
use std::ops::Index;
use std::sync::Arc;
use std::sync::Mutex;

use image as im;
use nalgebra as na;
use rand as rnd;
use rayon::prelude::*;

mod visible;
use visible::ray::utility::gamma;
use visible::ray::utility::math::Radian;
pub use visible::{
    ray, Color, Dielectrics, Interval, Lambertian, Material, Metal, Pnt3, Ray, RayHittable, Sphere,
    Vec3, Visible,
};
pub use visible::ray::utility::math::Degree;

pub fn perturbation<T>(st_rng: &mut T) -> Vec3
where
    T: rnd::Rng,
{
    na::vector![
        (st_rng.gen::<f64>() - 0.5),
        (st_rng.gen::<f64>() - 0.5),
        0.0
    ]
}

pub struct Camera {
    pub aspect_ratio: f64,
    pub image_width: u32,
    image_height: u32,
    center: Pnt3,
    pixel00_loc: Pnt3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    pub samples_per_pixel: u32,
    pixel_samples_scale: f64,
    pub max_reflects_depth: u32,
    vfov: Radian,
}

impl Camera {
    pub fn new(
        aspect: f64,
        height: u32,
        focal_length: f64,
        num_of_samples: u32,
        num_of_reflects_depth: u32,
        vfov: Degree,
    ) -> Camera {
        let calculated_width = (aspect * height as f64) as u32;
        let width = if calculated_width < 1 {
            1
        } else {
            calculated_width
        };

        let camera_center = na::point![0.0, 0.0, 0.0];

        let theta: Radian = vfov.into();
        let h = (theta.radian / 2.0f64).tan();
        let viewport_height = 2.0f64 * h * focal_length;
        let viewport_width = aspect * viewport_height;

        let viewport_u = na::vector![viewport_width, 0.0, 0.0];
        let viewport_v = na::vector![0.0, -viewport_height, 0.0];

        let delta_u = viewport_u / width as f64;
        let delta_v = viewport_v / height as f64;

        let viewport_upper_left = camera_center
            - viewport_u / 2.0
            - viewport_v / 2.0
            - na::vector![0.0, 0.0, focal_length];

        Camera {
            aspect_ratio: aspect,
            image_width: width,
            image_height: height,
            center: camera_center,
            pixel_delta_u: delta_u,
            pixel_delta_v: delta_v,
            pixel00_loc: viewport_upper_left + 0.5 * (delta_u + delta_v),
            samples_per_pixel: num_of_samples,
            pixel_samples_scale: 1.0 / num_of_samples as f64,
            max_reflects_depth: num_of_reflects_depth,
            vfov: theta
        }
    }

    fn get_ray<T>(&self, i: u32, j: u32, st_rng: &mut T) -> Ray
    where
        T: rnd::Rng,
    {
        let offset = perturbation(st_rng);
        let pixel_sample = self.pixel00_loc
            + ((i as f64 + offset.x) * self.pixel_delta_u)
            + ((j as f64 + offset.y) * self.pixel_delta_v);

        let ray_origin = self.center;
        let ray_direction = pixel_sample - ray_origin;

        Ray {
            origin: ray_origin,
            direction: ray_direction,
        }
    }

    pub fn render<T>(&self, visible: &Visible, random_seed: u64) -> im::RgbImage
    where
        T: rnd::Rng + rnd::SeedableRng + std::fmt::Debug + std::marker::Send + std::marker::Sync,
    {
        let mut initial_rng_state = T::seed_from_u64(random_seed);
        let thread_local_rngs_state: Vec<Arc<Mutex<T>>> = (0..rayon::current_num_threads())
            .map(|_| Arc::new(Mutex::new(T::seed_from_u64(initial_rng_state.gen::<u64>()))))
            .collect();

        let mut img = im::RgbImage::new(self.image_width, self.image_height);
        img.enumerate_pixels_mut()
            .collect::<Vec<(u32, u32, &mut im::Rgb<u8>)>>()
            .par_iter_mut()
            .for_each(|(x, y, pixel)| {
                const INIT_T: Interval = Interval {
                    range: 0.001f64..=f64::INFINITY,
                };

                let rng_state = thread_local_rngs_state
                    .index(rayon::current_thread_index().unwrap())
                    .clone();
                let mut rng_state = rng_state.lock().unwrap();
                let st_rng = rng_state.deref_mut();

                let acc_color_vec = (0..self.samples_per_pixel).fold(Vec3::zeros(), |acc, _i| {
                    acc + Vec3::from(
                        visible
                            .color::<T>(
                                &self.get_ray(*x, *y, st_rng),
                                &INIT_T,
                                self.max_reflects_depth,
                                st_rng,
                            )
                            .0,
                    )
                });

                let acc_color_raw_vec: [u8; 3] = acc_color_vec
                    .map(|x| {
                        (ray::NORMAL_COLOR_RANGE.clamp(gamma(self.pixel_samples_scale * x)) * 255.0)
                            as u8
                    })
                    .into();
                pixel.0 = acc_color_raw_vec;
            });
        img
    }
}
