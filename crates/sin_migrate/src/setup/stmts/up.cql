CREATE KEYSPACE IF NOT EXISTS metadata WITH REPLICATION = {'class': 'SimpleStrategy'};

CREATE TABLE IF NOT EXISTS metadata.migration_metadata (
       version text,
       time timestamp,
       is_run boolean,
       PRIMARY KEY(version)
);
