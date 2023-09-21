uniform sampler2D texture;
uniform vec4 bg_color;
uniform vec4 fg_color;

void main() {
    vec4 tex_color = texture2D(texture, gl_TexCoord[0].xy);
    if (tex_color == vec4(0.0, 0.0, 0.0, 1.0)) {
        gl_FragColor = bg_color;
    } else {
        gl_FragColor = tex_color * fg_color;
	gl_FragColor = mix(bg_color, fg_color, tex_color.r);
    }
}
