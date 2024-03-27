#[macro_use]
extern crate glium;

use std::time::{Duration, Instant};


use glium::{glutin::{self}, Surface};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

#[derive(Copy, Clone)]
struct Uniforms {
    screen_size: [f32; 2],
}

impl glium::uniforms::Uniforms for Uniforms {
    fn visit_values<'a, F: FnMut(&str, glium::uniforms::UniformValue<'a>)>(&'a self, mut f: F) {
        f("screen_size", glium::uniforms::UniformValue::Vec2(self.screen_size));
    }
}

// This macro implements the glium::Vertex trait for the Vertex struct.
implement_vertex!(Vertex, position);

fn main() {
    let target_frame_time = Duration::from_secs_f64(1.0 / 60.0); // 60 FPS
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_title("Circle from Quad");
    let cb = glutin::ContextBuilder::new();
    let display = match glium::Display::new(wb, cb, &event_loop) {
        Ok(display) => display,
        Err(e) => {
            eprintln!("Failed to create display: {:?}", e);
            return;
        }
    };

    // Define vertices of a quad using the Vertex struct
    let quad_vertices = [
        Vertex { position: [-0.05, -0.05] }, // bottom left
        Vertex { position: [0.05, -0.05] },  // bottom right
        Vertex { position: [0.05, 0.05] },   // top right
        Vertex { position: [-0.05, 0.05] },  // top left
    ];

    let vertex_buffer = glium::VertexBuffer::new(&display, &quad_vertices).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleFan);

    // Vertex and fragment shaders remain the same...
    let vertex_shader_src = r#"
        #version 140
        in vec2 position;
        void main() {
            gl_Position = vec4(position, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140
        uniform vec2 screen_size;
        out vec4 color;
        void main() {
            vec2 center = vec2(0.5, 0.5);
            vec2 pos = (gl_FragCoord.xy / screen_size) - center;
            float radius = 0.01;
            if (length(pos) > radius) {
                discard;
            }
            color = vec4(1.0, 1.0, 1.0, 1.0);
        }
    "#;
    // let fragment_shader_src = r#"
    //     #version 140
    //     out vec4 color;
    //     void main() {
    //         // Set the fragment's color to white (or any color of your choice)
    //         color = vec4(1.0, 1.0, 1.0, 1.0); // RGBA, where each component is in the range [0, 1]
    //     }
    // "#;

    let mut frame_counter: i64 = 0;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    event_loop.run(move |ev, _, control_flow| {
        let frame_start = Instant::now();

        frame_counter += 1;
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        let (width, height) = display.get_framebuffer_dimensions();
        let uniforms = Uniforms {
            screen_size: [width as f32, height as f32],
        };

        target.draw(&vertex_buffer, &indices, &program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();

        let elapsed = frame_start.elapsed();
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
            println!("{}, {}", width, height);
        }
    });
}
