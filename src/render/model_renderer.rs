use std::{
    ffi::{c_void, CStr, CString},
    mem,
    path::PathBuf,
    ptr,
};

use gl::types::*;
use image::{ColorType, DynamicImage};
use na::Matrix4;

use crate::{
    shader::{create_shader, template_dir_light, template_point_light, DirLight, DirLightProp, OpenGLError, PointLight, PointLightProp},
    state::{Model, XYZ},
};

use super::model_loader::{Material, Mesh};

pub struct ModelRenderer {
    shader_id: u32,
    parsed_meshes: Vec<ParsedModel>,
    materials: Vec<Material>,
    textures: Vec<u32>,
    view_uniform: i32,
    projection_uniform: i32,
    camera_position_uniform: i32,
    transformation_uniform: i32,
    material_uniform: MaterialUniform,
    point_light_uniform: [PointLightUniform; MAX_LIGHTS],
    dir_light_uniform: [DirLightUniform; MAX_LIGHTS],
}

impl ModelRenderer {
    pub fn new(
        vert_shader: &PathBuf,
        frag_shader: &PathBuf,
        meshes: Vec<Mesh>,
        materials: Vec<Material>,
        textures: &[DynamicImage],
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

                parsed_models.push(ParsedModel {
                    vao,
                    indices: meshes.indices,
                    material: meshes.material,
                });
            }

