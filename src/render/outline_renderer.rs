use std::{
    ffi::{c_void, CStr},
    mem,
    path::PathBuf,
    ptr,
};

use gl::types::*;
use na::Matrix4;

use crate::{
    shader::{create_shader, OpenGLError},
    state::Model,
};

use super::model_loader::Mesh;

pub struct OutlineRenderer {
    shader_id: u32,
    parsed_meshes: Vec<ParsedModel>,
    view_uniform: i32,
    projection_uniform: i32,
    transformation_uniform: i32,
}

impl OutlineRenderer {
    pub fn new(
        vert_shader: &PathBuf,
        frag_shader: &PathBuf,
        meshes: &Vec<Mesh>,
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

            let mut parsed_models = Vec::with_capacity(meshes.len());
            // Set up vertices and indices
            for meshes in meshes {
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
                    (meshes.vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                    &meshes.vertices[0] as *const _ as *const c_void,
                    gl::STATIC_DRAW,
                );
    
                gl::BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    (meshes.indices.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
                    &meshes.indices[0] as *const _ as *const c_void,
                    gl::STATIC_DRAW,
                );

                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
                let stride = 8 * mem::size_of::<GLfloat>() as i32;
                gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
                gl::EnableVertexAttribArray(0);

                parsed_models.push(ParsedModel {
                    vao,
                    indices: meshes.indices.clone(),
                });
            }

            // Get uniforms
            let projection_uniform = gl::GetUniformLocation(program, PROJECTION.as_ptr());
            let view_uniform = gl::GetUniformLocation(program, VIEW.as_ptr());
            let transformation_uniform = gl::GetUniformLocation(program, TRANSFORMATION.as_ptr());

            Ok(Self {
                shader_id: program,
                parsed_meshes: parsed_models,
                projection_uniform,
                view_uniform,
                transformation_uniform,
            })
        }
    }

    pub fn draw(
        &self,
        models: &[Model],
        view: Matrix4<f32>,
        projection: Matrix4<f32>,
    ) {
        unsafe {
            gl::UseProgram(self.shader_id);

            gl::UniformMatrix4fv(self.view_uniform, 1, gl::FALSE, view.as_ptr());
            gl::UniformMatrix4fv(self.projection_uniform, 1, gl::FALSE, projection.as_ptr());

            for model in models {
                for mesh_index in &model.meshes {
                    let mesh = &self.parsed_meshes[*mesh_index];
                    gl::BindVertexArray(mesh.vao);
                    gl::BufferData(
                        gl::ELEMENT_ARRAY_BUFFER,
                        (mesh.indices.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
                        &mesh.indices[0] as *const _ as *const c_void,
                        gl::STATIC_DRAW,
                    );
                    gl::ActiveTexture(gl::TEXTURE0);
                    let transform = model.transform.to_matrix4();
                    gl::UniformMatrix4fv(
                        self.transformation_uniform,
                        1,
                        gl::FALSE,
                        transform.as_ptr(),
                    );
                    gl::DrawElements(
                        gl::TRIANGLES,
                        mesh.indices.len() as i32,
                        gl::UNSIGNED_INT,
                        ptr::null(),
                    );
                }
            }
            gl::BindVertexArray(0);
        }
    }
}

struct ParsedModel {
    vao: u32,
    indices: Vec<u32>,
}

const TRANSFORMATION: &'static CStr =
    unsafe { CStr::from_bytes_with_nul_unchecked(b"transformation\0") };
    
const PROJECTION: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"projection\0") };
const VIEW: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"view\0") };
