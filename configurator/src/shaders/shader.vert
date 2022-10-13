#version 400 core
in vec3 position;
out vec3 center_position;

uniform mat4 mvp;

void main() {
    gl_Position = mvp * vec4(position, 1.0);
    center_position = gl_Position.xyz / gl_Position.w;
}