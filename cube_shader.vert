#version 100

attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
attribute vec4 normal;

varying lowp vec2 uv;
varying lowp vec4 color;
varying lowp vec3 vert_normal;
varying lowp vec3 vert_pos;

uniform mat4 Model;
uniform mat4 Projection;
uniform mat4 CubeRotation;
uniform mat4 XRotation[3];
uniform mat4 YRotation[3];
uniform mat4 ZRotation[3];

float modI(float a,float b) {
    float m=a-floor((a+0.5)/b)*b;
    return floor(m+0.5);
}

ivec3 get_3d_index(int idx) {
    if (idx >= 13) {
        idx += 1;
    }

    int z = idx / (3 * 3);
    idx -= z * 3 * 3;
    int y = idx / 3;
    int x = int(modI(float(idx), 3.0));
    return ivec3(x, y, z);
}

void main() {
    int index = int(normal.w);
    ivec3 layer_idx = get_3d_index(index);
    mat4 AnimMat = XRotation[layer_idx.x] * YRotation[layer_idx.y] * ZRotation[layer_idx.z];

    vec4 pos = CubeRotation * AnimMat * vec4(position, 1);
    vert_pos = pos.xyz;
    gl_Position = Projection * pos;
    color = color0 / 255.0;
    vert_normal = (CubeRotation * AnimMat * normal).xyz;
    uv = texcoord;
}
