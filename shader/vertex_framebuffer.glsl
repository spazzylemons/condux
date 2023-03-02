#version 330 core

attribute vec2 pos;
attribute vec2 in_coord;

out vec2 coord;

void main() {
    gl_Position = vec4(pos, 0.0f, 1.0f);
    coord = in_coord;
}
