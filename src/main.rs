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
use crate::data::{Data, Item, Set};
use crate::cache::{cache_set, create_cacher};
use std::collections::HashMap;
use glam::{Mat4, Affine2, Vec2, Vec3, Affine3A, Vec4};
use glium::backend::glutin::glutin::event::VirtualKeyCode;
use tokio::time::{Instant, Duration};
use glium::backend::glutin::glutin::platform::macos::WindowBuilderExtMacOS;
use glium::backend::glutin::glutin::dpi::{Size, PhysicalSize};
use std::ops::Neg;
use nalgebra_glm::make_mat4x4;
use nalgebra::{Matrix4, MatrixSlice4};
use glium::{Frame, Surface, Display};
use glium::backend::glutin::glutin::event_loop::EventLoopProxy;
use glium::glutin::event_loop::EventLoop;
use glium::glutin::window::WindowBuilder;
use dashmap::DashMap;
use tokio::sync::mpsc;
use image::imageops::tile;

#[tokio::main]
async fn main() {

    let event_loop = glium::glutin::event_loop::EventLoop::<Call>::with_user_event();
    let display = init_display(&event_loop);
    let texture_tile_renderer = TileRenderer::<TexturedVertex>::new(&display);
    let color_tile_renderer = TileRenderer::<Vertex>::new(&display);

    let proxy = event_loop.create_proxy();

    let mut vert_lerper: Lerper = Lerper::new();

    let context = Arc::new(Context::new(texture_tile_renderer, color_tile_renderer).await );
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
/*                                    if grid.vert_offset.is_active() {
                                        return;
                                    }

                                    let nudge_up= Affine3A::from_translation(Vec3::new(0.0, vert_nudge, 0.0));
                                    grid.vert_offset.apply(nudge_up);

 */
                                    grid.up();
                                }
                                VirtualKeyCode::Down=> {
                                    /*
                                    if grid.vert_offset.is_active() {
                                        return;
                                    }

                                    let nudge_down= Affine3A::from_translation(Vec3::new(0.0, -vert_nudge, 0.0));
                                    grid.vert_offset.apply(nudge_down);
                                     */
                                    grid.down();
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
                        json::fetch(proxy).await.unwrap_or_default();
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
        frame.clear_color(
        0.129, 0.588, 0.953, 1.0 );


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

pub struct Selection {
    pub row: usize,
    pub col: usize
}

impl Selection {
    pub fn new()->Self {
        Self{
            row: 0,
            col: 0
        }
    }
    pub fn row_offset(&self) -> Mat4 {
        Mat4::from_translation(Vec3::new(0.0, -(self.row as f32*1.0), 0.0 ))
    }

    pub fn col_offset(&self) -> Mat4 {
        Mat4::from_translation(Vec3::new(0.0, self.col as f32*1.0, 0.0 ))
    }

    pub fn up(&mut self) {
        self.row = self.row - 1;
    }

    pub fn down(&mut self) {
        self.row = self.row + 1;
    }
}

pub struct Grid {
    pub vert_offset: Lerper,
    pub rows: Vec<Row>,
    pub selection: Selection
}

impl Grid {

    fn new() -> Self {
        Self {
            vert_offset: Lerper::new(),
            rows: vec![],
            selection: Selection::new()
        }
    }

    fn add(&mut self, set: Set ) {
                let tiles : Vec<Tile> = set.items.iter().map( |item| {
                    Tile{
                        item: item.clone()
                    }
                } ).collect();
                let row = Row {
                    set,
                    tiles,
                };
        self.rows.push(row);
    }

    pub fn draw(&self, frame: &mut Frame, projection: Mat4, context: Arc<Context>, texture_cache: & mut HashMap<String,glium::texture::SrgbTexture2d> ) {
        let mut matrix = projection * self.vert_offset.lerp();
        for row in &self.rows {
            row.draw( frame, matrix, context.clone(), texture_cache );
            let next= Affine3A::from_translation(Vec3::new(0.0, 1.0, 0.0 ));
            matrix = matrix * next;
        }
    }

    pub fn up(&mut self) {
        if self.vert_offset.is_active() {
            return;
        }
        if self.selection.row > 0 {
           self.selection.up();
           self.vert_offset.next( self.selection.row_offset() );
       }
    }

    pub fn down(&mut self) {
        if self.vert_offset.is_active() {
            return;
        }
        if self.selection.row < self.rows.len() {
            self.selection.down();
            self.vert_offset.next( self.selection.row_offset() );
        }
    }

}

pub struct Row {
  pub set: Set,
  pub tiles: Vec<Tile>,
}

impl Row {
    pub fn draw(&self, frame: &mut Frame, matrix: Mat4, context: Arc<Context>, texture_cache: & mut HashMap<String,glium::texture::SrgbTexture2d> ) {

        let mut matrix = matrix;

        // Yippie!  A Hard coded fix aspect value of 1.78.... I hope this never comes back to bite me!
        let tile_aspect_fix = Affine3A::from_scale(Vec3::new(1.78, 1.0, 1.0));
        matrix = matrix*tile_aspect_fix ;

        let next= Affine3A::from_translation(Vec3::new(1.0, 0.0, 0.0 ));
        for tile in &self.tiles {
            tile.draw(frame,matrix,context.clone(),texture_cache);
            matrix = matrix*next;
        }
    }
}


fn init_display( event_loop: &EventLoop<Call> ) -> glium::Display {
    let wb = WindowBuilder::new();
    let wb = wb.with_resizable(true);
    let wb = wb.with_movable_by_window_background(true);
    let wb = wb.with_inner_size(Size::Physical(PhysicalSize { width: 1920, height: 1080}));
    let wb = wb.with_title("A Dystopian Streaming Experience from an Alternate Reality");
    let cb = glium::glutin::ContextBuilder::new();
    glium::Display::new(wb, cb, event_loop).unwrap()
}




pub enum Call {
    ToTexture{ bytes: Bytes, url: String },
    TextureCachingBatchComplete,
    AddSet(Set)
}

pub struct Tile {
  pub item: Item
}

impl Tile {
   pub fn draw(&self, frame: &mut Frame, matrix: Mat4, context: Arc<Context>, texture_cache: & mut HashMap<String,glium::texture::SrgbTexture2d> ) {

       let margin = Affine3A::from_scale(Vec3::new(0.9, 0.9, 1.0 ));
       let matrix = matrix* margin;
       let offset = Affine3A::from_translation(Vec3::new( 0.05, 0.05, 0.0 ));
       let matrix = matrix* offset;

       match texture_cache.get(&self.item.image_url.clone() ) {
           Some(_) => {
               // we remove the texture, use it then insert it back in the texture_cache...
               // I wrestled with grabbing a reference, however, gave up the battle with the borrow checker
               // and determined to instead to utilize this 'hack' ... at least for now
               let texture = texture_cache.remove(&self.item.image_url ).unwrap();
               context.texture_tile_renderer.draw( frame, matrix, &texture );
               texture_cache.insert(self.item.image_url.clone(), texture );
           }
           None => {
               context.color_tile_renderer.draw( frame, matrix, Vec4::from( ( 1.0,1.0,1.0,0.75)));
           }
       }
        /*        target.draw(&self.vertex_buffer, &self.indices, &self.program, &uniforms,
                            &Default::default()).unwrap();

         */

    }
}


pub fn lerp( a: &Mat4, b: &Mat4, value: f32 ) -> Mat4 {
    let value = clamp(value);
    a.clone()+((b.clone()-a.clone())*value)
}

pub fn clamp( v: f32 ) -> f32 {
    if v < 0.0 {
        return 0.0;
    } else if v > 1.0 {
        return 1.0;
    } else {
        return v;
    }
}

pub struct Lerper {
  pub begin: Mat4,
  pub end: Mat4,
  pub start_time: Instant,
  pub duration: Duration
}

impl Lerper {
    pub fn new() -> Self {
        Self{
            begin: Mat4::IDENTITY.clone(),
            end: Mat4::IDENTITY.clone(),
            start_time: Instant::now(),
            duration: Duration::from_millis(200 )
        }
    }

    // make this the next location we will lerp to and start the timer
    pub fn next( &mut self, end: Mat4 ) {
        self.begin = self.end.clone();
        self.end = end;
        self.start_time = Instant::now();
    }

    pub fn apply( &mut self, xform: Affine3A ) {
        self.begin = self.end.clone();
        self.end = self.end * xform;
        self.start_time = Instant::now();
    }

    pub fn set( &mut self, begin: Mat4, end: Mat4 ) {
        self.begin = begin;
        self.end = end;
        self.start_time = Instant::now();
    }

    pub fn unset( &mut self ) {
        self.begin = Mat4::IDENTITY.clone();
        self.end = Mat4::IDENTITY.clone();
    }

    pub fn lerp(&self) -> Mat4 {
        let elapsed = self.start_time.elapsed();
        let v= (elapsed.as_millis() as f32/self.duration.as_millis() as f32) as f32;
        lerp( &self.begin, &self.end, v)
    }

    pub fn is_active(&self) -> bool {
        !self.is_done()
    }

    pub fn is_done(&self) -> bool {
        return  self.start_time+self.duration < Instant::now()
    }
}


#[derive(Copy, Clone)]
pub struct TexturedVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

#[derive(Copy, Clone)]
pub struct Vertex{
    position: [f32; 2]
}

implement_vertex!(TexturedVertex, position, tex_coords);
implement_vertex!(Vertex, position);

pub struct TileRenderer<T:Copy> {
   pub vertex_buffer: glium::VertexBuffer<T>,
   pub program: glium::Program,
   pub indices: glium::index::NoIndices,
}

impl TileRenderer<Vertex> {

    pub fn new(display: &Display)->Self {
        let vertex1 = Vertex { position: [0.0, 0.0] };
        let vertex2 = Vertex { position: [ 0.0,  1.0] };
        let vertex3 = Vertex { position: [ 1.0, 0.0] };
        let vertex4 = Vertex { position: [ 1.0, 1.0] };
        let vertex5 = Vertex { position: [ 0.0,  1.0] };
        let vertex6 = Vertex { position: [ 1.0, 0.0] };

        let shape = vec![vertex1, vertex2, vertex3, vertex4, vertex5, vertex6 ];

        let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        let vertex_shader_src = r#"
        #version 140
        in vec2 position;
        uniform mat4 matrix;
        void main() {
            gl_Position = matrix * vec4(position, 0.0, 1.0);
        }
    "#;

        let fragment_shader_src = r#"
        #version 140
        out vec4 color;
        uniform vec4 color_in;
        void main() {
            color = color_in;
        }
    "#;

        let program = glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap();

        Self {
            indices,
            program,
            vertex_buffer
        }
    }

    pub fn draw(&self, frame: &mut Frame, matrix: Mat4, color: Vec4 ) {
        let uniforms = uniform! {
            matrix: [
                [ matrix.x_axis.x , matrix.x_axis.y, matrix.x_axis.z, matrix.x_axis.w],
                [ matrix.y_axis.x , matrix.y_axis.y, matrix.y_axis.z, matrix.y_axis.w],
                [ matrix.z_axis.x , matrix.z_axis.y, matrix.z_axis.z, matrix.z_axis.w],
                [ matrix.w_axis.x , matrix.w_axis.y, matrix.w_axis.z, matrix.w_axis.w],
            ],
            color_in: [color.x,color.y,color.z,color.w]
        };
        frame.draw(&self.vertex_buffer, &self.indices, &self.program, &uniforms, &Default::default()).unwrap();
    }
}

impl TileRenderer<TexturedVertex> {

    pub fn new(display: &Display)->Self {
        let vertex1 = TexturedVertex { position: [0.0, 0.0], tex_coords: [0.0, 1.0] };
        let vertex2 = TexturedVertex { position: [ 0.0,  1.0], tex_coords: [0.0, 0.0] };
        let vertex3 = TexturedVertex { position: [ 1.0, 0.0], tex_coords: [1.0, 1.0] };
        let vertex4 = TexturedVertex { position: [ 1.0, 1.0], tex_coords: [1.0, 0.0] };
        let vertex5 = TexturedVertex { position: [ 0.0,  1.0], tex_coords: [0.0, 0.0] };
        let vertex6 = TexturedVertex { position: [ 1.0, 0.0], tex_coords: [1.0, 1.0] };

        let shape = vec![vertex1, vertex2, vertex3, vertex4, vertex5, vertex6 ];

        let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
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

        let program = glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap();

        Self {
            indices,
            program,
            vertex_buffer
        }
    }


    pub fn draw(&self, frame: &mut Frame, matrix: Mat4, texture: &glium::texture::SrgbTexture2d) {
        let uniforms = uniform! {
            matrix: [

                [ matrix.x_axis.x , matrix.x_axis.y, matrix.x_axis.z, matrix.x_axis.w],
                [ matrix.y_axis.x , matrix.y_axis.y, matrix.y_axis.z, matrix.y_axis.w],
                [ matrix.z_axis.x , matrix.z_axis.y, matrix.z_axis.z, matrix.z_axis.w],
                [ matrix.w_axis.x , matrix.w_axis.y, matrix.w_axis.z, matrix.w_axis.w],
            ],
            tex: texture,
        };
        frame.draw(&self.vertex_buffer, &self.indices, &self.program, &uniforms, &Default::default()).unwrap();
    }
}

pub struct Context {
    pub texture_tile_renderer: TileRenderer<TexturedVertex>,
    pub color_tile_renderer: TileRenderer<Vertex>,
}

impl Context {
    pub async fn new(texture_tile_renderer: TileRenderer<TexturedVertex>,color_tile_renderer: TileRenderer<Vertex>,)->Self {
        Self {
            texture_tile_renderer,
            color_tile_renderer
        }
    }
}
