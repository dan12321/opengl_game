use gl::types::GLfloat;

use crate::model::Model;

pub struct Light<const R: usize, const S: usize> {
    pub model: Model<R, S>,
    pub color: [GLfloat; 3],
    pub strength: GLfloat,
}

impl<const R: usize, const S: usize> Light<R, S> {
    pub fn new(model: Model<R, S>, r: GLfloat, g: GLfloat, b: GLfloat, strength: GLfloat) -> Self {
        Light {
            model,
            color: [r, g, b],
            strength,
        }
    }
}