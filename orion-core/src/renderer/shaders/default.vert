#version <version>

#ifdef GL_ES
precision mediump float;
#endif

uniform mat4 view;
uniform mat4 proj;

layout (location = 0) in vec2 position;
layout (location = 1) in uint color;
layout (location = 2) in vec2 uv;
out vec4 vertexColor;
out vec2 vertexUv;

void main()
{
    vec4 color_converted = vec4(
        float((color & uint(0xFF000000)) >> 24) / 255.0, 
        float((color & uint(0x00FF0000)) >> 16) / 255.0, 
        float((color & uint(0x0000FF00)) >> 8) / 255.0, 
        float((color & uint(0x000000FF)) >> 0) / 255.0
    );

    gl_Position = proj * view * vec4(position, 0.0, 1.0);
    vertexColor = color_converted;
    vertexUv = uv;
}