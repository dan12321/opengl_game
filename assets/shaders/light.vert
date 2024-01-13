#version 330 core
layout (location = 0) in vec3 pos;

uniform vec3 color;
uniform mat4 transformation;
uniform mat4 view;
uniform mat4 projection;

void main() {
    vec4 position = projection * view * transformation * vec4(pos, 1.0);
    gl_Position = position;
}
