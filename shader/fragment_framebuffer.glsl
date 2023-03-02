#version 330 core

out vec4 color;

in vec2 coord;

uniform sampler2D tex;

vec3 PHOSPOR = vec3(0.2f, 1.0f, 0.4f);
float GLOW_STRENGTH = 0.2f;

void main() {
    ivec2 size = textureSize(tex, 0);
    vec2 scale = 1.0f / size;
    float result = texture(tex, coord).r;
    vec2 d = size * coord;
    vec2 clamp_lo = vec2(0.0f, 0.0f);
    vec2 clamp_hi = size - 1.0f;
    for (int i = -1; i <= 1; i++) {
        for (int j = -1; j <= 1; j++) {
            ivec2 c = ivec2(i, j);
            result += texture(tex, clamp(d + c, clamp_lo, clamp_hi) * scale).r * GLOW_STRENGTH;
        }
    }
    color = vec4(PHOSPOR * result, 1.0f);
}
