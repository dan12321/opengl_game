use std::error::Error;
use std::ffi::CString;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{Error as IoError, Read};
use std::path::PathBuf;
use std::str::Utf8Error;
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
        let message = match str::from_utf8(&error_buffer) {
            Ok(m) => m,
            Err(e) => return Err(OpenGLError::FailedToReadFailToCompileError(e)),
        };
        error!(
            message = message,
            shader = path,
            "failed_to_compile_shader"
        );
        return Err(OpenGLError::FailedToCompileShader(message.to_string()));
    }
    Ok(shader)
}

#[derive(Debug)]
pub enum OpenGLError {
    FailedToReadFailToCompileError(Utf8Error),
    FailedToCompileShader(String),
    FailedToReadShader(IoError),
    FailedToLinkProgram(String),
    FailedToReadLinkProgramError(Utf8Error),
}

impl OpenGLError {
    pub fn link_program_error(error_buffer: &Vec<u8>) -> Self {
        let message = match str::from_utf8(error_buffer) {
            Ok(m) => m,
            Err(e) => return OpenGLError::FailedToReadLinkProgramError(e),
        };
        OpenGLError::FailedToLinkProgram(message.to_string())
    }
}

impl Display for OpenGLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FailedToReadFailToCompileError(e) => write!(f, "Could not parse fail to compile error: {}", e),
            Self::FailedToCompileShader(s) => write!(f, "Could not compile shader: {}", s),
            Self::FailedToReadShader(e) => write!(f, "Could not read shader: {}", e),
            Self::FailedToLinkProgram(s) => write!(f, "Failed to link program: {}", s),
            Self::FailedToReadLinkProgramError(e) => write!(f, "Could not parse link program error: {}", e),
        }
    }
}

impl Error for OpenGLError {}

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
