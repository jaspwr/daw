#version 330 core

in vec2 position;
in vec2 uv;

out vec4 color;

uniform sampler2D samp;

void main() {
    color = texture(samp, uv);
}
