#version 330 core

in float Pos;

uniform float progress;
uniform vec3 base_color;
uniform vec3 progress_color;
uniform vec3 merge_color;
uniform float merge_amount;

out vec4 FragColor;

void main()
{
    // If we're withen progress will = 1.0 otherwise 0.0
    float before_progress = ceil(max(progress - Pos, 0.0));
    vec3 color_masked = before_progress * progress_color + (1.0 - before_progress) * base_color;

    // Merge color
    vec3 color = merge_color * merge_amount + (1.0 - merge_amount) * color_masked;

    FragColor = vec4(color, 1.0);
}
