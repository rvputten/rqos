use sfml::graphics::RenderWindow;
use sfml::system::Vector2i;
use sfml::window::Style;

mod font;
mod font_editor;

fn main() {
    let desktop_mode = sfml::window::VideoMode::desktop_mode();
    let screen_width = desktop_mode.width;
    let screen_height = desktop_mode.height;
    let (window_width, window_height) = (screen_width / 4, screen_height / 4);
    let (window_pos_x, window_pos_y) = (
        ((screen_width / 4) - (window_width / 8)) as i32,
        ((screen_height * 3 / 4) - (window_height / 8)) as i32,
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
    let grid_size: i32 = 32;
    let scale = 4;
    font_editor::Editor::edit(font_name, font_size, grid_size, scale, window);
}
