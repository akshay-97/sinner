pub const UP_CQL: &str = "up.cql";
pub const DOWN_CQL: &str = "down.cql";

pub const SIN_SETUP_UP: &str = include_str!("./setup/stmts/up.cql");
pub const SIN_SETUP_DOWN: &str = include_str!("./setup/stmts/down.cql");

pub const PLACEHOLDER: &str = "// Put your CQL here";

// TODO: Replace this with the implementation in the main library
pub const SIN_CHECK_MIGRATE: &str = "SELECT version FROM metadata.migration_metadata LIMIT 1;";
