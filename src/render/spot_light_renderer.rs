use std::ffi::{c_void, CStr};
use std::mem;
use std::path::PathBuf;
use std::ptr;

use crate::shader::{create_shader, OpenGLError};
use crate::state::Light;

use gl;
use gl::types::*;
use na::Matrix4;

pub struct SpotLightRenderer {
    shader_id: u32,
    vao: u32,
    color_uniform: i32,
    view_uniform: i32,
    projection_uniform: i32,
    transformation_uniform: i32,
    indices: Vec<u32>,
}

impl SpotLightRenderer {
    pub fn new(
        vert_shader: &PathBuf,
        frag_shader: &PathBuf,
        vertices: &[f32],
        indices: &[u32],
    ) -> Result<Self, OpenGLError> {
        unsafe {
            // Create Program
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
                let mut message: Vec<u8> = Vec::with_capacity(len as usize);
                message.set_len(len as usize - 1);
                gl::GetProgramInfoLog(
                    program,
                    len,
                    ptr::null_mut(),
                    message.as_mut_ptr() as *mut GLchar,
                );
                return Err(OpenGLError::FailedToLinkProgram(String::from_utf8(message)));
            }
            gl::DeleteShader(vert);
            gl::DeleteShader(frag);

            // Set up vertices and indices
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

            let stride = 3 * mem::size_of::<GLfloat>() as i32;
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
            gl::EnableVertexAttribArray(0);

            // Get Uniforms
            let projection_uniform = gl::GetUniformLocation(program, PROJECTION.as_ptr());
            let view_uniform = gl::GetUniformLocation(program, VIEW.as_ptr());
            let transformation_uniform = gl::GetUniformLocation(program, TRANSFORMATION.as_ptr());
            let color_uniform = gl::GetUniformLocation(program, COLOR.as_ptr());

            Ok(Self {
                shader_id: program,
                vao,
                color_uniform,
                view_uniform,
                projection_uniform,
                transformation_uniform,
                indices: Vec::from(indices),
            })
        }
    }

    pub fn draw(&self, lights: &[Light], view: Matrix4<f32>, projection: Matrix4<f32>) {
        unsafe {
            gl::UseProgram(self.shader_id);
            gl::BindVertexArray(self.vao);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.indices.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
                &self.indices[0] as *const _ as *const c_void,
                gl::STATIC_DRAW,
            );
            gl::UniformMatrix4fv(self.view_uniform, 1, gl::FALSE, view.as_ptr());
            gl::UniformMatrix4fv(self.projection_uniform, 1, gl::FALSE, projection.as_ptr());
            for light in lights {
                let transform = light.transform.to_matrix4();
                gl::UniformMatrix4fv(
                    self.transformation_uniform,
                    1,
                    gl::FALSE,
                    transform.as_ptr(),
                );

                let (r, g, b) = light.diffuse;
                gl::Uniform3f(self.color_uniform, r, g, b);

                gl::DrawElements(
                    gl::TRIANGLES,
                    self.indices.len() as i32,
                    gl::UNSIGNED_INT,
                    ptr::null(),
                );
            }
        }
    }
}

const TRANSFORMATION: &'static CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked(b"transformation\0") };
const PROJECTION: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"projection\0") };
const VIEW: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"view\0") };
const COLOR: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"color\0") };
