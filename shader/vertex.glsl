#version 330 core

attribute vec2 pos;

uniform vec2 viewport;

void main() {
    float width = viewport.x;
    float height = viewport.y;

    float x = (pos.x / (width * 0.5f)) + ((1.0f - width) / width);
    float y = -((pos.y / (height * 0.5f)) + ((1.0f - height) / height));

    gl_Position = vec4(x, y, 0.0f, 1.0f);
}
