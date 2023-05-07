#version <version>

#ifdef GL_ES
precision mediump float;
#endif

out vec4 FragColor; 
in vec2 TexCoord;
in vec4 vertexColor;

uniform sampler2D ourTexture;

void main()
{
    FragColor = texture(ourTexture, TexCoord) * vertexColor;
}