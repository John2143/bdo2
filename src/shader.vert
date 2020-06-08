#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec2 a_tex_coords;

layout(location=0) out vec2 v_tex_coords;

layout(set=1, binding=0)
uniform Uniforms {
    mat4 u_view_proj;
    float t;
    float dt;
};

layout(set=1, binding=1)
buffer Instances {
    mat4 s_models[];
};

void main() {
    v_tex_coords = a_tex_coords;
    vec3 pos = a_position;
    pos.y += t;
    gl_Position = u_view_proj * s_models[gl_InstanceIndex] * vec4(pos, 1.0);
}
