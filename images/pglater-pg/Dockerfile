FROM quay.io/tembo/tembo-pg-cnpg:latest

ARG PG_LATER_VER=0.0.12
ARG PGMQ_VER=0.32.1

USER root
RUN chown -R postgres:postgres $PGDATA && \
    chmod -R 0700 $PGDATA
ENV PGDATA /var/lib/postgresql/data2
RUN mkdir -p $PGDATA
RUN chown -R postgres:postgres $PGDATA && \
    chmod -R 0700 $PGDATA

USER postgres
RUN pg_ctl init

RUN trunk install pgmq --version $PGMQ_VER
RUN trunk install pg_later --version $PG_LATER_VER

# Set permissive authentication
RUN echo "host all all 0.0.0.0/0 trust" >> ${PGDATA}/pg_hba.conf
COPY postgresql.conf ${PGDATA}/postgresql.conf

CMD ["postgres"]
