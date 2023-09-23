#version <version>

#ifdef GL_ES
precision mediump float;
#endif

uniform mat4 view;
uniform mat4 proj;

layout (location = 0) in vec2 offset;
layout (location = 1) in vec2 anchor;
layout (location = 2) in float rotation;
layout (location = 3) in vec2 size;
layout (location = 4) in uvec4 color;
layout (location = 5) in vec4 uv;

out vec4 vertexColor;
out vec2 vertexUv;

void main()
{
    vec2 position = vec2(((gl_VertexID + 1) & 3) >> 1, (gl_VertexID & 3) >> 1);
    
    vec2 p_anch = position - anchor;
    float r_sin = sin(rotation);
    float r_cos = cos(rotation);

    vec2 rotated = vec2(
        p_anch.x * size.x * r_cos - p_anch.y * size.y * r_sin, 
        p_anch.y * size.y * r_cos + p_anch.x * size.x * r_sin
    );

    gl_Position = proj * view * vec4(rotated + offset, 0.0, 1.0);

    vertexColor = vec4(color) / 255.0;
    vertexUv = uv.xy + uv.zw * vec2(position.x, 1.0 - position.y);
}