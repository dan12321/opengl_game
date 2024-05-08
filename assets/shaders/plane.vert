#version 330 core

layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 texCoord;

uniform mat4 transformation;
uniform mat4 view;
uniform mat4 projection;
uniform float offset;

out vec2 TexCoord;
out vec3 FragPos;
out vec3 Normal;

void main()
{
    vec4 world_position = transformation * vec4(pos, 1.0);
    FragPos = vec3(world_position);
    vec4 view_position = projection * view * world_position;
    gl_Position = view_position;

    vec3 norm = mat3(transpose(inverse(transformation))) * vec3(1.0, 1.0, 1.0);
    Normal = norm;

    TexCoord = texCoord;
}
