use chrono as ch;
use nalgebra as na;
use rand::SeedableRng;
use rand_xoshiro as xos;

mod ray_tracer;
use ray_tracer as rt;

fn main() {
    let camera = rt::Camera::new(16.0 / 9.0, 720, 2.0, 1.0, 128, 64);

    let world = rt::Visible::World(vec![
        rt::Visible::Sphere(rt::Sphere {
            center: na::point![0.0, 0.0, -1.0],
            radius: 0.5,
        }),
        rt::Visible::Sphere(rt::Sphere {
            center: na::point![0.0, -100.5, -1.0],
            radius: 100.0,
        }),
    ]);

    let mut rng_state = xos::Xoshiro256StarStar::seed_from_u64(101);
    let render_buffer = camera.render(&world, &mut rng_state);

    render_buffer
        .save(
            "./result/".to_string()
                + &ch::Local::now().format("%Y%m%d%H%M%S").to_string()
                + "_04_03_"
                + &std::env!("CARGO_CRATE_NAME")
                + ".png",
        )
        .unwrap();
}
