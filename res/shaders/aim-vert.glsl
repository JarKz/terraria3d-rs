#version 410 core

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 a_color;

uniform mat4 model;

out vec3 color_o;

void main()
{
    gl_Position = model * vec4(position, 1.0f);
    color_o = a_color;
}
