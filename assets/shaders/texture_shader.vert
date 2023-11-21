#version 330 core
layout (location = 0) in vec3 pos;
layout (location = 1) in vec3 normal;
layout (location = 2) in vec2 texCoord;

uniform mat4 transformation; 
uniform mat4 view;
uniform mat4 projection;
uniform vec3 lightPosition;
uniform float lightStrength;

out vec2 TexCoord;
out vec3 Normal;
out float LightIntensity;

void main()
{
    vec4 world_position = transformation * vec4(pos, 1.0);
    vec4 values = projection * view * world_position;
    float dist = distance(world_position, vec4(lightPosition, 1.0));
    vec3 lightDir = normalize(lightPosition - vec3(world_position));
    vec3 norm = normalize(mat3(transpose(inverse(transformation))) * normal);
    float diffuse = max(dot(norm, lightDir), 0.0);
    LightIntensity = min(1.0, lightStrength / pow(dist, 2.0)) * diffuse;
    TexCoord = texCoord;
    gl_Position = values;
}
