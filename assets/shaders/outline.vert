#version 330 core
layout(location = 0) in vec3 pos;

uniform mat4 transformation;
uniform mat4 view;
uniform mat4 projection;

void main()
{
    vec4 scale = vec4(1.1, 1.1, 1.1, 1.0);
    vec4 world_position = transformation * (vec4(pos, 1.0) * scale);
    vec4 values = projection * view * world_position;
    gl_Position = values;
}