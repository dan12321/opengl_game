use std::collections::HashMap;
use std::{
    ptr,
    str,
};
use std::ffi::{CString, NulError};
use std::fs::OpenOptions;
use std::io::{Error as IoError, Read};

use na::Matrix4;
use tracing::{error};

use gl::types::*;

pub struct Shader {
    id: u32,
    uniform1fs: HashMap<String, Uniform<f32>>,
    uniform1is: HashMap<String, Uniform<i32>>,
    uniform2fs: HashMap<String, Uniform<(f32, f32)>>,
    uniform3fs: HashMap<String, Uniform<(f32, f32, f32)>>,
    uniform_matrix4s: HashMap<String, Uniform<Matrix4<f32>>>,

}

impl Shader {
    pub fn new(vertex_shader_path: &str, fragment_shader_path: &str) -> Result<Self, OpenGLError> {
        let vertex_shader = unsafe { Self::create_shader(vertex_shader_path, gl::VERTEX_SHADER)? };
        let fragment_shader = unsafe { Self::create_shader(fragment_shader_path, gl::FRAGMENT_SHADER)? };
        let shader_program = unsafe { gl::CreateProgram() };
        unsafe {
            gl::AttachShader(shader_program, vertex_shader);
            gl::AttachShader(shader_program, fragment_shader);
            gl::LinkProgram(shader_program);
            let mut success = gl::FALSE as GLsizei;
            gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
            if success == (gl::FALSE as GLsizei) {
                let mut len = 0;
                gl::GetProgramiv(shader_program, gl::INFO_LOG_LENGTH, &mut len);
                let mut error_buffer = Vec::with_capacity(len as usize);
                error_buffer.set_len(len as usize - 1);
                gl::GetProgramInfoLog(shader_program, len, ptr::null_mut(), error_buffer.as_mut_ptr() as *mut GLchar);
                error!(message=str::from_utf8(&error_buffer).unwrap(), "failed_to_link_program");
                return Err(OpenGLError::FailedToLinkProgram);
            }
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
        }
        return Ok(Self {
            id: shader_program,
            uniform1fs: HashMap::new(),
            uniform2fs: HashMap::new(),
            uniform3fs: HashMap::new(),
            uniform1is: HashMap::new(),
            uniform_matrix4s: HashMap::new(),
        });
    }

    pub unsafe fn use_program(&self) {
        gl::UseProgram(self.id);
    }

    pub fn add_uniform1f(mut self, name: &str, value: f32) -> Result<Self, NulError> {
        unsafe { gl::UseProgram(self.id) };
        let name_cstring = CString::new(name.as_bytes()).unwrap();
        let location = unsafe { gl::GetUniformLocation(self.id, name_cstring.as_ptr()) };
        unsafe {gl::Uniform1f(location, value); }
        let uniform = Uniform::new(location, value);
        self.uniform1fs.insert(name.into(), uniform);
        Ok(self)
    }

    pub fn get_uniform1f(&self, name: &str) -> Option<f32> {
        match self.uniform1fs.get(name) {
            Some(u) => Some(u.value),
            None => None,
        }
    }

    /// Sets the value of the uniform in the shader
    /// Returns the old value or none if the uniform doesn't exist
    pub fn set_uniform1f(&mut self, name: &str, value: f32) -> Option<f32> {
        unsafe { gl::UseProgram(self.id) };
        let old_uniform = self.uniform1fs.get(name)?;
        unsafe { gl::Uniform1f(old_uniform.location, value); }
        let new_uniform = Uniform::new(old_uniform.location, value);
        match self.uniform1fs.insert(name.into(), new_uniform) {
            Some(u) => Some(u.value),
            None => None,
        }
    }


    pub fn add_uniform1i(mut self, name: &str, value: i32) -> Result<Self, NulError> {
        unsafe { gl::UseProgram(self.id) };
        let name_cstring = CString::new(name.as_bytes()).unwrap();
        let location = unsafe { gl::GetUniformLocation(self.id, name_cstring.as_ptr()) };
        unsafe {gl::Uniform1i(location, value); }
        let uniform = Uniform::new(location, value);
        self.uniform1is.insert(name.into(), uniform);
        Ok(self)
    }

    pub fn get_uniform1i(&self, name: &str) -> Option<i32> {
        match self.uniform1is.get(name) {
            Some(u) => Some(u.value),
            None => None,
        }
    }

    /// Sets the value of the uniform in the shader
    /// Returns the old value or none if the uniform doesn't exist
    pub fn set_uniform1i(&mut self, name: &str, value: i32) -> Option<i32> {
        unsafe { gl::UseProgram(self.id) };
        let old_uniform = self.uniform1is.get(name)?;
        unsafe { gl::Uniform1i(old_uniform.location, value); }
        let new_uniform = Uniform::new(old_uniform.location, value);
        match self.uniform1is.insert(name.into(), new_uniform) {
            Some(u) => Some(u.value),
            None => None,
        }
    }

