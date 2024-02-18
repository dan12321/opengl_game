use std::{
    ffi::{c_void, CStr, CString},
    mem,
    path::PathBuf,
    ptr,
};

use gl::types::*;
use image::DynamicImage;
use na::Matrix4;

use crate::{
    light::{self, template_light, LightUniform},
    shader::{create_shader, OpenGLError},
    state::{Cube, Transform, XYZ},
};

pub struct CubeRenderer {
    shader_id: u32,
    vao: u32,
    texture: u32,
    view_uniform: i32,
    projection_uniform: i32,
    camera_position_uniform: i32,
    transformation_uniform: i32,
    material_uniform: MaterialUniform,
    light_uniform: [SpotLightUniform; MAX_LIGHTS],
    indices: Vec<u32>,
}

#[derive(Copy, Clone, Debug)]
struct SpotLightUniform {
    position: i32,
    diffuse: i32,
    specular: i32,
    strength: i32,
}

struct MaterialUniform {
    ambient: i32,
    diffuse: i32,
    specular: i32,
    shininess: i32,
}

impl CubeRenderer {
    pub fn new(
        vert_shader: &PathBuf,
        frag_shader: &PathBuf,
        vertices: &[f32],
        indices: &[u32],
        texture: DynamicImage,
    ) -> Result<Self, OpenGLError> {
        unsafe {
            // Create program
            let vert = create_shader(vert_shader, gl::VERTEX_SHADER)?;
            let frag = create_shader(frag_shader, gl::FRAGMENT_SHADER)?;

            let program = gl::CreateProgram();
            gl::AttachShader(program, vert);
            gl::AttachShader(program, frag);
            gl::LinkProgram(program);
            let mut success = gl::FALSE as GLsizei;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success == (gl::FALSE as GLsizei) {
                let mut len = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut error_buffer = Vec::with_capacity(len as usize);
                error_buffer.set_len(len as usize - 1);
                gl::GetProgramInfoLog(
                    program,
                    len,
                    ptr::null_mut(),
                    error_buffer.as_mut_ptr() as *mut GLchar,
                );
                return Err(OpenGLError::FailedToLinkProgram(String::from_utf8(
                    error_buffer,
                )));
            }
            gl::DeleteShader(vert);
            gl::DeleteShader(frag);

            // Set up vertices and indeces
            let mut vao = 0;
            let mut vbo = 0;
            let mut ebo = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                &vertices[0] as *const _ as *const c_void,
                gl::STATIC_DRAW,
            );

            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
                &indices[0] as *const _ as *const c_void,
                gl::STATIC_DRAW,
            );
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            let stride = 8 * mem::size_of::<GLfloat>() as i32;
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
            gl::EnableVertexAttribArray(0);
            let normal_start = 3 * mem::size_of::<GLfloat>() as i32;
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                stride,
                normal_start as *mut c_void,
            );
            gl::EnableVertexAttribArray(1);
            let tex_start = normal_start + (3 * mem::size_of::<GLfloat>() as i32);
            gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, stride, tex_start as *mut c_void);
            gl::EnableVertexAttribArray(2);

            // texture
            let mut texture_location = 0;
            let texture_bytes = texture.as_bytes();
            gl::GenTextures(1, &mut texture_location);
            gl::BindTexture(gl::TEXTURE_2D, texture_location);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                texture.width() as i32,
                texture.height() as i32,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                &texture_bytes[0] as *const _ as *const c_void,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);

            // Get uniforms
            let material = MaterialUniform {
                ambient: gl::GetUniformLocation(program, AMBIENT.as_ptr()),
                diffuse: gl::GetUniformLocation(program, DIFFUSE.as_ptr()),
                specular: gl::GetUniformLocation(program, SPECULAR.as_ptr()),
                shininess: gl::GetUniformLocation(program, SHININESS.as_ptr()),
            };
            let camera_position_uniform = gl::GetUniformLocation(program, CAMERA_POSITION.as_ptr());
            let projection_uniform = gl::GetUniformLocation(program, PROJECTION.as_ptr());
            let view_uniform = gl::GetUniformLocation(program, VIEW.as_ptr());
            let transformation_uniform = gl::GetUniformLocation(program, TRANSFORMATION.as_ptr());

            let mut lights = [SpotLightUniform {
                position: -1,
                diffuse: -1,
                specular: -1,
                strength: -1,
            }; MAX_LIGHTS];
            for i in 0..MAX_LIGHTS {
                let pos = template_light(i, light::Prop::Position);
                let dif = template_light(i, light::Prop::Diffuse);
                let spec = template_light(i, light::Prop::Specular);
                let stren = template_light(i, light::Prop::Strength);
                let position = CString::new(pos.as_bytes()).unwrap();
                let diffuse = CString::new(dif.as_bytes()).unwrap();
                let specular = CString::new(spec.as_bytes()).unwrap();
                let strength = CString::new(stren.as_bytes()).unwrap();
                lights[i] = SpotLightUniform {
                    position: gl::GetUniformLocation(program, position.as_ptr()),
                    diffuse: gl::GetUniformLocation(program, diffuse.as_ptr()),
                    specular: gl::GetUniformLocation(program, specular.as_ptr()),
                    strength: gl::GetUniformLocation(program, strength.as_ptr()),
                };
            }
            Ok(Self {
                shader_id: program,
                vao,
                texture: texture_location,
                indices: indices.to_vec(),
                material_uniform: material,
                camera_position_uniform,
                projection_uniform,
                view_uniform,
                transformation_uniform,
                light_uniform: lights,
            })
        }
    }

    pub fn draw(
        &self,
        cubes: &[Cube],
        lights: &[LightUniform],
        camera_position: &XYZ,
        view: Matrix4<f32>,
        projection: Matrix4<f32>,
    ) {
        unsafe {
            gl::UseProgram(self.shader_id);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::BindVertexArray(self.vao);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.indices.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
                &self.indices[0] as *const _ as *const c_void,
                gl::STATIC_DRAW,
            );
            for i in 0..lights.len() {
                gl::Uniform3f(
                    self.light_uniform[i].position,
                    lights[i].position.0,
                    lights[i].position.1,
                    lights[i].position.2,
                );
                gl::Uniform3f(
                    self.light_uniform[i].diffuse,
                    lights[i].diffuse.0,
                    lights[i].diffuse.1,
                    lights[i].diffuse.2,
                );
                gl::Uniform3f(
                    self.light_uniform[i].specular,
                    lights[i].specular.0,
                    lights[i].specular.1,
                    lights[i].specular.2,
                );
                gl::Uniform1f(self.light_uniform[i].strength, lights[i].strength);
            }
            gl::UniformMatrix4fv(self.view_uniform, 1, gl::FALSE, view.as_ptr());
            gl::UniformMatrix4fv(self.projection_uniform, 1, gl::FALSE, projection.as_ptr());
            gl::Uniform3f(
                self.camera_position_uniform,
                camera_position.x,
                camera_position.y,
                camera_position.z,
            );

            for cube in cubes {
                let transform = cube.transform.to_matrix4();
                gl::UniformMatrix4fv(
                    self.transformation_uniform,
                    1,
                    gl::FALSE,
                    transform.as_ptr(),
                );
                gl::Uniform3f(
                    self.material_uniform.ambient,
                    cube.material.ambient.0,
                    cube.material.ambient.1,
                    cube.material.ambient.2,
                );
                gl::Uniform3f(
                    self.material_uniform.diffuse,
                    cube.material.diffuse.0,
                    cube.material.diffuse.1,
                    cube.material.diffuse.2,
                );
                gl::Uniform3f(
                    self.material_uniform.specular,
                    cube.material.specular.0,
                    cube.material.specular.1,
                    cube.material.specular.2,
                );
                gl::Uniform1i(self.material_uniform.shininess, cube.material.shininess);
                gl::DrawElements(
                    gl::TRIANGLES,
                    self.indices.len() as i32,
                    gl::UNSIGNED_INT,
                    ptr::null(),
                );
            }
            gl::BindVertexArray(0);
        }
    }
}

const TRANSFORMATION: &'static CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked(b"transformation\0") };
const PROJECTION: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"projection\0") };
const VIEW: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"view\0") };
const CAMERA_POSITION: &'static CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked(b"cameraPosition\0") };
const AMBIENT: &'static CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked(b"material.ambient\0") };
const DIFFUSE: &'static CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked(b"material.diffuse\0") };
const SPECULAR: &'static CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked(b"material.specular\0") };
const SHININESS: &'static CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked(b"material.shininess\0") };
const MAX_LIGHTS: usize = 64;
