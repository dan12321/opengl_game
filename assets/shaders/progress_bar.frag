#version 330 core

in float Pos;

uniform float progress;
uniform vec3 base_color;
uniform vec3 progress_color;

out vec4 FragColor;

void main()
{
    // If we're withen progress will = 1.0 otherwise 0.0
    float before_progress = ceil(max(progress - Pos, 0.0));
    vec3 color_masked = before_progress * progress_color + (1.0 - before_progress) * base_color;
    FragColor = vec4(color_masked, 1.0);
}