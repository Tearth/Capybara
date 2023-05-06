#version <version>

#ifdef GL_ES
precision mediump float;
#endif

out vec4 FragColor; 
in vec4 vertexColor;

void main()
{
    FragColor = vertexColor;
}