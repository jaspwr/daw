#version 330

in vec4 in_position;

out vec2 position;
out vec2 uv;

uniform vec2 window_size;

void main() {
    position = ((in_position.xy) / window_size) * 2.0 - 1.0;
    uv = in_position.zw;

    gl_Position = vec4(position, 1.0, 1.0);
}