use pgrx::prelude::*;
use pgrx::spi;
use std::env;

use std::time::Duration;

use pgrx::bgworkers::*;

use crate::api::{delete_from_queue, get_job, query_to_json};

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

    let db = from_env_default("PG_LATER_DATABASE", "postgres");
    log!("Connecting background worker to database: {}", db);
    BackgroundWorker::connect_worker_to_spi(Some(&db), None);

    log!("Starting BG Workers {}", BackgroundWorker::get_name(),);
    // poll at 10s or on a SIGTERM
    while BackgroundWorker::wait_latch(Some(Duration::from_secs(5))) {
        if BackgroundWorker::sighup_received() {
            // TODO: reload config
        }
        if !ready() {
            log!("pg-later: not ready");
            continue;
        }
        let _result: Result<(), pgrx::spi::Error> = BackgroundWorker::transaction(|| {
            let job: Option<(i64, String)> = get_job(120);
            match job {
                Some((job_id, query)) => {
                    log!("pg-later: executing job: {}", query);
                    let _exec_job = exec_job(job_id, &query);
                    delete_from_queue(job_id)?;
                }
                None => {
                    log!("pg-later: no jobs in queue");
                }
            }
            Ok(())
        });
    }

    log!(
        "Goodbye from inside the {} BGWorker! ",
        BackgroundWorker::get_name()
    );
}

fn ready() -> bool {
    let exists: bool = BackgroundWorker::transaction(|| {
        Spi::get_one::<bool>(
            "SELECT EXISTS (
            SELECT 1
            FROM pg_tables
            WHERE tablename = 'pgmq_pg_later_jobs'
        );",
        )
        .expect("failed to interface with SPI")
        .expect("select 1 returned None")
    });
    exists
}

// executes a query and writes results to a results queue
fn exec_job(job_id: i64, query: &str) -> Result<(), spi::Error> {
    let result_message = match query_to_json(query) {
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
    let enqueue = format!("select pgmq_send('pg_later_results', '{result_message}')");
    let _: i64 = Spi::get_one(&enqueue)?.expect("query did not return message id");
    Ok(())
}

fn from_env_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_owned())
}
