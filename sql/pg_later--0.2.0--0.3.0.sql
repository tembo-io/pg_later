DROP FUNCTION pglater."exec";

-- src/api.rs:26
-- pg_later::api::exec
CREATE  FUNCTION "exec"(
        "query" TEXT, /* &str */
        "delay" bigint DEFAULT 0, /* i64 */
        "validate" bool DEFAULT true /* bool */
) RETURNS bigint /* core::result::Result<i64, pgrx::spi::SpiError> */
STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'exec_wrapper';
/* </end connected objects> */
