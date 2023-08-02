FROM rust:1.70-bookworm as builder

RUN cargo install pg-trunk

# FROM postgres:15.3-bullseye
FROM quay.io/coredb/coredb-pg-base:latest
USER root

COPY --from=builder /usr/local/cargo/bin/trunk /usr/bin/trunk

RUN trunk install pg_later

USER postgres
CMD ["postgres"]