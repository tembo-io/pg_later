use pgrx::prelude::*;
use pgrx::spi::SpiTupleTable;

#[pg_extern]
fn init() -> Result<bool, spi::Error> {
    let setup_queries = [
        "select pgmq_create_non_partitioned('pg_later_jobs')",
        "select pgmq_create_non_partitioned('pg_later_results')",
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

// gets a job query from the queue
pub fn get_job(timeout: i64) -> Option<(i64, String)> {
    let job = match poll_queue(timeout) {
        Ok(Some(j)) => Some(j),
        Ok(None) => None,
        Err(e) => {
            log!("pg-later: error, {:?}", e);
            None
        }
    };
    match job {
        Some(job) => {
            let msg_id = job[0].0;
            let m = serde_json::to_value(&job[0].1).expect("failed parsing jsonb");
            let q = m["query"].as_str().expect("no query").to_owned();
            Some((msg_id, q))
        }
        None => None,
    }
}

fn poll_queue(timeout: i64) -> Result<Option<Vec<(i64, pgrx::JsonB)>>, spi::Error> {
    let mut results: Vec<(i64, pgrx::JsonB)> = Vec::new();

    let query =
        format!("select msg_id, message from public.pgmq_read('pg_later_jobs' ,{timeout}, 1)");

    let _: Result<(), spi::Error> = Spi::connect(|mut client| {
        let tup_table_handle: Result<SpiTupleTable, spi::Error> = client.update(&query, None, None);
        let tup_table = match tup_table_handle {
            Ok(t) => t,
            Err(e) => {
                log!("pg-later: error, {:?}", e);
                return Ok(());
            }
        };
        for row in tup_table {
            let msg_id = row["msg_id"].value::<i64>()?.expect("no msg_id");
            let message = row["message"].value::<pgrx::JsonB>()?.expect("no message");
            results.push((msg_id, message));
        }
        Ok(())
    });
    if results.is_empty() {
        Ok(None)
    } else {
        Ok(Some(results))
    }
}

pub fn delete_from_queue(msg_id: i64) -> Result<(), spi::Error> {
    let del = format!("select pgmq_delete('pg_later_jobs', {msg_id})");
    let _: bool = Spi::get_one(&del)?.expect("failed to send message to queue");
    Ok(())
}
