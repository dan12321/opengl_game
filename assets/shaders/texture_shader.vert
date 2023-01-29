#version 330 core
layout (location = 0) in vec3 pos;
layout (location = 1) in vec2 texCoord;

uniform mat4 transformation; 
uniform mat4 view;
uniform mat4 projection;

out vec2 TexCoord;

void main()
{
    vec4 values = projection * view * transformation * vec4(pos, 1.0);
    TexCoord = texCoord;
    gl_Position = values;
}