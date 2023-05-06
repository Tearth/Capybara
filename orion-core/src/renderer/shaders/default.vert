#version <version>

#ifdef GL_ES
precision mediump float;
#endif

uniform mat4 model;
uniform mat4 view;
uniform mat4 proj;

layout (location = 0) in vec3 aPos;
out vec4 vertexColor;

void main()
{
    gl_Position = proj * view * vec4(aPos, 1.0);
    vertexColor = vec4(0.5, 0.5, 0.5, 1.0);
}