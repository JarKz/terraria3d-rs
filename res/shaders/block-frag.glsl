#version 410 core

in vec2 uv_t;
in vec3 norm;

out vec4 Color;

uniform sampler2DArray texel;

void main()
{
    Color = texture(texel, vec3(uv_t, 0.0f));
}
