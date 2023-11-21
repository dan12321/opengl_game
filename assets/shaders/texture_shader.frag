#version 330 core

in vec2 TexCoord;
in float LightIntensity;
uniform sampler2D texture0;
uniform vec3 lightColor;
uniform vec3 ambientColor;
uniform float ambientColorIntensity;

out vec4 FragColor;

void main()
{
    vec4 tex = texture2D(texture0, TexCoord);
    vec4 ambient = vec4(ambientColorIntensity * ambientColor, 1.0);
    vec4 light = vec4(LightIntensity * lightColor, 1.0);
    FragColor = (ambient + light) * tex;
}
