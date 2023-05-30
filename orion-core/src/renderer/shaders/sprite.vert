#version <version>

#ifdef GL_ES
precision mediump float;
#endif

uniform mat4 view;
uniform mat4 proj;

layout (location = 0) in vec2 position;
layout (location = 1) in vec2 offset;
layout (location = 2) in vec2 anchor;
layout (location = 3) in float rotation;
layout (location = 4) in vec2 size;
layout (location = 5) in uvec4 color;
layout (location = 6) in vec4 uv;

out vec4 vertexColor;
out vec2 vertexUv;

void main()
{
    float r_cos = cos(rotation);
    float r_sin = sin(rotation);

    vec2 position_adjusted = position - anchor;
    vec2 rotated = vec2(
        position_adjusted.x * r_cos - position_adjusted.y * r_sin, 
        position_adjusted.y * r_cos + position_adjusted.x * r_sin
    );

    gl_Position = proj * view * vec4(rotated * size + offset, 0.0, 1.0);

    vertexColor = vec4(
        float(color.x) / 255.0, 
        float(color.y) / 255.0, 
        float(color.z) / 255.0, 
        float(color.w) / 255.0
    );
    vertexUv = uv.xy + uv.zw - uv.zw * position;
}