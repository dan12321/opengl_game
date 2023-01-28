#version 330 core
layout (location = 0) in vec3 aPos;

uniform mat4 transformation; 
uniform mat4 view;
uniform mat4 projection;

out vec4 vertexColor;

void main()
{
    vec4 values = projection * view * transformation * vec4(aPos, 1.0);
    gl_Position = values;
    vertexColor = vec4((aPos.x + 1.0) / 2, (aPos.y + 1.0) / 2, (aPos.z + 1.0) / 2, 1.0);
}