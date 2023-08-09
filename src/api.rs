use pgrx::prelude::*;
use pgrx::spi::SpiTupleTable;

use crate::executor::JobMessage;
use pgmq::PGMQueueExt;

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
pub async fn get_job(queue: &PGMQueueExt, timeout: i32) -> Option<pgmq::Message<JobMessage>> {
    match queue.read::<JobMessage>("pg_later_jobs", timeout).await {
        Ok(Some(job)) => Some(job),
        Ok(None) => None,
        Err(e) => {
            log!("pg-later: error, {:?}", e);
            None
        }
    }
}

pub fn delete_from_queue(msg_id: i64) -> Result<(), spi::Error> {
    let del = format!("select pgmq_delete('pg_later_jobs', {msg_id})");
    let _: bool = Spi::get_one(&del)?.expect("failed to send message to queue");
    Ok(())
}
