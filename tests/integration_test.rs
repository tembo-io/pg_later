use rand::Rng;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres, Row};
use tokio::time::{sleep, Duration};

async fn connect(url: &str) -> Pool<Postgres> {
    let options = pgmq_core::util::conn_options(url).expect("failed to parse url");
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(10))
        .max_connections(5)
        .connect_with(options)
        .await
        .unwrap()
}

#[tokio::test]
async fn test_lifecycle() {
    let username = whoami::username();
    let conn = connect(&format!(
        "postgres://{username}:postgres@localhost:28815/postgres"
    ))
    .await;
    let mut rng = rand::thread_rng();

    let _ = sqlx::query("DROP EXTENSION IF EXISTS pg_later")
        .execute(&conn)
        .await
        .expect("failed to drop extension");

    let _ = sqlx::query("CREATE EXTENSION pg_later CASCADE")
        .execute(&conn)
        .await
        .expect("failed to create");

    let _ = sqlx::query("SELECT pglater.init()")
        .execute(&conn)
        .await
        .expect("failed to init");

    // simple select case
    let q0 = sqlx::query("SELECT pglater.exec('select 1')")
        .fetch_one(&conn)
        .await
        .expect("failed to exec")
        .get::<i64, usize>(0);
    assert!(q0 > 0);
    sleep(Duration::from_secs(7)).await;

    let row: (serde_json::Value,) = sqlx::query_as(&format!(
        "SELECT pglater.fetch_results({q0})::json as results"
    ))
    .fetch_one(&conn)
    .await
    .expect("failed to fetch");
    let r = row.0;
    assert_eq!(
        r.get("query").expect("no query").to_owned(),
        "select 1".to_string()
    );
    assert_eq!(
        r.get("status").expect("no query").to_owned(),
        "success".to_string()
    );

    // invalid query case
    let invalid_query = "invalid query";
    let result = sqlx::query(&format!("SELECT pglater.exec('{}')", invalid_query))
        .fetch_one(&conn)
        .await;
    assert!(
        result.is_err(),
        "Executing an invalid query should result in an error"
    );

    // create table case
    let test_num = rng.gen_range(0..100000);
    let table_name = format!("test_table_{}", test_num);
    let pglater_exec = format!(
        "SELECT pglater.exec('create table if not exists \"{}\" (x text)')",
        table_name
    );
    println!("pglater exec: {}", pglater_exec);
    let q1 = sqlx::query(&pglater_exec)
        .fetch_one(&conn)
        .await
        .expect("failed to exec create table")
        .get::<i64, usize>(0);
    assert!(q1 > q0, "job ids should increase");
    sleep(Duration::from_secs(5)).await;
    let row: (serde_json::Value,) = sqlx::query_as(&format!(
        "SELECT pglater.fetch_results({q1})::json as results"
    ))
    .fetch_one(&conn)
    .await
    .expect("failed to fetch");
    let r = row.0;
    assert_eq!(
        r.get("status").expect("no query").to_owned(),
        "success".to_string()
    );

    let exists_query = format!(
        "
        SELECT EXISTS (
            SELECT 1
            FROM pg_tables
            WHERE tablename = '{}'
        );",
        table_name
    );
    println!("exists query: {}", exists_query);
    let row: (bool,) = sqlx::query_as(&exists_query)
        .fetch_one(&conn)
        .await
        .expect("failed to fetch");
    assert!(row.0, "table must exist");
}
