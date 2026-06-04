#version 310 es
precision mediump float;

uniform float u_time;
uniform vec2  u_resolution;
uniform vec4  u_audio;
uniform float u_audio_mid;
uniform float u_beat;
uniform float u_bpm;
uniform float u_trigger;
uniform mediump sampler2D u_audio_history;

uniform float u_param0;
uniform float u_param1;
uniform float u_param2;
uniform float u_param3;
uniform float u_param4;
uniform float u_param5;
uniform float u_param6;
uniform float u_param7;

uniform mediump sampler2D u_source_0;
uniform mediump sampler2D u_prev;
uniform vec2              u_source_0_size;

in  vec2 v_uv;
out vec4 frag_color;

// Compatibility shims so GLSL ES 1.00-style shader bodies (gl_FragColor,
// texture2D) compile unchanged under the 3.10 profile. The prelude is the
// single portability layer; shader files stay authored in the 1.00 idiom.
#define gl_FragColor frag_color
#define texture2D    texture
