use image as im;
use nalgebra as na;

type Pnt3 = na::Point3<f64>;
type Vec3 = na::Vector3<f64>;
type UVec3 = na::UnitVector3<f64>;
type Color = im::Rgb<f64>;

struct Ray {
    origin: Pnt3,
    direction: Vec3,
}

impl Ray {
    fn at(&self, t:f64) -> Pnt3 {
        self.origin + t * self.direction
    }

    fn hit_sphere(&self, center: Pnt3, radius: f64)
        -> bool
    {
        let oc = self.origin - center;
        let a = self.direction.dot(&self.direction);
        let b = 2.0 * oc.dot(&self.direction);
        let c = oc.dot(&oc) - radius * radius;
        let discriminant = b * b - 4.0 * a * c;

        discriminant > 0.0
    }

    fn color(&self) -> Color {
        if self.hit_sphere(na::point![0.0, 0.0, -1.0], 0.5) {
            return Color::from([1.0, 0.0, 0.0])
        }

        let t = 0.5 * (UVec3::new_normalize(self.direction).y + 1.0);
        let color_vec =
            (1.0 - t) * na::vector![1.0, 1.0, 1.0]
                + t * na::vector![0.5, 0.7, 1.0];
        Color::from([ color_vec.x, color_vec.y, color_vec.z ])
    }
}

fn main() {
    let aspect_ratio = 16.0 / 9.0;
    let image_height = 720;
    let image_width = (image_height as f64 * aspect_ratio) as u32;

    let viewport_height = 2.0;
    let viewport_width = aspect_ratio * viewport_height;
    let focal_length = 1.0;

    let origin_point = na::point![0.0, 0.0, 0.0];
    let horizontal = na::vector![viewport_width, 0.0, 0.0];
    let vertical = na::vector![0.0, viewport_height, 0.0];
    let lower_left_corner =
        origin_point - horizontal / 2.0 - vertical / 2.0 - na::vector![0.0, 0.0, focal_length];

    let mut render_buffer =
        im::RgbImage::from_fn(
            image_width,
            image_height,
            |x, y| {
                let u = x as f64 / image_width as f64;
                let v = (image_height - y) as f64 / image_height as f64;
                let r = Ray {
                    origin: origin_point,
                    direction: lower_left_corner + u * horizontal + v * vertical - origin_point };
                im::Rgb::from(r.color().0.map(|x| { (x * 255.0 ) as u8 } ))
            });
    render_buffer.save("../result/20240828_01_sphere.png").unwrap();
}
