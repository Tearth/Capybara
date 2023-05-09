#version <version>

#ifdef GL_ES
precision mediump float;
#endif

in vec2 vertexUv;
in vec4 vertexColor;
out vec4 fragmentColor;

uniform sampler2D textureSampler;

void main()
{
    fragmentColor = texture(textureSampler, vertexUv) * vertexColor;
}