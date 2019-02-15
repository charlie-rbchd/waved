pub struct CommandLineArgs {
    pub files: Vec<String>,
}

pub fn parse_commandline() -> CommandLineArgs {
    let matches = clap::App::new("myprog")
        .arg(clap::Arg::with_name("files").multiple(true))
        .get_matches();
    
    let files = matches.values_of("files").unwrap_or_default()
        .map(|s| s.to_string())
        .collect();

    CommandLineArgs { files }
}
