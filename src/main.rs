use std::collections::HashMap;
use std::io::Cursor;
use std::ops::Neg;
use std::sync::Arc;

use anyhow::Error;
use bytes::Bytes;
use glam::{Affine2, Affine3A, Mat4, Vec2, Vec3, Vec4};
use glium::{Display, Frame, Surface};
use glium::backend::glutin::glutin::dpi::{PhysicalSize, Size};
use glium::backend::glutin::glutin::event::VirtualKeyCode;
use glium::backend::glutin::glutin::event_loop::EventLoopProxy;
use glium::backend::glutin::glutin::platform::macos::WindowBuilderExtMacOS;
use glium::glutin::event_loop::EventLoop;
use glium::glutin::window::WindowBuilder;
use image::imageops::tile;
use nalgebra::{Matrix4, MatrixSlice4};
use nalgebra_glm::make_mat4x4;
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};

use ux::{Renderers, Grid, Lerper, TexturedVertex, TileRenderer, Vertex};

use crate::cache::{cache_set, create_cacher};
use crate::data::{Data, Item, Set};

pub mod data;
pub mod cache;
pub mod ux;

#[macro_use]
extern crate glium;

#[macro_use]
extern crate tokio;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate anyhow;

#[tokio::main]
async fn main() {

    let event_loop = glium::glutin::event_loop::EventLoop::<Call>::with_user_event();
    let display = init_display(&event_loop);
    let texture_tile_renderer = TileRenderer::<TexturedVertex>::new(&display);
    let color_tile_renderer = TileRenderer::<Vertex>::new(&display);

    let proxy = event_loop.create_proxy();

    let context = Arc::new(Renderers::new(texture_tile_renderer, color_tile_renderer).await );
    let mut grid = Grid::new();

    let mut texture_cache : HashMap<String,glium::texture::SrgbTexture2d> = HashMap::new();

    event_loop.run(move |event, _, control_flow| {

        match event {
            glium::glutin::event::Event::WindowEvent { event, .. } => match event {
                glium::glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glium::glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                glium::glutin::event::WindowEvent::KeyboardInput{input,..}=> {

                    match input.virtual_keycode {
                        None => {}
                        Some(key) => {
                            let vert_nudge = 1.0;
                            match key {
                                VirtualKeyCode::Escape=> {
                                    *control_flow = glium::glutin::event_loop::ControlFlow::Exit;
                                    return;
                                }
                                VirtualKeyCode::Up=> {
                                    grid.up();
                                }
                                VirtualKeyCode::Down=> {
                                    grid.down();
                                }
                                VirtualKeyCode::Left=> {
                                    grid.left();
                                }
                                VirtualKeyCode::Right=> {
                                    grid.right();
                                }
                                _ => {
                                    return;
                                }
                            }
                        }
                    }

                }
                _ => return,
            },
            glium::glutin::event::Event::NewEvents(cause) => match cause {
                glium::glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glium::glutin::event::StartCause::Init => {
                    let proxy = proxy.clone();
                    tokio::spawn( async move {
println!("Init!");
                        data::fetch(proxy).await.unwrap_or_default();
                    });
                },
                _ => return,
            },

            glium::glutin::event::Event::UserEvent(call) => match call {
                Call::ToTexture{bytes,url} => {

                    fn to_texture(bytes: Bytes, url: String, display: &glium::Display , texture_cache: &mut HashMap<String,glium::texture::SrgbTexture2d>) -> Result<(),Error> {
                        let image = image::load(Cursor::new(bytes ),
                                                image::ImageFormat::Jpeg)?.to_rgba8();
                        let image_dimensions = image.dimensions();
                        let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
                        let texture = glium::texture::SrgbTexture2d::new(display, image)?;
                        {
                            texture_cache.insert(url, texture);
                        }
                        Ok(())
                    }

                    match to_texture(bytes,url.clone(), &display, & mut texture_cache ) {
                        Ok(_) => {
                            println!("ToTexture: {}", url);
                            return;
                        }
                        Err(error) => {
                            println!("ToTexture: {} ERROR: {}", url, error.to_string());
                            return;
                        }
                    }
               }
                Call::TextureCachingBatchComplete => {
                    println!("TextureCachingBatchComplete")
                }
                Call::AddSet(set) => {
                    grid.add(set);
                }
            },
            _ => return,
        }

        let next_frame_time = std::time::Instant::now() +
            std::time::Duration::from_nanos(16_666_667);
        *control_flow = glium::glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);


        let mut frame = display.draw();
        frame.clear_color_and_depth(
            (0.129, 0.588, 0.953, 1.0), 1.0 );

        let (width, height) = frame.get_dimensions();
        let aspect_ratio = height as f32 / width as f32;

        let mut aspect_matrix = Mat4::from_cols_array_2d(&[
            [aspect_ratio, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [ 0.0 , 0.0, 0.0, 1.0],]);

        let matrix = {
            let size = 5.0;
            let matrix:Matrix4<f32> = Matrix4::new_orthographic( 0.0,size, size, 0.0, -10.0, 10.0);
            Mat4::from_cols_array_2d(&matrix.data.0 )
        };

        let ortho = matrix* aspect_matrix;
        grid.draw( &mut frame, ortho, context.clone(), & mut texture_cache  );

        frame.finish().unwrap();
    });
}

fn init_display( event_loop: &EventLoop<Call> ) -> glium::Display {
    let wb = WindowBuilder::new();
    let wb = wb.with_resizable(true);
    let wb = wb.with_movable_by_window_background(true);
    let wb = wb.with_inner_size(Size::Physical(PhysicalSize { width: 1920, height: 1080}));
    let wb = wb.with_title("A Dystopian Streaming Experience from an Alternate Reality");
    let cb = glium::glutin::ContextBuilder::new().with_depth_buffer(24);
    glium::Display::new(wb, cb, event_loop).unwrap()
}

pub enum Call {
    ToTexture{ bytes: Bytes, url: String },
    TextureCachingBatchComplete,
    AddSet(Set)
}