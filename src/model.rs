use super::shader::Shader;

use std::{
    mem,
    ptr, ffi::NulError,
};

use gl::types::*;
use na::{matrix, Matrix4, vector, Vector4, Translation3};

pub struct Model<const R: usize, const S: usize> {
    pub vertices: [GLfloat; R],
    pub indices: [GLuint; S],
    pub transform: Translation3<GLfloat>,
    pub rotation: Matrix4<GLfloat>,
    pub shader: Shader,
    vao: u32,
    vbo: u32,
    ebo: u32,
}

impl<const R: usize, const S: usize> Model<R, S> {
    pub fn world_space_operation(&self) -> Matrix4<GLfloat> {
        self.transform.to_homogeneous() * self.rotation
    }

    pub fn draw(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (self.indices.len() * mem::size_of::<GLuint>()) as GLsizeiptr, mem::transmute(&self.indices), gl::STATIC_DRAW);
            gl::DrawElements(gl::TRIANGLES, self.indices.len() as i32, gl::UNSIGNED_INT, ptr::null());
            gl::BindVertexArray(0);
        }
    }

    pub fn add_uniform1f(mut self, name: &str, value: f32) -> Result<Self, NulError> {
        self.shader = self.shader.add_uniform1f(name, value)?;
        Ok(self)
    }

    pub fn get_uniform1f(&self, name: &str) -> Option<f32> {
        self.shader.get_uniform1f(name)
    }

    /// Sets the value of the uniform in the shader
    /// Returns the old value or none if the uniform doesn't exist
    pub fn set_uniform1f(&mut self, name: &str, value: f32) -> Option<f32> {
        self.shader.set_uniform1f(name, value)
    }

    pub fn add_uniform2f(mut self, name: &str, value: (f32, f32)) -> Result<Self, NulError> {
        self.shader = self.shader.add_uniform2f(name, value)?;
        Ok(self)
    }

    pub fn get_uniform2f(&self, name: &str) -> Option<(f32, f32)> {
        self.shader.get_uniform2f(name)
    }

    /// Sets the value of the uniform in the shader
    /// Returns the old value or none if the uniform doesn't exist
    pub fn set_uniform2f(&mut self, name: &str, value: (f32, f32)) -> Option<(f32, f32)> {
        self.shader.set_uniform2f(name, value)
    }

    pub fn add_uniform1i(mut self, name: &str, value: i32) -> Result<Self, NulError> {
        self.shader = self.shader.add_uniform1i(name, value)?;
        Ok(self)
    }

    pub fn get_uniform1i(&self, name: &str) -> Option<i32> {
        self.shader.get_uniform1i(name)
    }

    /// Sets the value of the uniform in the shader
    /// Returns the old value or none if the uniform doesn't exist
    pub fn set_uniform1i(&mut self, name: &str, value: i32) -> Option<i32> {
        self.shader.set_uniform1i(name, value)
    }

    pub fn add_uniform_mat4(mut self, name: &str, value: Matrix4<f32>) -> Result<Self, NulError> {
        self.shader = self.shader.add_uniform_mat4(name, value)?;
        Ok(self)
    }

    pub fn get_uniform_mat4(&self, name: &str) -> Option<Matrix4<f32>> {
        self.shader.get_uniform_mat4(name)
    }

    /// Sets the value of the uniform in the shader
    /// Returns the old value or none if the uniform doesn't exist
    pub fn set_uniform_mat4(&mut self, name: &str, value: Matrix4<f32>) -> Option<Matrix4<f32>> {
        self.shader.set_uniform_mat4(name, value)
    }
}

pub struct ModelBuilder<const R: usize, const S: usize> {
    vertices: [GLfloat; R],
    indices: [GLuint; S],
    shader: Shader,
    transform: Option<Translation3<GLfloat>>,
    rotation: Option<Matrix4<GLfloat>>
}

impl<const R: usize, const S: usize> ModelBuilder<R, S> {
    pub fn new(vertices: [GLfloat; R], indices: [GLuint; S], shader: Shader) -> Self {
        Self {
            vertices,
            indices,
            shader,
            transform: None,
            rotation: None,
        }
    }

    pub fn add_transform(self, transform: Translation3<GLfloat>) -> Self {
        Self {
            vertices: self.vertices,
            indices: self.indices,
            shader: self.shader,
            transform: Some(transform),
            rotation: self.rotation,
        }
    }

    pub fn add_rotation(self, rotation: Matrix4<GLfloat>) -> Self {
        Self {
            vertices: self.vertices,
            indices: self.indices,
            shader: self.shader,
            transform: self.transform,
            rotation: Some(rotation),
        }
    }

    pub fn init(self) -> Model<R, S> {
        let mut vao = 0;
        let mut vbo = 0;
        let mut ebo = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (self.vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr, mem::transmute(&self.vertices), gl::STATIC_DRAW);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (self.indices.len() * mem::size_of::<GLuint>()) as GLsizeiptr, mem::transmute(&self.indices), gl::STATIC_DRAW);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
    
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (3 * mem::size_of::<GLfloat>()) as i32, ptr::null());
            gl::EnableVertexAttribArray(0);
        }

        Model {
            vertices: self.vertices,
            indices: self.indices,
            transform: match self.transform {
                Some(t) => t,
                None => Translation3::new(0.0, 0.0, 0.0),
            },
            rotation: match self.rotation {
                Some(r) => r,
                None => Matrix4::identity(),                
            },
            shader: self.shader,
            vao,
            vbo,
            ebo
        }
    }
}

pub const VERTICESA: [GLfloat; 24] = [
    0.5,  0.5, 0.5, // top right forward
    0.5, -0.5, 0.5,  // bottom right forward
   -0.5,  0.5, 0.5,   // top left forward
   -0.5,  -0.5, 0.5,   // bottom left forward
    0.5,  0.5, -0.5, // top right back
    0.5, -0.5, -0.5,  // bottom right back
   -0.5,  0.5, -0.5,   // top left back
   -0.5,  -0.5, -0.5,   // bottom left back
];

pub const INDICESA: [GLuint; 36] = [
    0, 1, 2, // front
    2, 1, 3, // front
    0, 2, 4, // top
    4, 2, 6, // top
    0, 1, 4, // right
    4, 1, 5, // right
    1, 3, 5, // bottom
    5, 3, 7, // bottom
    2, 3, 6, // left
    6, 3, 7, // left
    4, 5, 6, // back
    6, 5, 7, // back
];