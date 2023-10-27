uniform sampler2D texture;
uniform vec4 bg_color;
uniform vec4 fg_color;
uniform vec2 atlas_size;
uniform vec2 sprite_position;
uniform vec2 sprite_size;
uniform float bold_offset;

void main() {
    vec4 tex_color = texture2D(texture, gl_TexCoord[0].xy);
    if (tex_color == vec4(0.0, 0.0, 0.0, 1.0)) {
	if (bold_offset > 0.0) {
	    vec2 one_pixel = vec2(bold_offset, 0.0) / atlas_size;
            vec2 sprite_end = (sprite_position + sprite_size)/atlas_size;
	    vec4 right_color = texture2D(texture, gl_TexCoord[0].xy + one_pixel);
	    bool is_right_pixel_outside = (gl_TexCoord[0].x + one_pixel.x >= sprite_end.x);
	    if ((!is_right_pixel_outside) && (right_color.r > 0.0)) {
		gl_FragColor = mix(bg_color, fg_color, right_color.r);
	    } else {
	        gl_FragColor = bg_color;
	    }
	} else {
	    gl_FragColor = bg_color;
	}
    } else {
	gl_FragColor = mix(bg_color, fg_color, tex_color.r);
    }
}
