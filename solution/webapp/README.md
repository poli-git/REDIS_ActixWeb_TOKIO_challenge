# WebApp Platform

Web server application that exposes endpoints for interacting with the events platform.

## ENV vars

| Env Var                                             | Required | Info                                                                          | Default Value                             |
| --------------------------------------------------- | -------- | ----------------------------------------------------------------------------- | ----------------------------------------- |
| ACTIX_CLIENT_SHUTDOWN_MS                             | yes      |  Sets server connection shutdown (in Milliseconds) timeout.                                                 | 5000                                       |
| ACTIX_CLIENT_TIMEOUT_MS                           | yes      | Sets server client timeout for first request (in Milliseconds).                                        | 5000                                        |
| ACTIX_SHUTDOWN_TIMEOUT_S                               | yes         | Sets timeout for graceful worker shutdown of workers (in Seconds)                                       | 30                                       |
| ACTIX_KEEPALIVE_SECONDS                               | yes         | Sets server keep-alive preference (in Seconds)                                           | 5                                       |
| ACTIX_NUM_WORKERS                               | yes         | Sets number of workers to start (per bind address).                                          | 4                                      |
| WEB_APP_SERVER                               | yes         |  Address used to create server listener(s).                                      | 127.0.0.1:8088                                       |
| REDIS_URI                               | yes         | The URL of the Cache server                                            | n/a                                       |


## Project Dependencies: Rust

To install Rust, visit [Rust's installation page](https://www.rust-lang.org/tools/install).


## Build the code

```shell
cargo build
```

## Running the WebApp Server

```shell
cargo run
```