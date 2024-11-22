use gl::types::*;

pub const CUBE_VERTICES: [GLfloat; 24] = [
    // Front
    0.5, 0.5, 0.5, // top right
    0.5, -0.5, 0.5, // bottom right
    -0.5, 0.5, 0.5, // top left
    -0.5, -0.5, 0.5, // bottom left
    // Back
    0.5, 0.5, -0.5, // top right
    0.5, -0.5, -0.5, // bottom right
    -0.5, 0.5, -0.5, // top left
    -0.5, -0.5, -0.5, // bottom left
];

pub const CUBE_INDICES: [GLuint; 36] = [
    0, 1, 2, // front
    2, 3, 1, 4, 5, 6, // back
    6, 7, 5, 0, 2, 4, // top
    4, 6, 2, 1, 3, 5, // bottom
    5, 7, 3, 2, 3, 6, // left
    6, 7, 3, 0, 1, 4, // right
    4, 5, 1,
];

pub const QUAD_VERTICES: [GLfloat; 12] =
    [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0];

pub const QUAD_INDICES: [GLuint; 6] = [0, 1, 2, 1, 3, 2];

