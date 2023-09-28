use pgmq::PGMQueueExt;
use pgrx::bgworkers::*;
use pgrx::prelude::*;
use sqlx::Pool;
use sqlx::Postgres;
use std::time::Duration;

use crate::executor::{query_to_json, Job};
use crate::util;
use anyhow::Result;

pub const PGMQ_QUEUE_NAME: &str = "pg_later_jobs";

#[pg_guard]
pub extern "C" fn _PG_init() {
    BackgroundWorkerBuilder::new("PG Later Background Worker")
        .set_function("background_worker_main")
        .set_library("pg_later")
        .enable_spi_access()
        .load();
}

#[pg_guard]
#[no_mangle]
pub extern "C" fn background_worker_main(_arg: pg_sys::Datum) {
    BackgroundWorker::attach_signal_handlers(SignalWakeFlags::SIGHUP | SignalWakeFlags::SIGTERM);

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    let (conn, queue) = runtime.block_on(async {
        let conn = util::get_pg_conn()
            .await
            .expect("failed to connect to database");
        let queue = PGMQueueExt::new_with_pool(conn.clone())
            .await
            .expect("failed to init db connection");
        (conn, queue)
    });

    log!("Starting BG Workers {}", BackgroundWorker::get_name());

    while BackgroundWorker::wait_latch(Some(Duration::from_secs(5))) {
        if BackgroundWorker::sighup_received() {
            // TODO: reload config
        }
        let rdy = runtime.block_on(async { ready(&conn).await });
        if !rdy {
            log!("pg-later: not ready");
            continue;
        }

        runtime.block_on(async {
            match queue.read::<Job>(PGMQ_QUEUE_NAME, 100).await {
                Ok(Some(msg)) => {
                    let job = msg.message;
                    log!("pg-later: executing job: {}", job.query);
                    let result_message = exec_job(msg.msg_id, &job.query, &conn)
                        .await
                        .expect("failed to get result");
                    let msg_id = queue
                        .send("pg_later_results", &result_message)
                        .await
                        .expect("failed to send result");
                    log!("pg-later: sent message id: {}", msg_id);

                    // for now, always delete whether the incoming job succeeded or failed
                    // the job is reported with its status. in future, support some sort of retry
                    queue
                        .archive(PGMQ_QUEUE_NAME, msg.msg_id)
                        .await
                        .expect("failed to archive job");
                }
                Ok(None) => {
                    log!("pg-later: no jobs in queue")
                }
                Err(e) => {
                    log!("pg-later: error, {:?}", e);
                }
            }
        });
    }
    log!("shutting down {} BGWorker", BackgroundWorker::get_name());
}

async fn ready(conn: &Pool<Postgres>) -> bool {
    sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1
            FROM pg_tables
            WHERE schemaname = 'pgmq'
            WHERE tablename = 'q_pg_later_jobs'
        );",
    )
    .fetch_one(conn)
    .await
    .expect("failed")
}

// executes a query and writes results to a results queue
async fn exec_job(job_id: i64, query: &str, conn: &Pool<Postgres>) -> Result<serde_json::Value> {
    let result_message = match query_to_json(query, conn).await {
        Ok(json) => {
            serde_json::json!({
                "status": "success",
                "job_id": job_id,
                "query": query.replace('\'', "''"),
                "result": json,
            })
        }
        Err(e) => {
            log!("Error: {:?}", e);
            serde_json::json!({
                "status": "failure",
                "job_id": job_id,
                "query": query.replace('\'', "''"),
                "result": format!("error: {e}"),
            })
        }
    };
    Ok(result_message)
}
