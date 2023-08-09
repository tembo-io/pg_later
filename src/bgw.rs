use pgmq::{Message, PGMQueueExt};
use pgrx::bgworkers::*;
use pgrx::prelude::*;
use sqlx::postgres::PgRow;
use sqlx::Pool;
use sqlx::Postgres;
use std::time::Duration;

use crate::executor::{query_to_json, JobMessage};
use crate::util;
use anyhow::Result;
use sqlx::Row;

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

    log!(
        "Starting BG Workers {}, connection: {:?}",
        BackgroundWorker::get_name(),
        conn
    );
    // poll at 10s or on a SIGTERM
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
            // let x = sqlx::query("select to_json(t) from (select * from pgmq_pg_later_jobs))").fetch_all(&conn).await.expect("failed query");
            // for row in x {
            //     let r: serde_json::Value = row.into();
            //     log!("pg-later: got row: {:?}", row.into());
            // }

            let msg: String = sqlx::query_scalar("select CURRENT_USER")
                .fetch_one(&conn)
                .await
                .expect("failed to read from queue");
            log!("current_db {}", msg);
            // let msg = fetch_one_message::<JobMessage>("select * from public.pgmq_read('pg_later_jobs', 1, 1)", &conn).await.expect("failed to read from queue");
            // let msg = queue.read::<serde_json::Value>("pg_later_jobs", 30).await.expect("failed to read from queue");

            // let row: (serde_json::Value,) = sqlx::query_as(&format!(
            //     "select to_jsonb(t) as results from (select message from pgmq_pg_later_jobs) t"
            // ))
            // let row: (serde_json::Value,) = sqlx::query_as(&format!(
            //     "select * from pgmq_read('pg_later_jobs'::text, 1::integer,1::integer)"
            // ))
            // .fetch_one(&conn)
            // .await
            // .expect("failed to fetch");
            // log!("pg-later: got message: {:?}", row.0);
            match queue.read::<JobMessage>("pg_later_jobs", 30).await {
                Ok(Some(job)) => {
                    log!("pg-later: executing job: {}", job.message.query);
                    let _ = exec_job(job.msg_id, &job.message.query, &conn);
                    // for now, always delete whether the incoming job succeeded or failed
                    queue
                        .archive("pg_later_jobs", job.msg_id)
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

pub async fn fetch_one_message<T: for<'de> serde::Deserialize<'de>>(
    query: &str,
    connection: &Pool<Postgres>,
) -> Result<Option<Message<T>>> {
    log!("executing query: {}", query);
    let row: Result<PgRow, sqlx::Error> = sqlx::query(query).fetch_one(connection).await;
    match row {
        Ok(row) => {
            // happy path - successfully read a message
            let raw_msg = row.get("message");
            let parsed_msg = serde_json::from_value::<T>(raw_msg);
            match parsed_msg {
                Ok(parsed_msg) => Ok(Some(Message {
                    msg_id: row.get("msg_id"),
                    vt: row.get("vt"),
                    read_ct: row.get("read_ct"),
                    enqueued_at: row.get("enqueued_at"),
                    message: parsed_msg,
                })),
                Err(e) => {
                    log!("error");
                    Ok(None)
                }
            }
        }
        Err(sqlx::error::Error::RowNotFound) => Ok(None),
        Err(e) => Err(e)?,
    }
}

async fn ready(conn: &Pool<Postgres>) -> bool {
    sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1
            FROM pg_tables
            WHERE tablename = 'pgmq_pg_later_jobs'
        );",
    )
    .fetch_one(conn)
    .await
    .expect("failed")
}

// executes a query and writes results to a results queue
async fn exec_job(job_id: i64, query: &str, conn: &Pool<Postgres>) -> Result<(), spi::Error> {
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

    let enqueue: String = format!("select pgmq_send('pg_later_results', '{result_message}')");
    let _: i64 = Spi::get_one(&enqueue)?.expect("query did not return message id");
    Ok(())
}
