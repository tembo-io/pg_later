/// "CREATE", "ALTER", "DROP", "COMMENT", "TRUNCATE", "INSERT", "UPDATE",
/// "DELETE", "SELECT", "COPY", "BEGIN", "COMMIT", "ROLLBACK", "SAVEPOINT",
/// "RELEASE", "GRANT", "REVOKE", "SET", "RESET", "SHOW", "LISTEN", "NOTIFY",
/// "VACUUM", "ANALYZE", "EXPLAIN", "CLUSTER", "CHECKPOINT", "REINDEX",
/// "PG_TRY_ADVISORY", "PREPARE", "EXECUTE", "DEALLOCATE", "$BODY$"]

/// naive classifier of SQL statements
///
///
pub fn returns_rows(sql: &str) -> bool {
    // TODO - build a more complex statement classifier
    sql.trim().to_uppercase().starts_with("SELECT")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_returns_rows() {
        let stmt = "SELECT * FROM foo";
        assert!(returns_rows(stmt));

        let stmt = "CREATE INDEX IF NOT EXISTS my_idx ON yolo (x)";
        assert!(!returns_rows(stmt));
    }
}
