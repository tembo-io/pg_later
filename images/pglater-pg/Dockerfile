FROM quay.io/tembo/tembo-local:latest

ARG PG_LATER_VER=0.1.0
ARG PGMQ_VER=1.3.3

RUN trunk install pgmq --version $PGMQ_VER
RUN trunk install pg_later --version $PG_LATER_VER

USER root
RUN echo "shared_preload_libraries = 'pg_later'" >> /var/lib/postgresql/data2/postgresql.conf
USER postgres