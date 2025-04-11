layout(location = 0) in vec2 position;
layout(location = 1) in vec2 tex_coord;

layout(location = 0) out vec4 frag_color;

void main() {
    frag_color = vec4(tex_coord, 0.0, 1.0);
}