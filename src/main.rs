// TODO: Implement wave file parser using nom (in a subproject)
// TODO: Implement hot reloading
mod cli;
use cli::parse_commandline;

mod app;
use app::app;

fn main() {
    let args = parse_commandline();
    app.with(|a| a.run(args));
}
