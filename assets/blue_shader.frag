#version 330 core

in vec4 vertexColor;
in vec2 TexCoord;

uniform sampler2D aTexture;
uniform sampler2D bTexture;
uniform float offset;

out vec4 FragColor;

void main()
{
    float ratio = min(smoothstep(-0.1-offset, -0.05-offset, -abs(TexCoord.x - 0.5)), smoothstep(-0.1-offset, -0.05-offset, -abs(TexCoord.y -0.5)));
    FragColor = mix(texture2D(aTexture, TexCoord), texture2D(bTexture, vec2(-TexCoord.x, TexCoord.y)), ratio * offset);
}