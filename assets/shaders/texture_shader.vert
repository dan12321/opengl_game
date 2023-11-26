#version 330 core
layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 texCoord;

uniform mat4 transformation;
uniform mat4 view;
uniform mat4 projection;
uniform vec3 lightPosition;
uniform float lightStrength;
uniform vec3 cameraPosition;

out vec2 TexCoord;
out vec3 FragPos;
out vec3 Normal;

void main()
{
    vec4 world_position = transformation * vec4(pos, 1.0);
    FragPos = vec3(world_position);
    vec4 values = projection * view * world_position;
    gl_Position = values;

    vec3 norm = normalize(mat3(transpose(inverse(transformation))) * normal);
    Normal = norm;

    TexCoord = texCoord;
}
