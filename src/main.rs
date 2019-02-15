mod cli;
use cli::parse_commandline;
mod app;
use app::app;

fn main() {
    let args = parse_commandline();
    app.with(|a| a.run(args));
}
