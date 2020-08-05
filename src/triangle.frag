#version 330 core

in VS_OUTPUT {
    vec3 Color;
    vec2 TexCoord;
} IN;

out vec4 Color;

uniform sampler2D tex;

void main()
{
    // Color = vec4(IN.Color, 1);
    Color = texture(tex, IN.TexCoord);
}