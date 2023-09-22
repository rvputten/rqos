use sfml::graphics::RenderWindow;
use sfml::system::Vector2i;
use sfml::window::Style;

mod font_editor;

fn main() {
    let desktop_mode = sfml::window::VideoMode::desktop_mode();
    let screen_width = desktop_mode.width;
    let screen_height = desktop_mode.height;

    let font_name = "font";
    let font_size = Vector2i::new(20, 40);
    let edit_char_scale: i32 = 40;
    let font_scale = 2;
    let sample_text_scale = 1;

    let window_width = edit_char_scale * font_size.x + 80 * font_size.x * sample_text_scale;
    let window_height = edit_char_scale * font_size.y
        + 16 * font_size.y * font_scale
        + 3 * font_size.y * font_scale;
    let (window_pos_x, window_pos_y) = (
        ((screen_width as i32) - (window_width / 2)),
        ((screen_height as i32 / 2) - (window_height / 2)),
    );

    let mut window = RenderWindow::new(
        (window_width as u32, window_height as u32),
        "Font Editor",
        Style::CLOSE,
        &Default::default(),
    );
    window.set_position(Vector2i::new(window_pos_x, window_pos_y));
    window.set_vertical_sync_enabled(true);

    // make a new font
    // downscaling:
    // convert resources/font_10x16.png -resize 80x128 resources/font_5x8.png
    // upscaling:
    // convert resources/font_10x16.png -sample 160x320! resources/font_10x20.png
    // (! to no preserve aspect ratio)

    font_editor::Editor::edit(
        font_name,
        font_size,
        edit_char_scale,
        font_scale,
        sample_text_scale,
        window,
    );
}
