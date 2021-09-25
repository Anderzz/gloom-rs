#version 430 core


out vec4 color;
layout(location=1) in  vec4 newcolors;


void main()
{
    color=newcolors;
    //color = vec4(1.0f, 0.1f, 0.2f, 1.0f);
}