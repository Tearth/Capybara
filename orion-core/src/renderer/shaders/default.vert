#version <version>

#ifdef GL_ES
precision mediump float;
#endif

uniform mat4 model;
uniform mat4 view;
uniform mat4 proj;

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec4 aColor;
layout (location = 2) in vec2 aTexCoord;

out vec4 vertexColor;
out vec2 TexCoord;

void main()
{
    gl_Position = proj * view * vec4(aPos, 1.0);
    vertexColor = aColor;
    TexCoord = aTexCoord;
}