#version <version>

#ifdef GL_ES
precision mediump float;
#endif

in vec2 vertexUv;
in vec4 vertexColor;
out vec4 fragmentColor;

uniform sampler2D textureSampler;

vec3 fromLinear(vec3 rgb) {
    vec3 a = 12.92 * rgb;
    vec3 b = 1.055 * pow(rgb, vec3(1.0 / 2.4)) - 0.055;
    vec3 c = step(vec3(0.0031308), rgb);
    return mix(a, b, c);
}

vec4 fromLinear(vec4 rgba) {
    return vec4(fromLinear(rgba.rgb), rgba.a);
}

void main()
{
    fragmentColor = fromLinear(texture(textureSampler, vertexUv)) * vertexColor;
}