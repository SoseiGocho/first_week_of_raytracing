use nalgebra as na;

mod ray_tracer;
use ray_tracer as rt;

fn main() {
    let camera =
        rt::Camera::new(16.0/9.0, 720, 2.0, 1.0);

    let world = rt::Visible::World(vec![
            rt::Visible::Sphere(rt::Sphere { center: na::point![0.0, 0.0, -1.0], radius: 0.5 }),
            rt::Visible::Sphere(rt::Sphere { center: na::point![0.0, -100.5, -1.0], radius: 100.0 })]);

    let render_buffer = camera.render(&world);

    render_buffer.save("../result/20240829_02_02_multi_object.png").unwrap();
}