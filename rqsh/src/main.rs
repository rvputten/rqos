// rqsh
mod app;
mod args;
mod builtin;
mod execute;
mod glob;

use app::App;

fn main() {
    // debugging
    let text = "\x1b[1;31mHello\x1b[0m";
    let _codes = ansi::Ansi::parse(text);

    let mut app = App::new();
    app.run();
}
