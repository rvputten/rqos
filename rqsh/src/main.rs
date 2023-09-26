// rqsh
mod app;
mod builtin;
mod execute;
mod glob;

use app::App;

fn main() {
    let mut app = App::new();
    app.run();
}
