#version 330 core
layout(location = 0) in vec3 pos;

uniform mat4 transformation;

out float Pos;

void main()
{
    vec4 screen_position = transformation * vec4(pos, 1.0);
    gl_Position = screen_position;
    Pos = pos.x;
}