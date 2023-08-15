# pg_later
Execute SQL now and get the results later.

A postgres extension to execute queries asynchronously. 

## Installation

### Run with docker

```bash
docker run -p 5432:5432 -e POSTGRES_PASSWORD=postgres quay.io/tembo/pglater-pg:latest
```

If you'd like to build from source, you can follow the instructions in [CONTRIBUTING.md](https://github.com/tembo-io/pg_later/blob/main/CONTRIBUTING.md).


### Using the extension

Initialize the extension's backend:

```sql
CREATE EXTENSION pg_later CASCADE;

SELECT pglater.init();
```

Execute a SQL query now:

```sql
select pglater.exec(
  'select * from pg_available_extensions order by name limit 2'
) as job_id;

 job_id 
--------
     1
(1 row)
```

Come back at some later time, and retrieve the results by providing the job id:

```sql
select pglater.fetch_results(1);

 pg_later_results                                                                                                                                                                                       
--------------------
{
  "query": "select * from pg_available_extensions order by name limit 2",
  "job_id": 1,
  "result": [
    {
      "name": "adminpack",
      "comment": "administrative functions for PostgreSQL",
      "default_version": "2.1",
      "installed_version": null
    },
    {
      "name": "amcheck",
      "comment": "functions for verifying relation integrity",
      "default_version": "1.3",
      "installed_version": null
    }
  ],
  "status": "success"
}
```
