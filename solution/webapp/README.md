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

## API Specification
**API SPEC**: http://localhost:8088/swagger-ui/



## SEARCH EndPoint
Invoke `search` endpoint from WebApp platform:
   
    Open a new command line window execute

    ```shell
    curl -X 'GET' \
        'http://localhost:8088/search?starts_at=2021-05-10T10:30:00&ends_at=2021-08-30T23:50:00'\
        -H 'accept: application/json'
    ```


    Expected outcome example: 
    ```shell
            {
            "data": {
                "events": [
                {
                    "id": "291",
                    "title": "Camela en concierto",
                    "start_date": "2021-06-30",
                    "start_time": "21:00:00",
                    "end_date": "2021-06-30",
                    "end_time": "21:30:00",
                    "min_price": 15,
                    "max_price": 30
                },
                {
                    "id": "1591",
                    "title": "Los Morancos",
                    "start_date": "2021-07-31",
                    "start_time": "20:00:00",
                    "end_date": "2021-07-31",
                    "end_time": "21:00:00",
                    "min_price": 65,
                    "max_price": 75
                }
                ]
            },
        "error": null
        }
    ```


---------------