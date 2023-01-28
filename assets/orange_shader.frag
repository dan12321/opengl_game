#version 330 core

in vec4 vertexColor;
uniform float offset;

out vec4 FragColor;

void main()
{
    FragColor = vertexColor;
} 