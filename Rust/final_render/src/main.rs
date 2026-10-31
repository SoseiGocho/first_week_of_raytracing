use core::f64;
use std::sync::Arc;

use chrono as ch;
use nalgebra as na;
use rand::{Rng, SeedableRng};
use rand_xoshiro as xos;

mod ray_tracer;
use ray_tracer::{self as rt};

fn main() {
    let mut object_list: Vec<rt::Visible> = Vec::new();
    object_list.reserve(512);

    let ground = Arc::new(rt::Material::Lambertian(rt::Lambertian {
        albedo: rt::Color::from([0.5f64, 0.5f64, 0.5f64]),
    }));

    object_list.push(rt::Visible::Sphere(rt::Sphere {
        center: na::point![0.0f64, -1000.0f64, 0.0f64],
        radius: 1000.0,
        material: ground,
    }));

    let mut st_rng = xos::Xoshiro256StarStar::seed_from_u64(103);

    for i in -11..11 {
        for j in -11..11 {
            let center = na::point![
                i as f64 + 0.9 * st_rng.gen::<f64>(),
                0.2f64,
                j as f64 + 0.9f64 * st_rng.gen::<f64>()
            ];

            let choose_material = st_rng.gen::<f64>();

            if (center - na::point![4.0f64, 0.2f64, 0.0f64]).norm() > 0.9f64 {
                if choose_material < 0.8f64 {
                    // NOTE: 以前は単位ベクトル同士の成分積を使っており、各成分が
                    // [-1, 1] のため負の albedo が生成されていた。書籍と同様に
                    // [0, 1) の乱数同士の積にする。
                    let albedo = rt::Color::from([
                        st_rng.gen::<f64>() * st_rng.gen::<f64>(),
                        st_rng.gen::<f64>() * st_rng.gen::<f64>(),
                        st_rng.gen::<f64>() * st_rng.gen::<f64>(),
                    ]);

                    object_list.push(rt::Visible::Sphere(rt::Sphere {
                        center,
                        radius: 0.2f64,
                        material: Arc::new(rt::Material::Lambertian(rt::Lambertian { albedo })),
                    }));
                } else if choose_material < 0.95f64 {
                    let albedo = rt::Color::from([
                        st_rng.gen_range(0.5f64..=1.0f64),
                        st_rng.gen_range(0.5f64..=1.0f64),
                        st_rng.gen_range(0.5f64..=1.0f64),
                    ]);
                    let fuzzyness = st_rng.gen_range(0.0f64..0.5f64);

                    object_list.push(rt::Visible::Sphere(rt::Sphere {
                        center,
                        radius: 0.2f64,
                        material: Arc::new(rt::Material::Metal(rt::Metal::new(&albedo, fuzzyness))),
                    }))
                } else {
                    object_list.push(rt::Visible::Sphere(rt::Sphere {
                        center,
                        radius: 0.2f64,
                        material: Arc::new(rt::Material::Dielectrics(rt::Dielectrics {
                            refraction_idx: 1.5,
                        })),
                    }))
                }
            }
        }
    }

    object_list.push(rt::Visible::Sphere(rt::Sphere {
        center: na::point![0.0f64, 1.0f64, 0.0f64],
        radius: 1.0f64,
        material: Arc::new(rt::Material::Dielectrics(rt::Dielectrics {
            refraction_idx: 1.5f64,
        })),
    }));

    object_list.push(rt::Visible::Sphere(rt::Sphere {
        center: na::point![-4.0f64, 1.0f64, 0.0f64],
        radius: 1.0f64,
        material: Arc::new(rt::Material::Lambertian(rt::Lambertian {
            albedo: rt::Color::from([0.4f64, 0.2f64, 0.1f64]),
        })),
    }));

    object_list.push(rt::Visible::Sphere(rt::Sphere {
        center: na::point![4.0f64, 1.0f64, 0.0f64],
        radius: 1.0f64,
        material: Arc::new(rt::Material::Metal(rt::Metal::new(
            &rt::Color::from([0.7f64, 0.6f64, 0.5f64]),
            0.0f64,
        ))),
    }));

    let world = rt::Visible::World(object_list);

    let lookfrom = na::point![13.0f64, 2.0f64, 3.0f64];
    let lookat = na::point![0.0f64, 0.0f64, 0.0f64];
    let vup = na::vector![0.0f64, 1.0f64, 0.0f64];
    let camera = rt::Camera::new(
        16.0 / 9.0,
        360,
        128,
        64,
        rt::Degree { degree: 20.0f64 },
        &lookfrom,
        &lookat,
        &vup,
        rt::Degree { degree: 0.6f64 },
        10.0f64,
    );

    let render_buffer = camera.render::<xos::Xoshiro256StarStar>(&world, 101);

    render_buffer
        .save(
            "./result/".to_string()
                + &ch::Local::now().format("%Y%m%d%H%M%S").to_string()
                + "_09_"
                + &std::env!("CARGO_CRATE_NAME")
                + ".png",
        )
        .unwrap();
}
