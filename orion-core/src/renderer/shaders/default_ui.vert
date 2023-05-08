#version <version>

#ifdef GL_ES
precision mediump float;
#endif

uniform mat4 proj;

layout (location = 0) in vec3 position;
layout (location = 1) in vec4 color;
layout (location = 2) in vec2 uv;
out vec4 vertexColor;
out vec2 vertexUv;

void main()
{
    gl_Position = proj * vec4(position, 1.0);
    vertexColor = color;
    vertexUv = uv;
}