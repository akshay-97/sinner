mod generate;
mod run;

use crate::conn::Conn;
use std::path::PathBuf;

pub async fn run_migration(args: &clap::ArgMatches) -> std::io::Result<()> {
    match args.subcommand().expect("Invalid") {
        ("generate", args) => generate::generate_migration_file(args),
        ("run", args) => {
            let url = args
                .get_one::<String>("DATABASE_URL")
                .cloned()
                .expect("This should be prevented from the CLI");

            let keyspace = args
                .get_one::<String>("KEYSPACE")
                .cloned()
                .expect("This should be prevented from CLI");

            let conn = Conn::from_url(url, None, None, keyspace).await;

            let dir = args
                .get_one::<PathBuf>("MIGRATION_DIR")
                .cloned()
                .unwrap_or(PathBuf::from("./migrations"));

            run::run_migrations(dir, conn).await
        }
        _ => panic!("invalid"),
    }
}
