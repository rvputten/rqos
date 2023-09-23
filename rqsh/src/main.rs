// rqsh
mod app;
mod builtin;

use app::App;

fn main() {
    let mut app = App::new();
    app.run();
}
