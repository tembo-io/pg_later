use pgrx::prelude::*;
use sqlx::postgres::PgRow;
use sqlx::{Pool, Postgres, Row};

use anyhow::Result;

use crate::clf;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PostgresType)]
pub struct Job {
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

// execute query that does not return rows
async fn exec_utility(query: &str, conn: &Pool<Postgres>) -> Result<Vec<serde_json::Value>> {
    let result = sqlx::query(query).execute(conn).await?;
    Ok(vec![
        serde_json::json!({ "rows_affected": result.rows_affected() }),
    ])
}

pub async fn query_to_json(query: &str, conn: &Pool<Postgres>) -> Result<Vec<serde_json::Value>> {
    match clf::returns_rows(query) {
        true => {
            log!("row query");
            exec_row_query(query, conn).await
        }
        false => {
            log!("utility statement");
            exec_utility(query, conn).await
        }
    }
}
