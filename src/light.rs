use gl::types::GLfloat;

use crate::model::Model;

pub struct Light<const R: usize, const S: usize> {
    pub model: Model<R, S>,
    pub diffuse: (GLfloat, GLfloat, GLfloat),
    pub specular: (GLfloat, GLfloat, GLfloat),
    pub strength: GLfloat,
}

#[derive(Debug)]
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

pub fn template_light(index: usize, prop: Prop) -> String {
    let property = match prop {
        Prop::Position => "position",
        Prop::Diffuse => "diffuse",
        Prop::Specular => "specular",
        Prop::Strength => "strength",
    };

    format!("pointLights[{}].{}", index, property)
}

pub enum Prop {
    Position,
    Diffuse,
    Specular,
    Strength,
}

