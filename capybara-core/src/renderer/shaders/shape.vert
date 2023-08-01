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
    
    vertexColor = vec4(
        float(color.x) / 255.0, 
        float(color.y) / 255.0, 
        float(color.z) / 255.0, 
        float(color.w) / 255.0
    );
    vertexUv = uv;
}