use pgrx::prelude::*;
use sqlx::postgres::PgRow;
use sqlx::{Pool, Postgres, Row};

use anyhow::Result;

use crate::clf;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct JobMessage {
    pub query: String,
}

async fn exec_row_query(query: &str, conn: &Pool<Postgres>) -> Result<Vec<serde_json::Value>> {
    let mut results: Vec<serde_json::Value> = Vec::new();
    let q = format!("select to_jsonb(t) as results from ({query}) t");
    log!("pg-later: executing query: {q}");
    let rows: Vec<PgRow> = sqlx::query(&q).fetch_all(conn).await?;
    for row in rows.iter() {
        results.push(row.get("results"));
    }
    Ok(results)
}


pub async fn query_to_json(query: &str, conn: &Pool<Postgres>) -> Result<Vec<serde_json::Value>> {
    match clf::returns_rows(query) {
        true => {
            log!("row query");
            exec_row_query(query, conn).await
        }
        false => {
            log!("utility statement");
            exec_row_query(query, conn).await
        }
    }
}

// #[cfg(any(test, feature = "pg_test"))]
// #[pg_schema]
// mod tests {
//     use super::*;

//     #[pg_test]
//     fn test_json() {
//         let q = query_to_json("select 1").expect("failed to execute query");
//         assert_eq!(q.len(), 1);
//         let q = query_to_json("CREATE TABLE IF NOT EXISTS YOLO(x text)")
//             .expect("failed to execute query");
//         assert_eq!(q.len(), 1);
//     }
// }
