use gl::types::GLfloat;

use crate::model::Model;

pub struct Light<const R: usize, const S: usize> {
    pub model: Model<R, S>,
    pub diffuse: (GLfloat, GLfloat, GLfloat),
    pub specular: (GLfloat, GLfloat, GLfloat),
    pub strength: GLfloat,
}

pub struct LightUniform {
    pub position: (GLfloat, GLfloat, GLfloat),
    pub diffuse: (GLfloat, GLfloat, GLfloat),
    pub specular: (GLfloat, GLfloat, GLfloat),
    pub strength: GLfloat,
}

impl<const R: usize, const S: usize> Light<R, S> {
    pub fn new(
        model: Model<R, S>,
        diffuse: (GLfloat, GLfloat, GLfloat),
        specular: (GLfloat, GLfloat, GLfloat),
        strength: GLfloat,
    ) -> Self {
        Light {
            model,
            diffuse,
            specular,
            strength,
        }
    }

    pub fn as_light_uniforms(&self) -> LightUniform {
        LightUniform {
            position: (self.model.transform.x, self.model.transform.y, self.model.transform.z),
            diffuse: self.diffuse,
            specular: self.specular,
            strength: self.strength,
        }
    }
}

pub const POSITION_UNIFORM: &'static str = "light.position";
pub const DIFFUSE_UNIFORM: &'static str = "light.diffuse";
pub const SPECULAR_UNIFORM: &'static str = "light.specular";
pub const STRENGTH_UNIFORM: &'static str = "light.strength";
