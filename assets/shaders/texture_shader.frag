#version 330 core

in vec2 TexCoord;
in float LightIntensity;
uniform sampler2D texture0;
uniform vec3 lightColor;

out vec4 FragColor;

void main()
{
    vec4 texColor = texture2D(texture0, TexCoord);
    vec3 light = LightIntensity * lightColor;
    FragColor = vec4(light, 1.0) * texColor;
}