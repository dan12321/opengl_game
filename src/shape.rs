use gl::types::*;

pub const TEXTURED_CUBE_VERTICES: [GLfloat; 120] = [
    // Front
    0.5,  0.5, 0.5, 1.0, 1.0, // top right
    0.5, -0.5, 0.5, 1.0, 0.0,  // bottom right
   -0.5,  0.5, 0.5, 0.0, 1.0,   // top left
   -0.5,  -0.5, 0.5, 0.0, 0.0,   // bottom left

    // Back
    0.5,  0.5, -0.5, 1.0, 1.0, // top right
    0.5, -0.5, -0.5, 1.0, 0.0,  // bottom right
   -0.5,  0.5, -0.5, 0.0, 1.0,  // top left
   -0.5,  -0.5, -0.5, 0.0, 0.0,   // bottom left

    // Top
    -0.5,  0.5, -0.5, 1.0, 1.0, // left back
    0.5, 0.5, -0.5, 1.0, 0.0,   // right back
    -0.5,  0.5, 0.5, 0.0, 1.0,  // left front
    0.5,  0.5, 0.5, 0.0, 0.0,   // right front

    // Bottom
    -0.5,  -0.5, -0.5, 1.0, 1.0, // left back
    0.5, -0.5, -0.5, 1.0, 0.0,   // right back
    -0.5, -0.5, 0.5, 0.0, 1.0,   // left front
    0.5,  -0.5, 0.5, 0.0, 0.0,   // right front

    // Left
    -0.5,  -0.5, -0.5, 1.0, 1.0, // bottom back
    -0.5, 0.5, -0.5, 1.0, 0.0,   // top back
    -0.5,  -0.5, 0.5, 0.0, 1.0,  // bottom front
    -0.5,  0.5, 0.5, 0.0, 0.0,   // top front

    // Right
    0.5,  -0.5, -0.5, 1.0, 1.0, // bottom back
    0.5, 0.5, -0.5, 1.0, 0.0,   // top back
    0.5,  -0.5, 0.5, 0.0, 1.0,  // bottom front
    0.5,  0.5, 0.5, 0.0, 0.0,   // top front
];

pub const TEXTURED_CUBE_INDICES: [GLuint; 36] = [
    0, 1, 2, // front
    2, 1, 3, // front
    4, 5, 6, // back
    6, 5, 7, // back
    8, 9, 10, // top
    10, 9, 11, // top
    12, 13, 14, // bottom
    14, 13, 15, // bottom
    16, 17, 18, // left
    18, 17, 19, // left
    20, 21, 22, // right
    22, 21, 23, // right
];

pub const CUBE_VERTICES: [GLfloat; 24] = [
    // Front
    0.5,  0.5, 0.5, // top right
    0.5, -0.5, 0.5,  // bottom right
   -0.5,  0.5, 0.5,   // top left
   -0.5,  -0.5, 0.5,   // bottom left

    // Back
    0.5,  0.5, -0.5, // top right
    0.5, -0.5, -0.5,  // bottom right
   -0.5,  0.5, -0.5,  // top left
   -0.5,  -0.5, -0.5,   // bottom left
];

pub const CUBE_INDICES: [GLuint; 36] = [
    0, 1, 2, // front
    2, 3, 1,
    4, 5, 6, // back
    6, 7, 5,
    0, 2, 4, // top
    4, 6, 2,
    1, 3, 5, // bottom
    5, 7, 3,
    2, 3, 6, // left
    6, 7, 3,
    0, 1, 4, // right
    4, 5, 1,
];