#version 410 core

in vec3 color_o;

out vec4 Color;

void main()
{
    Color = vec4(color_o, 1.0f);
}
