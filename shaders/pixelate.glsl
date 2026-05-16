// Block-quantize uv before sampling. u_param0 in [0, 1] → block edge px.
void main() {
    float block = max(1.0, u_param0 * 64.0);
    vec2 px = u_resolution / block;
    vec2 quant = floor(v_uv * px) / px;
    gl_FragColor = texture2D(u_source_0, quant);
}
