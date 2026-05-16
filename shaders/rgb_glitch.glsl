// Per-channel UV offset, time-modulated.
void main() {
    float t = u_time * (u_param3 * 4.0 + 0.1);
    vec2 ro = vec2(u_param0, 0.0) * 0.05 * sin(t);
    vec2 go = vec2(u_param1, 0.0) * 0.05 * cos(t * 1.3);
    vec2 bo = vec2(u_param2, 0.0) * 0.05 * sin(t * 0.7 + 1.7);
    float r = texture2D(u_source_0, v_uv + ro).r;
    float g = texture2D(u_source_0, v_uv + go).g;
    float b = texture2D(u_source_0, v_uv + bo).b;
    gl_FragColor = vec4(r, g, b, 1.0);
}
