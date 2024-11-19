mod cli;
mod conn;
mod consts;
mod migrations;

#[tokio::main]
async fn main() {
    let args = cli::Cli::build();

    let matches = args.command().get_matches();

    match matches.subcommand().expect("No args") {
        ("migrate", matches) => migrations::run_migration(matches).await.unwrap(),
        _ => panic!("No args"),
    }
}
