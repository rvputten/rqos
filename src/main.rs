use sfml::graphics::{Color, PrimitiveType, RenderTarget, RenderWindow, Vertex};
use sfml::system::{Vector2f, Vector2i};
use sfml::window::{Event, Style};

fn main() {
    // user settings
    let font_size = Vector2i::new(12, 16);
    let grid_size = Vector2i::new(32, 32);
    let grid_offset = Vector2i::new(10, 10);

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

    let grid_pos = |x: i32, y: i32, color: Color| {
        Vertex::with_pos_color(
            Vector2f::new(
                (x * grid_size.x + grid_offset.x) as f32,
                (y * grid_size.y + grid_offset.y) as f32,
            ),
            color,
        )
    };

    let mut grid = Vec::new();

    for x in 0..=font_size.x {
        grid.push(vec![
            grid_pos(x, 0, Color::BLACK),
            grid_pos(x, font_size.y, Color::BLACK),
        ]);
    }

    for y in 0..=font_size.y {
        grid.push(vec![
            grid_pos(0, y, Color::BLACK),
            grid_pos(font_size.x, y, Color::BLACK),
        ]);
    }

    while window.is_open() {
        while let Some(event) = window.poll_event() {
            if event == Event::Closed {
                window.close();
            }
        }

        window.clear(Color::WHITE);
        for v in &grid {
            window.draw_primitives(v, PrimitiveType::LINES, &Default::default());
        }
        window.display();
    }
}
