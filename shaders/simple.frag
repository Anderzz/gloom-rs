#version 430 core


out vec4 color;
layout(location=1) in  vec4 newcolors;
layout(location=5) in  vec3 in_normals;


void main()
{
    color=vec4(in_normals,1.0);
    //color = vec4(1.0f, 0.1f, 0.2f, 1.0f);
}