/// This is the main library crate for a Universe Simulation.
/// It makes use of the `glium` library for rendering.

#[macro_use]
extern crate glium;
use std::time::{Duration, Instant};
mod sim;
use glium::{glutin::{self}, Surface};

/// A structure for representing a vertex with a 2D position.
#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

/// A structure for holding uniform values passed to the shader.
#[derive(Copy, Clone)]
struct Uniforms {
    screen_size: [f32; 2],
}

impl glium::uniforms::Uniforms for Uniforms {
    /// Visits the uniform values, allowing them to be used in the shader.
    fn visit_values<'a, F: FnMut(&str, glium::uniforms::UniformValue<'a>)>(&'a self, mut f: F) {
        f("screen_size", glium::uniforms::UniformValue::Vec2(self.screen_size));
    }
}

// This macro implements the glium::Vertex trait for the Vertex struct.
implement_vertex!(Vertex, position);

/// The main function sets up the rendering window, runs the simulation loop, and handles events.
fn main() {
    let target_frame_time = Duration::from_secs_f64(1.0 / 60.0); // 60 FPS
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_title("Universe Simulation");
    let cb = glutin::ContextBuilder::new();
    let display = match glium::Display::new(wb, cb, &event_loop) {
        Ok(display) => display,
        Err(e) => {
            eprintln!("Failed to create display: {:?}", e);
            return;
        }
    };

    let vertex_shader_src = r#"
        #version 140
        in vec2 position;
        void main() {
            gl_Position = vec4(position, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140
        out vec4 color;
        void main() {
            color = vec4(1.0, 1.0, 1.0, 1.0);
        }
    "#;

    let mut frame_counter: i64 = 0;
    let mut moving_fps: f32 = 0.0;
    let mut sim = sim::Simulation2D::gen_simulation_2d();

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    event_loop.run(move |ev, _, control_flow| {
        let frame_start = Instant::now();

        frame_counter += 1;
        sim.step();
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        let (width, height) = display.get_framebuffer_dimensions();
        let mut point_positions: Vec<Vertex> = vec![];

        for star in sim.get_stars() {
            let (screen_x, screen_y) = sim::point2d_to_screen_coords((star.x, star.y), (width, height), &sim.sim_domain).unwrap_or_else(|| (2.0, 2.0));
            let this_point = Vertex {position: [screen_x as f32, screen_y as f32]};
            point_positions.push(this_point);
        }

        let vertex_buffer = glium::VertexBuffer::new(&display, &point_positions).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::Points);
        let uniforms = Uniforms {
            screen_size: [width as f32, height as f32],
        };

        target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();

        let elapsed = frame_start.elapsed();
        moving_fps = (0.2 * 1000000.0 / (elapsed.as_micros() as f32)) + (0.8 * moving_fps);
        if elapsed < target_frame_time {
            std::thread::sleep(target_frame_time - elapsed);
        }

        *control_flow = match ev {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => glutin::event_loop::ControlFlow::Exit,
                _ => glutin::event_loop::ControlFlow::Poll,
            },
            _ => glutin::event_loop::ControlFlow::Poll,
        };
        if (frame_counter % 60) == 0 {
            println!("({}, {})    average fps: {}", width, height, moving_fps);
        }
    });
}
