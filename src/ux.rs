use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use glam::{Affine3A, Mat4, Vec3, Vec4};
use glium::{Display, Frame, Surface};
use tokio::time::Instant;

use crate::data::{Item, Set};

pub struct Grid {
    pub vert_offset: Lerper,
    pub rows: Vec<Row>,
    pub selection: usize
}

impl Grid {

    pub fn new() -> Self {
        Self {
            vert_offset: Lerper::new(),
            rows: vec![],
            selection: 0
        }
    }

    pub fn add(&mut self, set: Set ) {
                let tiles : Vec<Tile> = set.items.iter().map( |item| {
                    Tile::new(item.clone())
                } ).collect();
                let row = Row::new( set, tiles );
        self.rows.push(row);
        self.select();
    }

    pub fn draw(&self, frame: &mut Frame, projection: Mat4, context: Arc<Renderers>, texture_cache: & mut HashMap<String,glium::texture::SrgbTexture2d> ) {
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
        if self.selection > 0 {
           self.unselect();
           self.selection = self.selection.clone()-1;
           self.vert_offset.next( self.offset() );
           self.select();
       }
    }

    pub fn down(&mut self) {
        if self.vert_offset.is_active() {
            return;
        }
        if self.selection < self.rows.len()-1 {
            self.unselect();
            self.selection = self.selection.clone()+1;
            self.vert_offset.next( self.offset() );
            self.select();
        }
    }

    pub fn left(&mut self) {
        let row = self.rows.get_mut(self.selection);
        match row {
            None => {}
            Some(row) => {
                row.left();
            }
        }
    }

    pub fn right(&mut self) {
        let row = self.rows.get_mut(self.selection);
        match row {
            None => {}
            Some(row) => {
                row.right();
            }
        }
    }

    pub fn offset(&self) -> Mat4 {
        Mat4::from_translation(Vec3::new(0.0, -(self.selection as f32*1.0), 0.0 ))
    }

    fn unselect(&mut self) {
        match self.find_selection() {
            None => {}
            Some(tile) => {
                tile.unselect();
            }
        }
    }

    fn select(&mut self) {
        match self.find_selection() {
            None => {}
            Some(tile) => {
                tile.select();
            }
        }
    }

    fn find_selection(&mut self) -> Option<&mut Tile> {
        match self.rows.get_mut(self.selection) {
            None => None,
            Some(row) => {
                row.tiles.get_mut(row.selection)
            }
        }
    }
}

pub struct Row {
  pub set: Set,
  pub tiles: Vec<Tile>,
  pub selection: usize,
  pub offset: Lerper,
}

impl Row {

    pub fn new(set: Set, tiles: Vec<Tile>) -> Self {
        Self {
            set,
            tiles,
            selection: 0,
            offset: Lerper::new()
        }
    }

    pub fn draw(&self, frame: &mut Frame, matrix: Mat4, context: Arc<Renderers>, texture_cache: & mut HashMap<String,glium::texture::SrgbTexture2d> ) {

        let mut matrix = matrix;

        // Yippie!  A Hard coded fix aspect value of 1.78.... I hope this never comes back to bite me!
        let tile_aspect_fix = Affine3A::from_scale(Vec3::new(1.78, 1.0, 1.0));
        matrix = matrix*tile_aspect_fix ;

        matrix = matrix * self.offset.lerp();

        let next= Affine3A::from_translation(Vec3::new(1.0, 0.0, 0.0 ));
        for tile in &self.tiles {
            tile.draw(frame,matrix,context.clone(),texture_cache);
            matrix = matrix*next;
        }
    }

    pub fn left( &mut self ) {

        if self.offset.is_active() {
            return;
        }

        if self.selection == 0 {
            return;
        }
        self.unselect();
        self.selection = self.selection.clone()-1;
        self.offset.next(self.calc_offset());
        self.select();
    }

    pub fn right( &mut self ) {

        if self.offset.is_active() {
            return;
        }

        if self.selection == self.tiles.len()-1 {
            return;
        }
        self.unselect();
        self.selection = self.selection.clone()+1;
        self.offset.next(self.calc_offset());
        self.select();
    }

    fn unselect(&mut self) {
        match self.find_selection() {
            None => {}
            Some(tile) => {
                tile.unselect();
            }
        }
    }

    fn select(&mut self) {
        match self.find_selection() {
            None => {}
            Some(tile) => {
                tile.select();
            }
        }
    }


    fn find_selection(&mut self) -> Option<&mut Tile> {
        self.tiles.get_mut(self.selection)
    }

    fn calc_offset(&self) -> Mat4 {
        Mat4::from_translation(Vec3::new(-(self.selection as f32*1.0), 0.0, 0.0 ))
    }

}

pub struct Tile {
  pub item: Item,
  pub selected: Lerper
}


impl Tile {
   const MARGIN: f32 = 0.15;

   pub fn new( item: Item ) -> Self {
       Self {
           item,
           selected: Lerper::new()
       }
   }

   pub fn select(&mut self) {
       let mut mat = Mat4::from_scale(Vec3::new(1.0+Self::MARGIN,1.0+Self::MARGIN, 1.0 ));
       let lift = Mat4::from_translation(Vec3::new(-Self::MARGIN/4.0,-Self::MARGIN/4.0,5.0));
       mat = lift*mat;
       self.selected.next(mat);
   }

    pub fn unselect(&mut self) {
        self.selected.next(Mat4::IDENTITY.clone() );
    }

    pub fn draw(&self, frame: &mut Frame, matrix: Mat4, context: Arc<Renderers>, texture_cache: & mut HashMap<String,glium::texture::SrgbTexture2d> ) {

       let margin = Affine3A::from_scale(Vec3::new(1.0-Self::MARGIN, 1.0-Self::MARGIN, 1.0 ));
       let matrix = matrix* margin;
       let offset = Affine3A::from_translation(Vec3::new( Self::MARGIN/2.0, Self::MARGIN/2.0, 0.0 ));
       let matrix = matrix* offset;

       let matrix = matrix*self.selected.lerp();

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
    }
}


pub fn lerp(a: &Mat4, b: &Mat4, value: f32 ) -> Mat4 {
    let value = clamp(value);
    a.clone()+((b.clone()-a.clone())*value)
}

pub fn clamp(v: f32 ) -> f32 {
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

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            .. Default::default()
        };

        frame.draw(&self.vertex_buffer, &self.indices, &self.program, &uniforms, &params).unwrap();
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
        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            .. Default::default()
        };
        frame.draw(&self.vertex_buffer, &self.indices, &self.program, &uniforms, &params).unwrap();
    }
}

pub struct Renderers {
    pub texture_tile_renderer: TileRenderer<TexturedVertex>,
    pub color_tile_renderer: TileRenderer<Vertex>,
}

impl Renderers {
    pub async fn new(texture_tile_renderer: TileRenderer<TexturedVertex>,color_tile_renderer: TileRenderer<Vertex>,)->Self {
        Self {
            texture_tile_renderer,
            color_tile_renderer
        }
    }
}
