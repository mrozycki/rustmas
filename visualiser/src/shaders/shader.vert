#version 400

in vec3 position;
in vec3 in_color;

out vec3 color;

uniform mat4 mvp;

void main() {
    gl_Position = mvp * vec4(position, 1.0);
    color = in_color;
}