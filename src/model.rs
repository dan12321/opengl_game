use super::shader::Shader;

use std::{
    ffi::{c_void, NulError},
    mem, ptr,
};

use gl::types::*;
use na::{Matrix4, Translation3};
use tracing::debug;

pub struct Model<const R: usize, const S: usize> {
    pub vertices: [GLfloat; R],
    pub indices: [GLuint; S],
    pub transform: Translation3<GLfloat>,
    pub rotation: Matrix4<GLfloat>,
    pub scale: GLfloat,
    pub shader: Shader,
    vao: u32,
    vbo: u32,
    ebo: u32,
    material: Option<Material>,
    textures: [Option<u32>; 32],
}

impl<const R: usize, const S: usize> Model<R, S> {
    pub fn world_space_operation(&self) -> Matrix4<GLfloat> {
        self.transform.to_homogeneous()
            * self.rotation
            * na::Scale3::new(self.scale, self.scale, self.scale).to_homogeneous()
    }

    pub fn draw(&self) {
        unsafe {
            self.shader.use_program();
            for i in 0..self.textures.len() {
                match self.textures[i] {
                    Some(t) => {
                        gl::ActiveTexture(GL_TEXTURES[i]);
                        gl::BindTexture(gl::TEXTURE_2D, t);
                    }
                    None => break,
                }
            }
            gl::BindVertexArray(self.vao);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.indices.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
                mem::transmute(&self.indices),
                gl::STATIC_DRAW,
            );
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len() as i32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
            gl::BindVertexArray(0);
        }
    }

    pub fn set_material(mut self, material: Material) -> Result<Self, NulError> {
        let first_material = self.material.is_none();
        if first_material {
            self.shader = self.shader
                .add_uniform3f(AMBIENT_UNIFORM, material.ambient)?
                .add_uniform3f(DIFFUSE_UNIFORM, material.diffuse)?
                .add_uniform3f(SPECULAR_UNIFORM, material.specular)?
                .add_uniform1i(SHININESS_UNIFORM, material.shininess)?;
        } else {
            // TODO: Error if failed
            self.shader.set_uniform3f(AMBIENT_UNIFORM, material.ambient);
            self.shader.set_uniform3f(DIFFUSE_UNIFORM, material.diffuse);
            self.shader.set_uniform3f(SPECULAR_UNIFORM, material.specular);
            self.shader.set_uniform1i(SHININESS_UNIFORM, material.shininess);
        }
        self.material = Some(material);
        Ok(self)
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

    pub fn add_uniform3f(mut self, name: &str, value: (f32, f32, f32)) -> Result<Self, NulError> {
        self.shader = self.shader.add_uniform3f(name, value)?;
        Ok(self)
    }

    pub fn get_uniform3f(&self, name: &str) -> Option<(f32, f32, f32)> {
        self.shader.get_uniform3f(name)
    }

    /// Sets the value of the uniform in the shader
    /// Returns the old value or none if the uniform doesn't exist
    pub fn set_uniform3f(&mut self, name: &str, value: (f32, f32, f32)) -> Option<(f32, f32, f32)> {
        self.shader.set_uniform3f(name, value)
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
    rotation: Option<Matrix4<GLfloat>>,
    scale: GLfloat,
    textures: [Option<String>; 32],
    textures_count: usize,
    normals: bool,
}

impl<const R: usize, const S: usize> ModelBuilder<R, S> {
    pub fn new(vertices: [GLfloat; R], indices: [GLuint; S], shader: Shader) -> Self {
        Self {
            vertices,
            indices,
            shader,
            transform: None,
            rotation: None,
            scale: 1.0,
            textures: Default::default(),
            textures_count: 0,
            normals: false,
        }
    }

    pub fn add_transform(mut self, transform: Translation3<GLfloat>) -> Self {
        self.transform = Some(transform);
        self
    }

    pub fn set_rotation(mut self, rotation: Matrix4<GLfloat>) -> Self {
        self.rotation = Some(rotation);
        self
    }

    pub fn set_scale(mut self, scale: GLfloat) -> Self {
        self.scale = scale;
        self
    }

    pub fn add_texture(mut self, texture: String) -> Self {
        if self.textures_count < 32 {
            self.textures[self.textures_count] = Some(texture);
            self.textures_count += 1;
        }
        self
    }

    pub fn add_normals(mut self) -> Self {
        self.normals = true;
        self
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
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                mem::transmute(&self.vertices),
                gl::STATIC_DRAW,
            );
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.indices.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
                mem::transmute(&self.indices),
                gl::STATIC_DRAW,
            );
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);

            let mut stride = 3 * mem::size_of::<GLfloat>() as i32;
            if self.normals {
                stride += 3 * mem::size_of::<GLfloat>() as i32;
            }
            if self.textures_count > 0 {
                stride += 2 * mem::size_of::<GLfloat>() as i32;
            }
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
            gl::EnableVertexAttribArray(0);
            let mut tex_start = 3;
            let mut tex_index = 1;
            if self.normals {
                gl::VertexAttribPointer(
                    1,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    stride,
                    (3 * mem::size_of::<GLfloat>()) as *mut c_void,
                );
                gl::EnableVertexAttribArray(1);
                tex_start += 3;
                tex_index += 1;
            }
            for i in 0..self.textures_count {
                let first_position = (tex_start + 2 * i) * mem::size_of::<GLfloat>();
                debug!(first_position = first_position, "added texture");
                gl::VertexAttribPointer(
                    (i + tex_index) as u32,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    stride,
                    first_position as *mut c_void,
                );
                gl::EnableVertexAttribArray((i + tex_index) as u32);
            }
        }

        let mut textures: [Option<u32>; 32] = Default::default();
        for i in 0..self.textures.len() {
            match &self.textures[i] {
                Some(t) => {
                    let mut texture = 0;
                    let texture_image = image::open(t).unwrap();
                    // Uses native endian. Not sure if this always matches what opengl expects
                    let texture_bytes = texture_image.as_bytes();

                    unsafe {
                        gl::GenTextures(1, &mut texture);
                        gl::BindTexture(gl::TEXTURE_2D, texture);
                        gl::TexImage2D(
                            gl::TEXTURE_2D,
                            0,
                            gl::RGB as i32,
                            texture_image.width() as i32,
                            texture_image.height() as i32,
                            0,
                            gl::RGB,
                            gl::UNSIGNED_BYTE,
                            &texture_bytes[0] as *const _ as *const c_void,
                        );
                        gl::GenerateMipmap(gl::TEXTURE_2D);
                    }

                    textures[i] = Some(texture);
                }
                None => break,
            }
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
            scale: self.scale,
            shader: self.shader,
            vao,
            vbo,
            ebo,
            textures,
            material: None,
        }
    }
}

