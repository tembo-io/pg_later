/// The user facing API
///
use pgrx::prelude::*;
use pgrx::spi::SpiTupleTable;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

#[pg_extern]
fn init() -> Result<bool, spi::Error> {
    let setup_queries = [
        "select pgmq.create_non_partitioned('pg_later_jobs')",
        "select pgmq.create_non_partitioned('pg_later_results')",
    ];
    for q in setup_queries {
        let ran: Result<_, spi::Error> = Spi::connect(|mut c| {
            let _ = c.update(q, None, None)?;
            Ok(())
        });

        ran?
    }
    Ok(true)
}

/// send a query to be executed by the next available worker
#[pg_extern]
pub fn exec(query: &str, delay: default!(i64, 0)) -> Result<i64, spi::Error> {
    let prepared_query = query.replace('\'', "''").replace(';', "");
    let dialect = PostgreSqlDialect {}; // Use PostgreSqlDialect for PostgreSQL
    let parse_result = Parser::parse_sql(&dialect, &prepared_query);
    parse_result.expect("Query parsing failed, please submit a valid query");
    let msg = serde_json::json!({
        "query": prepared_query,
    });
    let enqueue = format!("select pgmq.send('pg_later_jobs', '{msg}'::jsonb, {delay})");
    log!("pg-later: sending query to queue: {}", enqueue);

    let msg_id: i64 = Spi::get_one(&enqueue)?.expect("failed to send message to queue");
    Ok(msg_id)
}

// get the resultset of a previously submitted query
#[pg_extern]
fn fetch_results(job_id: i64) -> Result<Option<pgrx::JsonB>, spi::Error> {
    let query = format!(
        "select * from pgmq.q_pg_later_results
        where message->>'job_id' = '{job_id}'
        "
    );
    let results: Result<Option<pgrx::JsonB>, spi::Error> = Spi::connect(|mut client| {
        let mut tup_table: SpiTupleTable = client.update(&query, None, None)?;
        if let Some(row) = tup_table.next() {
            let message = row["message"].value::<pgrx::JsonB>()?.expect("no message");
            return Ok(Some(message));
        }
        Ok(None)
    });
    let query_resultset = match results {
        Ok(Some(r)) => r,
        Ok(None) => {
            return Ok(None);
        }
        _ => {
            return Err(spi::Error::CursorNotFound(
                "pg-later: failed to execute query".to_owned(),
            ));
        }
    };
    Ok(Some(query_resultset))
}
