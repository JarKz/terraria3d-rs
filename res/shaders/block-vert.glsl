#version 410 core

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 a_norm;
layout (location = 2) in vec2 uv;

uniform mat4 projection;
uniform mat4 view;
uniform mat4 model;

uniform vec3 camera_position;
uniform float fog_min_dist;
uniform float fog_max_dist;

out vec2 uv_t;
out vec3 norm;
out float fog_factor;

float compute_fog_factor(vec3 vert_pos) {
    float dist = length(vert_pos - camera_position);
    float fog_range = fog_max_dist - fog_min_dist;
    float fog_distance = fog_max_dist - dist;
    float fog_factor = fog_distance / fog_range;
    fog_factor = clamp(fog_factor, 0.0, 1.0);
    return fog_factor;
}

void main()
{
    vec4 vert_pos = model * vec4(position, 1.0f);
    gl_Position = projection * view * vert_pos;

    fog_factor = compute_fog_factor(vert_pos.xyz);

    uv_t = uv;
    norm = a_norm;
}
