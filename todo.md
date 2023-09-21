Proposed GUI usage
==================
```
use gui::{Gui, WindowId, TextFieldId};

struct App {
  gui: Gui,
  win_id: WindowId,
  text_field_id: TextFieldId,
}

impl App {
  pub fn new() -> App {
    let mut gui = Gui::new()
      .title("My App")
      .size(800, 600)
      .font("font_10x20.png")
      .create();

    let win_id = gui.window()
      .title("My Window")
      .position(0, 0)
      .size(-1, -1)
      .create();

    let text_field_id = gui.text_field()
      .position(0, 0)
      .size(-1, -1)
      .text("Timer: 0")
      .create();

    let timer_id = gui.timer()
      .interval(1000)
      .create();

    App {
      gui: gui,
      win_id: win_id,
      text_field_id: text_field_id,
    }
  }

  pub fn run(&mut self) {
    loop {
      let event = self.gui.poll_event();
      match event {
        Event::WindowClose(id) => {
          if id == self.win_id {
            break;
          }
        }
        Event::Timer(id) => {
          if id == self.timer_id {
            self.gui.set_text(self.text_field_id, format!("Timer: {}", self.timer_id));
          }
        }

      }
    }
  }


