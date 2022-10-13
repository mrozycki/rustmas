#version 400 core
in vec3 center_position;
out vec4 out_color;

const vec2 resolution = vec2(1024, 768);

void main() {
    vec2 pos = ((center_position.xy + 1) / 2) * resolution;
    vec2 diff = (gl_FragCoord.xy - pos);

    if (dot(diff, diff) < 5*5) {
        out_color = vec4(1.0, 0.0, 0.0, 1.0);
    } else {
        discard;
    }
}