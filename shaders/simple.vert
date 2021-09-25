#version 430 core


in vec3 position;
layout(location=1) in  vec4 color_in;
layout(location=1) out  vec4 color_out;
uniform layout(location = 3) float uni; //used for task 3
uniform layout(location = 4) mat4 transform;
mat4x4 m = mat4(1);


void main()
{
    m[0][0]=uni; //task3, but breaks if I remove :)
    gl_Position = vec4(position, 1.0f)*transform;
    color_out=color_in;

}