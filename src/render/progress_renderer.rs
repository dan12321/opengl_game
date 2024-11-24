use std::ffi::{c_void, CStr};
use std::mem;
use std::path::PathBuf;
use std::ptr;

use crate::shader::{create_shader, OpenGLError};
use crate::state::scenes::ProgressBar;

use gl;
use gl::types::*;

pub struct ProgressRenderer {
    shader_id: u32,
    vao: u32,
    base_color_uniform: i32,
    progress_color_uniform: i32,
    progress_uniform: i32,
    transformation_uniform: i32,
    indices: Vec<u32>,
}

impl ProgressRenderer {
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
                return Err(OpenGLError::link_program_error(&message));
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
            let progress_color_uniform = gl::GetUniformLocation(program, PROGRESS_COLOR.as_ptr());
            let base_color_uniform = gl::GetUniformLocation(program, BASE_COLOR.as_ptr());
            let progress_uniform = gl::GetUniformLocation(program, PROGRESS.as_ptr());
            let transformation_uniform = gl::GetUniformLocation(program, TRANSFORMATION.as_ptr());

            Ok(Self {
                shader_id: program,
                vao,
                base_color_uniform,
                progress_color_uniform,
                progress_uniform,
                transformation_uniform,
                indices: Vec::from(indices),
            })
        }
    }

    pub fn draw(&self, bar: ProgressBar) {
        unsafe {
            gl::UseProgram(self.shader_id);
            gl::BindVertexArray(self.vao);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.indices.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
                &self.indices[0] as *const _ as *const c_void,
                gl::STATIC_DRAW,
            );
            let transform = bar.transform.to_matrix4();
            gl::UniformMatrix4fv(
                self.transformation_uniform,
                1,
                gl::FALSE,
                transform.as_ptr(),
            );

            let (r, g, b) = bar.base_color;
            gl::Uniform3f(self.base_color_uniform, r, g, b);
            let (r, g, b) = bar.progress_color;
            gl::Uniform3f(self.progress_color_uniform, r, g, b);
            gl::Uniform1f(self.progress_uniform, bar.progress);

            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len() as i32,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }
    }
}

const TRANSFORMATION: &'static CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked(b"transformation\0") };
const PROGRESS: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"progress\0") };
const PROGRESS_COLOR: &'static CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked(b"progress_color\0") };
const BASE_COLOR: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"base_color\0") };
