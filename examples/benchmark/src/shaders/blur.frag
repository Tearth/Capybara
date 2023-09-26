#version <version>

#ifdef GL_ES
precision mediump float;
#endif

in vec2 vertexUv;
in vec4 vertexColor;
out vec4 fragmentColor;

uniform vec2 resolution;
uniform sampler2D textureSampler;

void main()
{
    const float Pi2 = 6.28318530718;
    const float Directions = 16.0;
    const float Quality = 4.0;
    const float Size = 4.0;
   
    vec2 radius = Size/resolution.xy;
    vec4 color = texture(textureSampler, vertexUv);
    
    for (float d = 0.0; d < Pi2; d += Pi2 / Directions)
    {
		for (float q = 1.0 / Quality; q <= 1.0; q += 1.0 / Quality)
        {
			color += texture(textureSampler, vertexUv + vec2(cos(d), sin(d)) * radius * q);		
        }
    }
    
    color /= Quality * Directions;
    fragmentColor = color * vertexColor;
}