FROM --platform=linux/amd64 quay.io/coredb/pgrx-builder:pg15-pgrx0.11.2

RUN cargo install pg-trunk

USER root

RUN chown -R postgres:postgres /usr/share/postgresql
RUN chown -R postgres:postgres /usr/lib/postgresql

USER postgres

RUN trunk install pgmq
RUN echo "shared_preload_libraries = 'pg_later'" >> ~/.pgrx/data-15/postgresql.conf
ENV DATABASE_URL="postgres://postgres:postgres@localhost:28815/pg_later"
