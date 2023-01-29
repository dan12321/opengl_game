#version 330 core

in vec2 TexCoord;
uniform sampler2D texture0;

out vec4 FragColor;

void main()
{
    FragColor = texture2D(texture0, TexCoord) * vec4(TexCoord, 0.5, 1.0);
    // FragColor = vec4(TexCoord, 0.5, 1.0);
}