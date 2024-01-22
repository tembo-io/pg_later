FROM --platform=linux/amd64 quay.io/coredb/pgrx-builder:pg15-pgrx0.11.2

RUN cargo install pg-trunk

USER root

RUN chown -R postgres:postgres /usr/share/postgresql
RUN chown -R postgres:postgres /usr/lib/postgresql

USER postgres

RUN trunk install pgmq
RUN trunk install pg_partman
