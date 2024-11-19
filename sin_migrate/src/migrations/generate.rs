use crate::consts;
use std::{io::Write, path::PathBuf};

pub fn generate_migration_file(args: &clap::ArgMatches) -> std::io::Result<()> {
    let name = args
        .get_one::<String>("GENERATE_NAME")
        .expect("This should not be null should be prevented by clap");

    let dir = args
        .get_one::<PathBuf>("MIGRATION_DIR")
        .cloned()
        .unwrap_or(PathBuf::from("./migrations"));

    // Check if the migration director exists, should be created beforehand by the user
    if !std::fs::exists(&dir)? {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Migration directory not found",
        ));
    }

    let version = generate_version();
    let migration_folder = dir.join(format!("{}_{}", version, name));

    // Create directory in the format {timestamp}_{name}
    std::fs::create_dir(&migration_folder)?;

    let mut up = std::fs::File::create_new(migration_folder.join(consts::UP_CQL))?;
    let mut down = std::fs::File::create_new(migration_folder.join(consts::DOWN_CQL))?;

    up.write_all(b"// Put your CQL here")?;
    down.write_all(b"// Put your CQL here")?;

    Ok(())
}

fn generate_version() -> String {
    time::OffsetDateTime::now_utc()
        .unix_timestamp_nanos()
        .to_string()
}
