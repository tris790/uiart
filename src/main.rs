use app::App;

extern crate sdl2;

mod app;
mod navigation;
mod ui_bounding_box;

pub fn main() {
    let mut app = App::new(vec![]);
    app.run();
}
