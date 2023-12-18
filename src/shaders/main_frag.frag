#version 330

precision mediump float;

in vec2 position;
in vec2 uv;

out vec4 color;

uniform vec2 dims;
uniform vec4 background_col;
uniform vec4 border_col;
uniform float border_width;
uniform int mode;

uniform sampler2D tex;

#define MODE_SOLID_COLOUR 0
#define MODE_TEXTURE 1

void main() {
    switch (mode) {
        case MODE_SOLID_COLOUR:
            color = background_col;

            vec2 pos = uv * dims;

            if (border_width > 0.0
                && (pos.x < border_width
                || pos.x > dims.x - border_width
                || pos.y < border_width
                || pos.y > dims.y - border_width)) {

                color = border_col;
            }
            break;
        case MODE_TEXTURE:
            color = texture(tex, uv);
            break;
    }
}