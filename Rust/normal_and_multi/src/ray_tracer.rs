use image as im;
use nalgebra as na;

pub mod ray;
pub use ray::{ Vec3, Pnt3, Interval, Sphere, Visible, Ray };

pub struct Camera {
    pub aspect_ratio: f64,
    pub image_width: u32,
    image_height: u32,
    center: Pnt3,
    pixel00_loc: Pnt3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
}

impl Camera {
    pub fn new(aspect: f64,
        height: u32,
        viewport_height: f64,
        focal_length: f64) -> Camera
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
        }
    }

    pub fn render(&self, visible: &Visible) -> im::RgbImage {
        im::RgbImage::from_fn(
            self.image_width,
            self.image_height,
            |x, y| {
                let pixel_center = self.pixel00_loc + ( x as f64 * self.pixel_delta_u) + ( y as f64 * self.pixel_delta_v);
                let ray_direction = pixel_center - self.center;
                let r = Ray {
                    origin: self.center,
                    direction: ray_direction };
                im::Rgb::from(r.color(visible, &Interval::new(0.0, std::f64::INFINITY)).0.map(|x| { (x * 255.0 ) as u8 } ))
            })
    }
}
