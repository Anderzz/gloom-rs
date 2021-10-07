#version 430 core


in vec3 position;
layout(location=1) in  vec4 color_in;
layout(location=1) out  vec4 color_out;
layout(location=5) in  vec3 normals_in;
layout(location=5) out  vec3 normals_out;
uniform layout(location = 4) mat4 transform;
uniform layout(location = 2) mat4 modelmat;


void main()
{
    gl_Position = transform * vec4(position, 1.0f);
    color_out = color_in;
    normals_out = normalize(mat3(modelmat) * normals_in);
    //normals_out=normals_in;


}