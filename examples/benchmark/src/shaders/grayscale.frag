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
    vec4 color = texture(textureSampler, vertexUv) * vertexColor;
    float gray = dot(color.xyz, vec3(0.2126, 0.7152, 0.0722));
    float gammaGray = sqrt(gray);

    fragmentColor = vec4(gammaGray, gammaGray, gammaGray, color.w);
}