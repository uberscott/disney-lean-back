pub mod json;
pub mod data;
pub mod cache;

#[macro_use]
extern crate glium;

#[macro_use]
extern crate tokio;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate anyhow;

use std::io::{Cursor};
use std::sync::Arc;
use bytes::Bytes;
use anyhow::Error;
use crate::data::Data;
use crate::cache::cache_it_all;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    #[allow(unused_imports)]
    use glium::{glutin, Surface};

    let event_loop = glutin::event_loop::EventLoop::<Call>::with_user_event();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let image = image::load(Cursor::new(&include_bytes!("../images/peanut-head.jpg")),
                            image::ImageFormat::Jpeg).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    let texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();

    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 2],
        tex_coords: [f32; 2],
    }

    implement_vertex!(Vertex, position, tex_coords);

    let vertex1 = Vertex { position: [-0.5, -0.5], tex_coords: [0.0, 0.0] };
    let vertex2 = Vertex { position: [ -0.5,  0.5], tex_coords: [0.0, 1.0] };
    let vertex3 = Vertex { position: [ 0.5, -0.5], tex_coords: [1.0, 0.0] };


    let vertex4 = Vertex { position: [ 0.5, 0.5], tex_coords: [1.0, 1.0] };
    let vertex5 = Vertex { position: [ -0.5,  0.5], tex_coords: [0.0, 1.0] };
    let vertex6 = Vertex { position: [ 0.5, -0.5], tex_coords: [1.0, 0.0] };

    let shape = vec![vertex1, vertex2, vertex3, vertex4, vertex5, vertex6 ];


    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let vertex_shader_src = r#"
        #version 140
        in vec2 position;
        in vec2 tex_coords;
        out vec2 v_tex_coords;
        uniform mat4 matrix;
        void main() {
            v_tex_coords = tex_coords;
            gl_Position = matrix * vec4(position, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140
        in vec2 v_tex_coords;
        out vec4 color;
        uniform sampler2D tex;
        void main() {
            color = texture(tex, v_tex_coords);
        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    let proxy = event_loop.create_proxy();
    let mut texture_cache: HashMap<String,glium::texture::SrgbTexture2d> = HashMap::new();

    let mut t = -0.5;
    event_loop.run(move |event, _, control_flow| {


        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                _ => return,
            },
            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => {
                    let proxy = proxy.clone();
                    tokio::spawn( async move {
println!("Init!");
                        match json::fetch().await {
                            Ok(data) => {
                                // after a successful fetch, let's cache everything
                              cache_it_all(data,proxy).await;
                            }
                            Err(err) => {
                                eprintln!("bad news, the fetch didn't go so well: {}",err.to_string());
                            }
                        }
                    });
                },
                _ => return,
            },

            glutin::event::Event::UserEvent(call) => match call {
                Call::ToTexture{bytes,url} => {

                    fn to_texture(bytes: Bytes, url: String, display: &glium::Display , texture_cache: &mut HashMap<String,glium::texture::SrgbTexture2d> ) -> Result<(),Error> {
                        let image = image::load(Cursor::new(bytes ),
                                                image::ImageFormat::Jpeg)?.to_rgba8();
                        let image_dimensions = image.dimensions();
                        let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
                        let texture = glium::texture::SrgbTexture2d::new(display, image)?;
                        texture_cache.insert( url, texture );
                        Ok(())
                    }

                    match to_texture(bytes,url.clone(), &display, &mut texture_cache) {
                        Ok(_) => {
                            println!("ToTexture: {}", url);
                        }
                        Err(error) => {
                            println!("ToTexture: {} ERROR: {}", url, error.to_string());
                        }
                    }
               }
                Call::TextureCachingBatchComplete => {
                    println!("TextureCachingBatchComplete")
                }
            },
            _ => return,
        }

        let next_frame_time = std::time::Instant::now() +
            std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        // we update `t`
        t += 0.0002;
        if t > 0.5 {
            t = -0.5;
        }

        let mut target = display.draw();
        target.clear_color(
        0.129, 0.588, 0.953, 1.0 );

        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [ t , 0.0, 0.0, 1.0f32],
            ],
            tex: &texture,
        };

        target.draw(&vertex_buffer, &indices, &program, &uniforms,
                    &Default::default()).unwrap();
        target.finish().unwrap();
    });
}


pub enum Call {
    ToTexture{ bytes: Bytes, url: String },
    TextureCachingBatchComplete
}