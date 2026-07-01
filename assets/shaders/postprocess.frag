#version 330 core

in vec2 v_uv;

out vec4 frag_color;

uniform sampler2D u_scene;
uniform vec2 u_resolution;

uniform int u_vignette_enabled;
// Scaled like Unity Post Processing Stack (_Vignette_Settings)
uniform float u_vignette_intensity;  // intensity * 3
uniform float u_vignette_smoothness; // smoothness * 5
uniform float u_vignette_roundness;  // remapped roundness exponent
uniform float u_vignette_rounded;    // 1 = aspect-correct circle

vec3 apply_vignette(vec3 color, vec2 uv) {
    vec2 d = abs(uv - vec2(0.5)) * u_vignette_intensity;
    float aspect = u_resolution.x / max(u_resolution.y, 1.0);
    d.x *= mix(1.0, aspect, u_vignette_rounded);
    d = pow(clamp(d, vec2(0.0), vec2(1.0)), vec2(u_vignette_roundness));
    float vfactor = pow(clamp(1.0 - dot(d, d), 0.0, 1.0), u_vignette_smoothness);
    return color * vfactor;
}

void main() {
    vec3 color = texture(u_scene, v_uv).rgb;

    if (u_vignette_enabled != 0) {
        color = apply_vignette(color, v_uv);
    }

    frag_color = vec4(color, 1.0);
}
