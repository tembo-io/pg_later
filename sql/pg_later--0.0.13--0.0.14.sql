DROP function pglater."exec";

CREATE  FUNCTION pglater."exec"(
        "query" TEXT,
        "default" INT DEFAULT 0
) RETURNS INT
STRICT
LANGUAGE c
AS 'MODULE_PATHNAME', 'exec_wrapper';
