use std::{time::{Instant, Duration}, sync::{Arc, Mutex}};
use glium::{glutin::{self}, Surface};
use crate::sim::state::State;
use crate::FRAMERATE;

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

pub fn renderer(state_container: Arc<Mutex<State>>) {
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_title("Universe Simulation");
    let cb = glutin::ContextBuilder::new();
    let display = match glium::Display::new(wb, cb, &el) {
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

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    el.run(move |ev, _, control_flow| {
        let frame_start = Instant::now();

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        let (width, height) = display.get_framebuffer_dimensions();
        let mut point_positions: Vec<Vertex> = vec![];

        {
            let state = state_container.lock().unwrap();
            for position in &state.positions {
                let (screen_x, screen_y) = (position.x, position.y);
                let this_point = Vertex {position: [screen_x as f32, screen_y as f32]};
                point_positions.push(this_point);
            }
        }

        let vertex_buffer = glium::VertexBuffer::new(&display, &point_positions).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::Points);
        let uniforms = Uniforms {
            screen_size: [width as f32, height as f32],
        };

        target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();

        let elapsed = frame_start.elapsed();
        let target_frame_time = Duration::from_nanos(1000000000/FRAMERATE);
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
    });
}
