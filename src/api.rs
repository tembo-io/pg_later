/// The user facing API
///
use pgrx::prelude::*;
use pgrx::spi::SpiTupleTable;

/// send a query to be executed by the next available worker
#[pg_extern]
pub fn exec(query: &str) -> Result<i64, spi::Error> {
    let msg = serde_json::json!({
        "query": query.replace('\'', "''").replace(';', ""),
    });
    let enqueue = format!("select pgmq_send('pg_later_jobs', '{msg}'::jsonb)");
    log!("pg-later: sending query to queue: {query}");
    let msg_id: i64 = Spi::get_one(&enqueue)?.expect("failed to send message to queue");
    Ok(msg_id)
}

// get the resultset of a previously submitted query
#[pg_extern]
fn fetch_results(job_id: i64) -> Result<Option<pgrx::JsonB>, spi::Error> {
    let query = format!(
        "select * from pgmq_pg_later_results
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
