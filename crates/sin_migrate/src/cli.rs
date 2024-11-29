use clap::{Arg, Command};

pub struct Cli {
    command: Command,
}

impl Cli {
    pub fn command(self) -> Command {
        self.command
    }

    pub fn build() -> Self {
        let command = Command::new("Run migrations")
            .about("Run, generate, undo, redo CQl migrations")
            .arg(
                Arg::new("MIGRATION_DIR")
                    .long("dir")
                    .help("Directory where the migration files are")
                    .num_args(1)
                    .global(true)
                    .value_parser(clap::value_parser!(std::path::PathBuf)),
            )
            .subcommand(
                Command::new("migrate")
                    .subcommand(
                        Command::new("generate").arg(
                            Arg::new("GENERATE_NAME")
                                .index(1)
                                .num_args(1)
                                .required(true)
                                .help("Name of the migration"),
                        ),
                    )
                    .subcommand(
                        Command::new("run")
                            .arg(
                                Arg::new("DATABASE_URL")
                                    .long("url")
                                    .required(true)
                                    .help("Database connection URL"),
                            )
                            .arg(Arg::new("DATABASE_USERNAME").long("user").short('u'))
                            .arg(Arg::new("DATBASE_PASSWORD").long("password").short('p'))
                            .arg(
                                Arg::new("KEYSPACE")
                                    .long("keyspace")
                                    .short('k')
                                    .required(true),
                            ),
                    )
                    .subcommand(
                        Command::new("redo")
                            .arg(
                                Arg::new("DATABASE_URL")
                                    .long("url")
                                    .required(true)
                                    .help("Database connection URL"),
                            )
                            .arg(Arg::new("DATABASE_USERNAME").long("user").short('u'))
                            .arg(Arg::new("DATBASE_PASSWORD").long("password").short('p'))
                            .arg(
                                Arg::new("KEYSPACE")
                                    .long("keyspace")
                                    .short('k')
                                    .required(true),
                            ),
                    )
                    .subcommand(
                        Command::new("undo")
                            .arg(
                                Arg::new("DATABASE_URL")
                                    .long("url")
                                    .required(true)
                                    .help("Database connection URL"),
                            )
                            .arg(Arg::new("DATABASE_USERNAME").long("user").short('u'))
                            .arg(Arg::new("DATBASE_PASSWORD").long("password").short('p'))
                            .arg(
                                Arg::new("KEYSPACE")
                                    .long("keyspace")
                                    .short('k')
                                    .required(true),
                            ),
                    ),
            );
        Self { command }
    }
}
