#version 410 core

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 a_norm;
layout (location = 2) in vec2 uv;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

out vec2 uv_t;
out vec3 norm;

void main()
{
    gl_Position = projection * view * model * vec4(position, 1.0f);
    // gl_Position = vec4(position, 1.0);
    uv_t = uv;
    norm = a_norm;
}
