#version 410 core

in vec2 uv_t;
in vec3 norm;
in float fog_factor;

out vec4 Color;

uniform vec3 fog_color;

uniform sampler2DArray texel;

void main()
{
    Color = texture(texel, vec3(uv_t, 0.0f));
    Color = mix(vec4(fog_color, 1.0f), Color, fog_factor);
}
