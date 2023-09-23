#version <version>

#ifdef GL_ES
precision mediump float;
#endif

uniform mat4 view;
uniform mat4 proj;

layout (location = 0) in vec2 position;
layout (location = 1) in uvec4 color;
layout (location = 2) in vec2 uv;

out vec4 vertexColor;
out vec2 vertexUv;

void main()
{
    gl_Position = proj * view * vec4(position, 0.0, 1.0);
    
    vertexColor = vec4(color) / 255.0;
    vertexUv = uv;
}