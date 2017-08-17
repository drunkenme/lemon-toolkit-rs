#version 150
in vec4 v_Diffuse;
in vec4 v_Additive;
in vec2 v_Texcoord;

uniform sampler2D u_MainTex;
out vec4 color;

void main() {
    color = v_Additive + v_Diffuse * texture(u_MainTex, v_Texcoord);
}