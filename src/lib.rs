use pgrx::prelude::*;

pgrx::pg_module_magic!();

mod api;
mod bgw;

extension_sql!(
    "
    CREATE EXTENSION IF NOT EXISTS pgmq CASCADE;
    CREATE TABLE IF NOT EXISTS pglater.later_meta (
        id serial PRIMARY KEY,
        name text NOT NULL,
        description text,
        created_at timestamp NOT NULL DEFAULT now(),
        updated_at timestamp NOT NULL DEFAULT now()
    );",
    name = "pg_later_setup",
    bootstrap,
);

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;
}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}