const GL_TEXTURES: [GLenum; 32] = [
    gl::TEXTURE0,
    gl::TEXTURE1,
    gl::TEXTURE2,
    gl::TEXTURE3,
    gl::TEXTURE4,
    gl::TEXTURE5,
    gl::TEXTURE6,
    gl::TEXTURE7,
    gl::TEXTURE8,
    gl::TEXTURE9,
    gl::TEXTURE10,
    gl::TEXTURE11,
    gl::TEXTURE12,
    gl::TEXTURE13,
    gl::TEXTURE14,
    gl::TEXTURE15,
    gl::TEXTURE16,
    gl::TEXTURE17,
    gl::TEXTURE18,
    gl::TEXTURE19,
    gl::TEXTURE20,
    gl::TEXTURE21,
    gl::TEXTURE22,
    gl::TEXTURE23,
    gl::TEXTURE24,
    gl::TEXTURE25,
    gl::TEXTURE26,
    gl::TEXTURE27,
    gl::TEXTURE28,
    gl::TEXTURE29,
    gl::TEXTURE30,
    gl::TEXTURE31,
];

pub struct Material {
    ambient: (f32, f32, f32),
    diffuse: (f32, f32, f32),
    specular: (f32, f32, f32),
    shininess: GLint,
}

impl Material {
    pub fn new(
        ambient: (f32, f32, f32),
        diffuse: (f32, f32, f32),
        specular: (f32, f32, f32),
        shininess: i32,
    ) -> Self {
        Material {
            ambient,
            diffuse,
            specular,
            shininess,
        }
    }
}

static AMBIENT_UNIFORM: &'static str = "material.ambient";
static DIFFUSE_UNIFORM: &'static str = "material.diffuse";
static SPECULAR_UNIFORM: &'static str = "material.specular";
static SHININESS_UNIFORM: &'static str = "material.shininess";