            // texture
            let mut texture_locations = Vec::with_capacity(textures.len());
            for texture in textures {
                let mut texture_location = 0;
                let texture_bytes = texture.as_bytes();
                let format = match texture.color() {
                    ColorType::Rgb8 => gl::RGB,
                    ColorType::Rgba8 => gl::RGBA,
                    // TODO: Find what each color type should match to
                    _ => gl::RGB,
                };
                gl::GenTextures(1, &mut texture_location);
                gl::BindTexture(gl::TEXTURE_2D, texture_location);
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    format as i32,
                    texture.width() as i32,
                    texture.height() as i32,
                    0,
                    format,
                    gl::UNSIGNED_BYTE,
                    &texture_bytes[0] as *const _ as *const c_void,
                );
                gl::GenerateMipmap(gl::TEXTURE_2D);
                texture_locations.push(texture_location);
            }

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

            let mut point_lights = [PointLightUniform {
                position: -1,
                diffuse: -1,
                specular: -1,
                strength: -1,
            }; MAX_LIGHTS];
            for i in 0..MAX_LIGHTS {
                let pos = template_point_light(i, PointLightProp::Position);
                let dif = template_point_light(i, PointLightProp::Diffuse);
                let spec = template_point_light(i, PointLightProp::Specular);
                let stren = template_point_light(i, PointLightProp::Strength);
                let position = CString::new(pos.as_bytes()).unwrap();
                let diffuse = CString::new(dif.as_bytes()).unwrap();
                let specular = CString::new(spec.as_bytes()).unwrap();
                let strength = CString::new(stren.as_bytes()).unwrap();
                point_lights[i] = PointLightUniform {
                    position: gl::GetUniformLocation(program, position.as_ptr()),
                    diffuse: gl::GetUniformLocation(program, diffuse.as_ptr()),
                    specular: gl::GetUniformLocation(program, specular.as_ptr()),
                    strength: gl::GetUniformLocation(program, strength.as_ptr()),
                };
            }
            let mut dir_lights = [DirLightUniform {
                direction: -1,
                diffuse: -1,
                specular: -1,
            }; MAX_LIGHTS];
            for i in 0..MAX_LIGHTS {
                let dir = template_dir_light(i, DirLightProp::Direction);
                let dif = template_dir_light(i, DirLightProp::Diffuse);
                let spec = template_dir_light(i, DirLightProp::Specular);
                let direction = CString::new(dir.as_bytes()).unwrap();
                let diffuse = CString::new(dif.as_bytes()).unwrap();
                let specular = CString::new(spec.as_bytes()).unwrap();
                dir_lights[i] = DirLightUniform {
                    direction: gl::GetUniformLocation(program, direction.as_ptr()),
                    diffuse: gl::GetUniformLocation(program, diffuse.as_ptr()),
                    specular: gl::GetUniformLocation(program, specular.as_ptr()),
                };
            }
            Ok(Self {
                shader_id: program,
                parsed_meshes: parsed_models,
                textures: texture_locations,
                material_uniform: material,
                camera_position_uniform,
                projection_uniform,
                view_uniform,
                transformation_uniform,
                point_light_uniform: point_lights,
                dir_light_uniform: dir_lights,
                materials,
            })
        }
    }

    pub fn draw(
        &self,
        models: &[Model],
        point_lights: &[PointLight],
        dir_lights: &[DirLight],
        camera_position: &XYZ,
        view: Matrix4<f32>,
        projection: Matrix4<f32>,
    ) {
        unsafe {
            gl::UseProgram(self.shader_id);

            for i in 0..point_lights.len() {
                gl::Uniform3f(
                    self.point_light_uniform[i].position,
                    point_lights[i].position.0,
                    point_lights[i].position.1,
                    point_lights[i].position.2,
                );
                gl::Uniform3f(
                    self.point_light_uniform[i].diffuse,
                    point_lights[i].diffuse.0,
                    point_lights[i].diffuse.1,
                    point_lights[i].diffuse.2,
                );
                gl::Uniform3f(
                    self.point_light_uniform[i].specular,
                    point_lights[i].specular.0,
                    point_lights[i].specular.1,
                    point_lights[i].specular.2,
                );
                gl::Uniform1f(self.point_light_uniform[i].strength, point_lights[i].strength);
            }
            for i in 0..dir_lights.len() {
                gl::Uniform3f(
                    self.dir_light_uniform[i].direction,
                    dir_lights[i].direction.0,
                    dir_lights[i].direction.1,
                    dir_lights[i].direction.2,
                );
                gl::Uniform3f(
                    self.dir_light_uniform[i].diffuse,
                    dir_lights[i].diffuse.0,
                    dir_lights[i].diffuse.1,
                    dir_lights[i].diffuse.2,
                );
                gl::Uniform3f(
                    self.dir_light_uniform[i].specular,
                    dir_lights[i].specular.0,
                    dir_lights[i].specular.1,
                    dir_lights[i].specular.2,
                );
            }
            gl::UniformMatrix4fv(self.view_uniform, 1, gl::FALSE, view.as_ptr());
            gl::UniformMatrix4fv(self.projection_uniform, 1, gl::FALSE, projection.as_ptr());
            gl::Uniform3f(
                self.camera_position_uniform,
                camera_position.x,
                camera_position.y,
                camera_position.z,
            );

            for model in models {
                for mesh_index in &model.meshes {
                    let mesh = &self.parsed_meshes[*mesh_index];
                    let material = &self.materials[mesh.material];
                    gl::BindVertexArray(mesh.vao);
                    gl::BufferData(
                        gl::ELEMENT_ARRAY_BUFFER,
                        (mesh.indices.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
                        &mesh.indices[0] as *const _ as *const c_void,
                        gl::STATIC_DRAW,
                    );
                    gl::ActiveTexture(gl::TEXTURE0);
                    gl::BindTexture(gl::TEXTURE_2D, self.textures[material.diffuse_map]);
                    let transform = model.transform.to_matrix4();
                    gl::UniformMatrix4fv(
                        self.transformation_uniform,
                        1,
                        gl::FALSE,
                        transform.as_ptr(),
                    );
                    gl::Uniform3f(
                        self.material_uniform.ambient,
                        material.ambient.0,
                        material.ambient.1,
                        material.ambient.2,
                    );
                    gl::Uniform3f(
                        self.material_uniform.diffuse,
                        material.diffuse.0,
                        material.diffuse.1,
                        material.diffuse.2,
                    );
                    // Use Texture1 for specular
                    gl::Uniform1i(self.material_uniform.specular, 1);
                    gl::ActiveTexture(gl::TEXTURE1);
                    gl::BindTexture(gl::TEXTURE_2D, self.textures[material.specular_map]);
                    gl::Uniform1i(self.material_uniform.shininess, material.specular_exponent as i32);
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

#[derive(Copy, Clone, Debug)]
struct PointLightUniform {
    position: i32,
    diffuse: i32,
    specular: i32,
    strength: i32,
}

#[derive(Copy, Clone, Debug)]
struct DirLightUniform {
    direction: i32,
    diffuse: i32,
    specular: i32,
}

struct MaterialUniform {
    ambient: i32,
    diffuse: i32,
    specular: i32,
    shininess: i32,
}

struct ParsedModel {
    vao: u32,
    indices: Vec<u32>,
    material: usize,
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
