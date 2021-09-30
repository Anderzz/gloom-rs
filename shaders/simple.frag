#version 430 core


out vec4 color;
layout(location=1) in  vec4 newcolors;
layout(location=5) in  vec3 in_normals;



void main()
{
    vec3 null = vec3(0.0, 0.0, 0.0);
    vec3 lightDirection = normalize(vec3(0.8, -0.5, 0.6));
    color = newcolors * vec4(max(null, in_normals*-lightDirection),1.0f);
}