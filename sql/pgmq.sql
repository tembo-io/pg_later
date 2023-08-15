-- todo: these will become schema qualified when pgmq is moved to its own schema
select pgmq_create_non_partitioned('pg_later_jobs');
select pgmq_create_non_partitioned('pg_later_results');    