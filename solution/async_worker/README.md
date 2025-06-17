# Async Worker Process

Asynchronous process that retrieves a list of available plans (in XML format) from multiple providers and stores them in both Redis Cache and the PostgreSQL database.

## ENV vars

| Env Var                                             | Required | Info                                                                          | Default Value                             |
| --------------------------------------------------- | -------- | ----------------------------------------------------------------------------- | ----------------------------------------- |
| DATABASE_URL                             | yes      | The URL of the DB server.                                                 | n/a                                       |
| ASYNC_WORKER_INTERVAL_SEC                           | yes      | Execution Interval delay (in seconds).                                        | 30                                        |
| REDIS_URI                               | yes         | The URL of the Cache server                                            | n/a                                       |


## Project Dependencies: Rust

To install Rust, visit [Rust's installation page](https://www.rust-lang.org/tools/install).


## Build the code

```shell
cargo build
```

## Running the Worker

```shell
cargo run
```


## Deployment & Upgrade Process

See the [RUNBOOK](RUNBOOK.md) for information on deploying and maintaining hosted instances of this service.
