// TODO: Implement wave file parser using nom (in a subproject)
// TODO: Implement hot reloading

mod app;
use app::app;

fn main() {
    app.with(|a| {
        a.parse_commandline();
        a.run();
    });
}
