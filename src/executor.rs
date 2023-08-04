use pgrx::prelude::*;
use std::panic::{self};

use crate::clf;

fn exec_row_query(query: &str) -> Result<Vec<pgrx::JsonB>, spi::Error> {
    let mut results: Vec<pgrx::JsonB> = Vec::new();
    let queried = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let queried: Result<(), spi::Error> = Spi::connect(|mut client| {
            let q = format!("select to_jsonb(t) as results from ({query}) t");
            log!("pg-later: executing query: {q}");
            let tup_table = client.update(&q, None, None)?;
            results.reserve_exact(tup_table.len());
            for row in tup_table {
                let r = row["results"]
                    .value::<pgrx::JsonB>()
                    .expect("failed parsing as json")
                    .expect("no results from query");
                results.push(r);
            }
            Ok(())
        });

        queried
    }));

    match queried {
        Ok(_) => Ok(results),
        Err(e) => {
            log!("Error: {:?}", e);
            // TODO: a more appropriate error enum
            Err(spi::Error::CursorNotFound(
                "pg-later: failed to execute query".to_owned(),
            ))
        }
    }
}

// execute query that does not return rows
fn exec_utility(query: &str) -> Result<Vec<pgrx::JsonB>, spi::Error> {
    let mut results: Vec<pgrx::JsonB> = Vec::new();
    let _ = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let queried: Result<(), spi::Error> = Spi::connect(|mut client| {
            let tup_table = client.update(query, None, None)?;
            log!("{:?}", tup_table);
            results.push(pgrx::JsonB(serde_json::json!(format!("{:?}", tup_table))));
            Ok(())
        });
        queried
    }));
    Ok(results)
}

pub fn query_to_json(query: &str) -> Result<Vec<pgrx::JsonB>, spi::Error> {
    match clf::returns_rows(query) {
        true => {
            log!("row query");
            exec_row_query(query)
        }
        false => {
            log!("utility statement");
            exec_utility(query)
        }
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    #[pg_test]
    fn test_json() {
        let q = query_to_json("select 1").expect("failed to execute query");
        assert_eq!(q.len(), 1);
        let q = query_to_json("CREATE TABLE IF NOT EXISTS YOLO(x text)")
            .expect("failed to execute query");
        assert_eq!(q.len(), 1);
    }
}
