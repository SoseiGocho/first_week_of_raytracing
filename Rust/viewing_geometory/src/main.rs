use core::f64;
use std::sync::Arc;

use chrono as ch;
use nalgebra as na;
use rand_xoshiro as xos;

mod ray_tracer;
use ray_tracer as rt;

fn main() {
    let camera = rt::Camera::new(16.0 / 9.0, 720, 1.0, 128, 64, rt::Degree{ degree: 90.0f64});

    let large_r = (f64::consts::PI / 4.0f64).cos();

    let left = Arc::new(rt::Material::Lambertian(rt::Lambertian{albedo: rt::Color::from([0.0f64, 0.0f64, 1.0f64])}));
    let right = Arc::new(rt::Material::Lambertian(rt::Lambertian{albedo: rt::Color::from([1.0f64, 0.0f64, 0.0f64])}));

    let world = rt::Visible::World(vec![
        rt::Visible::Sphere(rt::Sphere {
            center: na::point![-large_r, 0.0f64, -1.0f64],
            radius: large_r,
            material: left,
        }),
        rt::Visible::Sphere(rt::Sphere {
            center: na::point![large_r, 0.0f64, -1.0f64],
            radius: large_r,
            material: right,
        }),
    ]);

    let render_buffer = camera.render::<xos::Xoshiro256StarStar>(&world, 101);

    render_buffer
        .save(
            "./result/".to_string()
                + &ch::Local::now().format("%Y%m%d%H%M%S").to_string()
                + "_07_01_"
                + &std::env!("CARGO_CRATE_NAME")
                + ".png",
        )
        .unwrap();
}
