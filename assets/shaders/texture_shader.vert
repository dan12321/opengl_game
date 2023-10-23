#version 330 core
layout (location = 0) in vec3 pos;
layout (location = 1) in vec2 texCoord;

uniform mat4 transformation; 
uniform mat4 view;
uniform mat4 projection;
uniform vec3 lightPosition;
uniform float lightStrength;

out vec2 TexCoord;
out float LightIntensity;

void main()
{
    vec4 world_position = transformation * vec4(pos, 1.0);
    vec4 values = projection * view * world_position;
    float dist = distance(world_position, vec4(lightPosition, 1.0));
    LightIntensity = min(1.0, lightStrength / pow(dist, 2.0));
    TexCoord = texCoord;
    gl_Position = values;
}