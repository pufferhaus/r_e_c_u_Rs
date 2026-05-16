// Radial reflect with adjustable wedge count.
void main() {
    vec2 p = v_uv - 0.5;
    float r = length(p);
    float a = atan(p.y, p.x);
    float wedges = max(2.0, floor(u_param0 * 16.0) + 2.0);
    float sector = 6.2831853 / wedges;
    a = mod(a + u_param1 * 3.14, sector);
    a = abs(a - sector * 0.5);
    vec2 q = vec2(cos(a), sin(a)) * r + 0.5;
    gl_FragColor = texture2D(u_source_0, q);
}
