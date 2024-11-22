use crate::{consts, error};
use std::{io::Write, path::PathBuf};

pub fn generate_migration_file(args: &clap::ArgMatches) -> error::CustomResult<()> {
    let name = args
        .get_one::<String>("GENERATE_NAME")
        .expect("This should not be null should be prevented by clap");

    let dir = args
        .get_one::<PathBuf>("MIGRATION_DIR")
        .cloned()
        .unwrap_or(PathBuf::from("./migrations"));

    let version = generate_version();
    let migration_folder = dir.join(format!("{}_{}", version, name));

    // Create directory in the format {timestamp}_{name}
    std::fs::create_dir_all(&migration_folder)?;

    let mut up = std::fs::File::create_new(migration_folder.join(consts::UP_CQL))?;
    let mut down = std::fs::File::create_new(migration_folder.join(consts::DOWN_CQL))?;

    up.write_all(consts::PLACEHOLDER.as_bytes())?;
    down.write_all(consts::PLACEHOLDER.as_bytes())?;

    Ok(())
}

fn generate_version() -> String {
    time::OffsetDateTime::now_utc()
        .unix_timestamp_nanos()
        .to_string()
}
