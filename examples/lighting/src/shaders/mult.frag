#version <version>

#ifdef GL_ES
precision mediump float;
#endif

in vec2 vertexUv;
in vec4 vertexColor;
out vec4 fragmentColor;

uniform vec2 resolution;
uniform float directions;
uniform float quality;
uniform float size;
uniform sampler2D mainSampler;
uniform sampler2D lightSampler;

void main()
{
    const float TAU = 6.28318530718;
   
    vec2 radius = size/resolution.xy;
    vec4 lightColor = texture(lightSampler, vertexUv);

    for (float d = 0.0; d < TAU; d += TAU / directions)
    {
		for (float q = 1.0 / quality; q <= 1.0; q += 1.0 / quality)
        {
			lightColor += texture(lightSampler, vertexUv + vec2(cos(d), sin(d)) * radius * q);		
        }
    }

    lightColor /= quality * directions;
    fragmentColor = lightColor * texture(mainSampler, vertexUv) * vertexColor;
}