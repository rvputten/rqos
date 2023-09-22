uniform sampler2D texture;
uniform vec4 bg_color;
uniform vec4 fg_color;
uniform vec2 texture_size;

void main() {
    vec4 tex_color = texture2D(texture, gl_TexCoord[0].xy);
    if (tex_color == vec4(0.0, 0.0, 0.0, 1.0)) {
	vec2 one_pixel = vec2(1.0, 0.0) / texture_size;
	vec4 right_color = texture2D(texture, gl_TexCoord[0].xy + one_pixel);
	if (right_color.r > 0.0) {
	    gl_FragColor = mix(bg_color, fg_color, right_color.r);
	} else {
	    gl_FragColor = bg_color;
	}
    } else {
	gl_FragColor = mix(bg_color, fg_color, tex_color.r);
    }
}