#version 400 core
out vec4 out_color;

const vec3 light_position = vec3(-5.0, -5.0, -5.0);
const vec3 camera_position = vec3(0.0, 0.0, -5.0);

void main() {
    vec2 point_coord_scaled = 2 * gl_PointCoord - 1;
    float mag = dot(point_coord_scaled.xy, point_coord_scaled.xy);

    float z = -sqrt(1.0 - mag);

    vec3 position = vec3(point_coord_scaled, z);

    if(mag > 1.0) discard;

    vec3 normal = normalize(position);

    out_color = vec4(1.0, 0.0, 0.0, 1.0);

    vec3 light = normalize(light_position-position);
    vec3 view = normalize(camera_position-position);
    vec3 h = normalize(light+view);

    vec3 ambient = 0.1 * out_color.xyz;
    vec3 diffuse = vec3(max(dot(normal,light),0.0));
    vec3 specular = vec3(max(0.2*pow(dot(normal,h),200.0),0.0));
    out_color.xyz *= ambient+diffuse+specular;
}