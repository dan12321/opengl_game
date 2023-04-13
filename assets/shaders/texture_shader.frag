#version 330 core

in vec2 TexCoord;
uniform sampler2D texture0;

out vec4 FragColor;

void main()
{
    FragColor = texture2D(texture0, TexCoord);
}