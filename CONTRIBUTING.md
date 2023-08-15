# Contributing to `pg_later`

## Installation

The fastest way to get started is by running the Tembo docker image, where `pg_later` and its dependencies come pre-installed.

```bash
docker run -d --name pglater-pg -e POSTGRES_PASSWORD=postgres -p 5432:5432 quay.io/tembo/pglater-pg:latest
```

## Building from source

`pg_later` is written as a Rust extension and requires [pgrx](https://github.com/pgcentralfoundation/pgrx).

To build `pg_later` from source, you need
* A toolchain capable of building Postgres
* Rust toolchain
* [pg_partman](https://github.com/pgpartman/pg_partman).

Once you have those pre-requisites, you need to setup `pgrx`.

```bash
cargo install --locked cargo-pgrx --version 0.9.8
```

Clone the repo and change into the directory.

```bash
git clone https://github.com/tembo-io/pg_later.git
cd pg_later
```

After this point, the steps differ slightly based on if you'd like to build
and install against an existing Postgres setup or develop against pgrx managed
development environment (which installs and allows you to test against multiple
Postgres versions).

### Install to a pre-existing Postgres

Initialize `cargo-pgrx`, and tell it the path to the your `pg_config`. For example,
if `pg_config` is on your `$PATH` and you have Postgres 15, you can run:

```bash
cargo pgrx init --pg15=`which pg_config`
```
Then, to install the release build, you can simply run:
```
cargo pgrx install --release
```

### Install against pgrx managed Postgres (Recommended for Development)

Initialize `cargo-pgrx` development environment:

```bash
cargo pgrx init
```

**Note**: Make sure you build and install `pgmq` against the postgres installation
you want to build against (`PG_CONFIG` in `~/.pgrx/PG_VERSION/pgrx-install/bin/pg_config`
and `PGDATA` in `/Users/<myuser>/.pgrx/data-PG_MAJOR_VERSION`). Refer to the [pgmq installation instructions](https://github.com/tembo-io/pgmq/blob/main/CONTRIBUTING.md#building-from-source).

Then, you can use the run command, which will build and install the extension
and drop you into psql:

```bash
cargo pgrx run pg15
```

Finally, you can create the extension and get started with the example in the [README.md](README.md).

```psql
CREATE EXTENSION pg_later cascade;
```

# Releases and versioning

`pg_later` Postgres Extension releases are automated through a [Github workflow](https://github.com/tembo-io/pg_later/blob/main/.github/workflows/pg_later_ext.yml). 
The compiled binaries are publish to and hosted at [pgt.dev](https://pgt.dev). To create a release, create a new tag follow a valid [semver](https://semver.org/), then create a release with the same name. 
This release must match the version found in Cargo.toml. 
Auto-generate the release notes and/or add more relevant details as needed.
