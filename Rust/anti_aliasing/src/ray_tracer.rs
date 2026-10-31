use image as im;
use nalgebra as na;
use rand as rnd;

pub mod ray;
pub use ray::{ Vec3, Pnt3, Interval, Sphere, Visible, Ray };

pub fn perturbation<T>(st_rng: &mut T) -> Vec3
    where T: rnd::Rng,
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
}

impl Camera {
    pub fn new(aspect: f64,
        height: u32,
        viewport_height: f64,
        focal_length: f64,
        num_of_samples: u32) -> Camera
    {
        let calculated_width = (aspect * height as f64) as u32;
        let width = if calculated_width < 1 { 1 } else { calculated_width };

        let viewport_width = aspect * viewport_height;
        let camera_center = na::point![0.0, 0.0, 0.0];

        let viewport_u = na::vector![viewport_width, 0.0, 0.0];
        let viewport_v = na::vector![0.0, -viewport_height, 0.0];

        let delta_u = viewport_u / width as f64;
        let delta_v = viewport_v / height as f64;

        let viewport_upper_left =
            camera_center
                - viewport_u / 2.0 - viewport_v / 2.0
                - na::vector![0.0, 0.0, focal_length];

        Camera {
            aspect_ratio: aspect,
            image_width: width,
            image_height: height,
            center: camera_center,
            pixel_delta_u: delta_u,
            pixel_delta_v: delta_v,
            pixel00_loc: viewport_upper_left + 0.5 * ( delta_u + delta_v),
            samples_per_pixel: num_of_samples,
            pixel_samples_scale: 1.0 / num_of_samples as f64,
        }
    }

    fn get_ray<T>(&self, i: u32, j: u32, st_rng: &mut T) -> Ray
        where T: rnd::Rng
    {
        let offset = perturbation(st_rng);
        let pixel_sample = self.pixel00_loc
            + ((i as f64 + offset.x) * self.pixel_delta_u)
            + ((j as f64 + offset.y) * self.pixel_delta_v);

        let ray_origin = self.center;
        let ray_direction = pixel_sample - ray_origin;

        Ray { origin: ray_origin, direction: ray_direction }
    }

    pub fn render<T>(&self, visible: &Visible, st_rng: &mut T) -> im::RgbImage
        where T: rnd::Rng
    {
        im::RgbImage::from_fn(
            self.image_width,
            self.image_height,
            |x, y| {
                let init_t = Interval::new(0.0, std::f64::INFINITY);

                let mut acc_color_vec: Vec3 = na::vector![0.0, 0.0, 0.0];
                for _i in 0..self.samples_per_pixel {
                    let r = self.get_ray(x, y, st_rng);
                    acc_color_vec += Vec3::from(r.color(visible, &init_t).0);
                }

                let acc_color_raw_vec:[u8; 3] = acc_color_vec.map(|x| { (ray::NORMAL_COLOR_RANGE.clamp(self.pixel_samples_scale * x)  * 255.0) as u8 }).into();
                im::Rgb::from(acc_color_raw_vec)
            })
    }
}
