use sfml::graphics::RenderWindow;
use sfml::system::Vector2i;
use sfml::window::Style;

mod font;
mod font_editor;

fn main() {
    let desktop_mode = sfml::window::VideoMode::desktop_mode();
    let screen_width = desktop_mode.width;
    let screen_height = desktop_mode.height;
    let (window_width, window_height) = (screen_width / 3, screen_height / 3);
    let (window_pos_x, window_pos_y) = (
        ((screen_width) - (window_width / 2)) as i32,
        ((screen_height / 2) - (window_height / 2)) as i32,
    );

    let mut window = RenderWindow::new(
        (window_width, window_height),
        "Font Editor",
        Style::CLOSE,
        &Default::default(),
    );
    window.set_position(Vector2i::new(window_pos_x, window_pos_y));
    window.set_vertical_sync_enabled(true);

    let font_name = "font";
    let font_size = Vector2i::new(10, 16);
    let edit_char_scale: i32 = 48;
    let font_scale = 3;
    font_editor::Editor::edit(font_name, font_size, edit_char_scale, font_scale, window);
}
