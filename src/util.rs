use anyhow::Result;
use pgrx::prelude::*;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Pool, Postgres};
use std::env;
use url::{ParseError, Url};

use crate::guc;

#[derive(Clone, Debug)]
pub struct Config {
    pub pg_conn_str: String,
    pub env_socket_url: Option<String>,
    pub guc_host: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pg_conn_str: from_env_default(
                "DATABASE_URL",
                "postgresql://postgres:postgres@0.0.0.0:5432/postgres",
            ),
            env_socket_url: env::var("PGLATER_SOCKET_URL").ok(),
            guc_host: guc::get_guc(guc::PglaterGUC::Host),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PostgresSocketConnection {
    user: Option<String>,
    dbname: Option<String>,
    host: Option<String>,
    password: Option<String>,
    port: Option<u16>, // Add other potential query parameters as needed
}

impl PostgresSocketConnection {
    fn from_unix_socket_string(s: &str) -> Option<Self> {
        let parsed_url = url::Url::parse(s).ok()?;
        let mut connection = PostgresSocketConnection::default();
        for (key, value) in parsed_url.query_pairs() {
            match key.as_ref() {
                "user" => connection.user = Some(value.into_owned()),
                "dbname" => connection.dbname = Some(value.into_owned()),
                "host" => connection.host = Some(value.into_owned()),
                "password" => connection.password = Some(value.into_owned()),
                "port" => connection.port = Some(value.parse::<u16>().expect("invalid port")),
                // Add other potential query parameters as needed
                _ => {} // Ignoring unknown parameters
            }
        }

        Some(connection)
    }
}

pub fn from_env_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_owned())
}

pub async fn get_pg_conn() -> Result<Pool<Postgres>> {
    let opts = get_pg_options()?;
    let pgp = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(10))
        .max_connections(4)
        .connect_with(opts)
        .await?;
    Ok(pgp)
}

pub fn get_pgc_socket_opt(socket_conn: PostgresSocketConnection) -> Result<PgConnectOptions> {
    let mut opts = PgConnectOptions::new();
    opts = opts.socket(socket_conn.host.expect("missing socket host"));
    log!("socket options: {:?}", opts);
    if socket_conn.port.is_some() {
        opts = opts.port(socket_conn.port.expect("missing socket port"));
    } else {
        opts = opts.port(5432);
    }
    if socket_conn.dbname.is_some() {
        opts = opts.database(&socket_conn.dbname.expect("missing socket dbname"));
    } else {
        opts = opts.database("postgres");
    }
    if socket_conn.user.is_some() {
        opts = opts.username(&socket_conn.user.expect("missing socket user"));
    } else {
        opts = opts.username("postgres");
    }
    if socket_conn.password.is_some() {
        opts = opts.password(&socket_conn.password.expect("missing socket password"));
    }
    Ok(opts)
}

fn get_pgc_tcp_opt(url: Url) -> Result<PgConnectOptions> {
    let options = PgConnectOptions::new()
        .host(url.host_str().ok_or(ParseError::EmptyHost)?)
        .port(url.port().ok_or(ParseError::InvalidPort)?)
        .username(url.username())
        .password(url.password().ok_or(ParseError::IdnaError)?)
        .database(url.path().trim_start_matches('/'));
    log!("tcp options: {:?}", options);
    Ok(options)
}

pub fn get_pg_options() -> Result<PgConnectOptions> {
    let cfg = Config::default();

    let guc_host: Option<String> = cfg.guc_host;
    let env_socket: Option<String> = cfg.env_socket_url;
    let env_url: String = cfg.pg_conn_str;

    match (guc_host.as_ref(), env_socket.as_ref()) {
        (Some(guc), _) => {
            log!("pg-later: connecting with value from pglater.host");
            let socket_conn = PostgresSocketConnection::from_unix_socket_string(guc)
                .expect("invalid value in pglater.host");
            get_pgc_socket_opt(socket_conn)
        }
        (None, Some(env)) => {
            log!("pg-later: connecting with value from env PGLATER_SOCKET_URL");
            let socket_conn = PostgresSocketConnection::from_unix_socket_string(env)
                .expect("invalid value in env PGLATER_SOCKET_URL");
            get_pgc_socket_opt(socket_conn)
        }
        (None, None) => {
            log!("pg-later: connecting with value from env DATABASE_URL");
            let url = Url::parse(&env_url)?;
            get_pgc_tcp_opt(url)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_parsing_socket() {
        let expected = PostgresSocketConnection {
            user: Some("me".to_string()),
            dbname: Some("pg_later_test".to_string()),
            host: Some("/home/me/.pgrx".to_string()),
            password: Some("pw".to_string()),
            port: Some(5432),
        };

        let parsed = PostgresSocketConnection::from_unix_socket_string(
            "postgresql:///?user=me&host=/home/me/.pgrx&password=pw&port=5432&dbname=pg_later_test",
        )
        .unwrap();
        assert_eq!(parsed, expected);
    }
}