    pub fn add_uniform_mat4(mut self, name: &str, value: Matrix4<f32>) -> Result<Self, NulError> {
        unsafe { gl::UseProgram(self.id) };
        let name_cstring = CString::new(name.as_bytes()).unwrap();
        let location = unsafe { gl::GetUniformLocation(self.id, name_cstring.as_ptr()) };
        let data = value.as_ptr();
        unsafe { gl::UniformMatrix4fv(location, 1, gl::FALSE, data); }
        let uniform = Uniform::new(location, value);
        self.uniform_matrix4s.insert(name.into(), uniform);
        Ok(self)
    }

    pub fn get_uniform_mat4(&self, name: &str) -> Option<Matrix4<f32>> {
        match self.uniform_matrix4s.get(name) {
            Some(u) => Some(u.value),
            None => None,
        }
    }

    /// Sets the value of the uniform in the shader
    /// Returns the old value or none if the uniform doesn't exist
    pub fn set_uniform_mat4(&mut self, name: &str, value: Matrix4<f32>) -> Option<Matrix4<f32>> {
        unsafe { gl::UseProgram(self.id) };
        let old_uniform = self.uniform_matrix4s.get(name)?;
        let data = value.as_ptr();
        unsafe { gl::UniformMatrix4fv(old_uniform.location, 1, gl::FALSE,  data) }
        let new_uniform = Uniform::new(old_uniform.location, value);
        match self.uniform_matrix4s.insert(name.into(), new_uniform) {
            Some(u) => Some(u.value),
            None => None,
        }
    }

    pub fn add_uniform2f(mut self, name: &str, value: (f32, f32)) -> Result<Self, NulError> {
        unsafe { gl::UseProgram(self.id) };
        let name_cstring = CString::new(name.as_bytes()).unwrap();
        let location = unsafe { gl::GetUniformLocation(self.id, name_cstring.as_ptr()) };
        unsafe { gl::Uniform2f(location, value.0, value.1); }
        let uniform = Uniform::new(location, value);
        self.uniform2fs.insert(name.into(), uniform);
        Ok(self)
    }

    pub fn get_uniform2f(&self, name: &str) -> Option<(f32, f32)> {
        match self.uniform2fs.get(name) {
            Some(u) => Some(u.value),
            None => None,
        }
    }

    /// Sets the value of the uniform in the shader
    /// Returns the old value or none if the uniform doesn't exist
    pub fn set_uniform2f(&mut self, name: &str, value: (f32, f32)) -> Option<(f32, f32)> {
        unsafe { gl::UseProgram(self.id) };
        let old_uniform = self.uniform2fs.get(name)?;
        unsafe {gl::Uniform2f(old_uniform.location, value.0, value.1); }
        let new_uniform = Uniform::new(old_uniform.location, value);
        match self.uniform2fs.insert(name.into(), new_uniform) {
            Some(u) => Some(u.value),
            None => None,
        }
    }

    pub fn add_uniform3f(mut self, name: &str, value: (f32, f32, f32)) -> Result<Self, NulError> {
        unsafe { gl::UseProgram(self.id) };
        let name_cstring = CString::new(name.as_bytes()).unwrap();
        let location = unsafe { gl::GetUniformLocation(self.id, name_cstring.as_ptr()) };
        unsafe { gl::Uniform3f(location, value.0, value.1, value.2); }
        let uniform = Uniform::new(location, value);
        self.uniform3fs.insert(name.into(), uniform);
        Ok(self)
    }

    pub fn get_uniform3f(&self, name: &str) -> Option<(f32, f32, f32)> {
        match self.uniform3fs.get(name) {
            Some(u) => Some(u.value),
            None => None,
        }
    }

    /// Sets the value of the uniform in the shader
    /// Returns the old value or none if the uniform doesn't exist
    pub fn set_uniform3f(&mut self, name: &str, value: (f32, f32, f32)) -> Option<(f32, f32, f32)> {
        unsafe { gl::UseProgram(self.id) };
        let old_uniform = self.uniform3fs.get(name)?;
        unsafe {gl::Uniform3f(old_uniform.location, value.0, value.1, value.2); }
        let new_uniform = Uniform::new(old_uniform.location, value);
        match self.uniform3fs.insert(name.into(), new_uniform) {
            Some(u) => Some(u.value),
            None => None,
        }
    }

    unsafe fn create_shader(shader_path: &str, shader_type: GLenum) -> Result<u32, OpenGLError> {
        let mut file = match OpenOptions::new()
            .read(true)
            .write(false)
            .open(shader_path) {
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
            gl::GetShaderInfoLog(shader, len, ptr::null_mut(), error_buffer.as_mut_ptr() as *mut GLchar);
            error!(message=str::from_utf8(&error_buffer).unwrap(), shader=shader_path , "failed_to_compile_shader");
            return Err(OpenGLError::FailedToCompileShader);
        }
        Ok(shader)
    }
}

struct Uniform<T> {
    location: i32,
    value: T
}

impl<T> Uniform<T> {
    fn new(location: i32, value: T) -> Self {
        Self {
            location,
            value,
        }
    }
}

#[derive(Debug)]
pub enum OpenGLError {
    FailedToCompileShader,
    FailedToReadShader(IoError),
    FailedToLinkProgram,
}