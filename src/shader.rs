use std::ffi::CString;
use std::fs::OpenOptions;
use std::io::{Error as IoError, Read};
use std::path::PathBuf;
use std::string::FromUtf8Error;
use std::{ptr, str};

use tracing::error;

use gl::types::*;

pub unsafe fn create_shader(
    shader_path: &PathBuf,
    shader_type: GLenum,
) -> Result<u32, OpenGLError> {
    let mut file = match OpenOptions::new().read(true).write(false).open(shader_path) {
        Ok(f) => f,
        Err(err) => return Err(OpenGLError::FailedToReadShader(err)),
    };

    let mut shader_bytes: Vec<u8> = Vec::new();
    match file.read_to_end(&mut shader_bytes) {
        Ok(_) => (),
        Err(err) => return Err(OpenGLError::FailedToReadShader(err)),
    }

    let shader = gl::CreateShader(shader_type);
    let vertex_str = CString::new(shader_bytes).unwrap();
    gl::ShaderSource(shader, 1, &vertex_str.as_ptr(), ptr::null());
    gl::CompileShader(shader);

    // Check worked
    let mut success = gl::FALSE as GLsizei;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

    if success == (gl::FALSE as GLsizei) {
        let mut len = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
        let mut error_buffer = Vec::with_capacity(len as usize);
        error_buffer.set_len(len as usize - 1);
        gl::GetShaderInfoLog(
            shader,
            len,
            ptr::null_mut(),
            error_buffer.as_mut_ptr() as *mut GLchar,
        );
        let path = shader_path.to_str().unwrap_or("Invalid Unicode");
        error!(
            message = str::from_utf8(&error_buffer).unwrap(),
            shader = path,
            "failed_to_compile_shader"
        );
        return Err(OpenGLError::FailedToCompileShader);
    }
    Ok(shader)
}

#[derive(Debug)]
pub enum OpenGLError {
    FailedToCompileShader,
    FailedToReadShader(IoError),
    FailedToLinkProgram(Result<String, FromUtf8Error>),
}

#[derive(Copy, Clone, Debug)]
pub struct Material {
    pub ambient: (f32, f32, f32),
    pub diffuse: (f32, f32, f32),
    pub specular_texture: usize,
    pub shininess: GLint,
    pub texture: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct PointLight {
    pub position: (GLfloat, GLfloat, GLfloat),
    pub diffuse: (GLfloat, GLfloat, GLfloat),
    pub specular: (GLfloat, GLfloat, GLfloat),
    pub strength: GLfloat,
}

pub fn template_point_light(index: usize, prop: PointLightProp) -> String {
    let property = match prop {
        PointLightProp::Position => "position",
        PointLightProp::Diffuse => "diffuse",
        PointLightProp::Specular => "specular",
        PointLightProp::Strength => "strength",
    };

    format!("pointLights[{}].{}", index, property)
}

pub enum PointLightProp {
    Position,
    Diffuse,
    Specular,
    Strength,
}

#[derive(Debug)]
pub struct DirLight {
    pub direction: (GLfloat, GLfloat, GLfloat),
    pub diffuse: (GLfloat, GLfloat, GLfloat),
    pub specular: (GLfloat, GLfloat, GLfloat),
}

pub fn template_dir_light(index: usize, prop: DirLightProp) -> String {
    let property = match prop {
        DirLightProp::Direction => "direction",
        DirLightProp::Diffuse => "diffuse",
        DirLightProp::Specular => "specular",
    };

    format!("dirLights[{}].{}", index, property)
}

pub enum DirLightProp {
    Direction,
    Diffuse,
    Specular,
}
