// rqsh
mod app;
mod args;
mod builtin;
mod execute;
mod glob;
mod util;

use app::App;

fn main() {
    let mut app = App::new();
    app.run();
}
