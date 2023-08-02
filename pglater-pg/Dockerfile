FROM rust:1.70-bookworm as builder

RUN cargo install pg-trunk

# FROM postgres:15.3-bullseye
FROM quay.io/tembo/tembo-pg-cnpg:latest

USER root
COPY --from=builder /usr/local/cargo/bin/trunk /usr/bin/trunk
RUN trunk install pg_later

ENV PGDATA /var/lib/postgresql/data2
RUN mkdir -p $PGDATA
RUN chown -R postgres:postgres $PGDATA
RUN chmod -R 0700 $PGDATA

USER postgres
RUN pg_ctl init

ENTRYPOINT ["/bin/bash"]
CMD ["postgres"]
