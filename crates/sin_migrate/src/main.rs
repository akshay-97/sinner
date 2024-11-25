use colored::Colorize;

mod cli;
mod conn;
mod consts;
mod error;
mod migrations;

mod setup;

async fn inner_main() -> error::CustomResult<()> {
    let args = cli::Cli::build();
    let matches = args.command().get_matches();

    match matches.subcommand().expect("No args") {
        ("migrate", matches) => migrations::run_migration(matches).await,
        _ => panic!("No args"),
    }
}

#[tokio::main]
async fn main() {
    if let Err(err) = inner_main().await {
        let err = err.to_string().red();

        eprintln!("{}", err)
    }
}
