use std::sync::Arc;

use chrono as ch;
use nalgebra as na;
use rand_xoshiro as xos;

mod ray_tracer;
use ray_tracer as rt;

fn main() {
    let camera = rt::Camera::new(16.0 / 9.0, 360, 2.0, 1.0, 128, 64);

    let ground = Arc::new(rt::Material::Lambertian(rt::Lambertian {
        albedo: rt::Color::from([0.8f64, 0.8f64, 0.0f64]),
    }));
    let center = Arc::new(rt::Material::Lambertian(rt::Lambertian {
        albedo: rt::Color::from([0.1f64, 0.2f64, 0.5f64]),
    }));
    let left = Arc::new(rt::Material::Metal(rt::Metal::new(
        &rt::Color::from([0.8f64, 0.8f64, 0.8f64]),
        0.3f64,
    )));
    let right = Arc::new(rt::Material::Metal(rt::Metal::new(
        &rt::Color::from([0.8f64, 0.6f64, 0.2f64]),
        1.0,
    )));

    let world = rt::Visible::World(vec![
        rt::Visible::Sphere(rt::Sphere {
            center: na::point![0.0f64, -100.5f64, -1.0f64],
            radius: 100.0f64,
            material: ground,
        }),
        rt::Visible::Sphere(rt::Sphere {
            center: na::point![0.0f64, 0.0f64, -1.2f64],
            radius: 0.5f64,
            material: center,
        }),
        rt::Visible::Sphere(rt::Sphere {
            center: na::point![-1.0f64, 0.0f64, -1.0f64],
            radius: 0.5f64,
            material: left,
        }),
        rt::Visible::Sphere(rt::Sphere {
            center: na::point![1.0f64, 0.0f64, -1.0f64],
            radius: 0.5f64,
            material: right,
        }),
    ]);

    let render_buffer = camera.render::<xos::Xoshiro256StarStar>(&world, 101);

    render_buffer
        .save(
            "./result/".to_string()
                + &ch::Local::now().format("%Y%m%d%H%M%S").to_string()
                + "_05_02_ex_"
                + &std::env!("CARGO_CRATE_NAME")
                + ".png",
        )
        .unwrap();
}
