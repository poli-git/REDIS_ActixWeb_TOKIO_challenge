# Deployment Guide

## Steps to deploy on a local environment

1. Deploy your docker dependency components: 

   Open a Command line window and navigate to the directory containing `docker-compose.yaml` file (***solution** folder*). 
   
   Execute the command:

    ```bash
    docker compose up
    ```

    Once Docker dependencies are running (**Redis** and **Postgresql**), keep it running in the terminal, and switch to a new terminal.

-------------

2. Install Rust
    
    To install Rust, visit [Rust's installation page](https://www.rust-lang.org/tools/install).

----------------

3. Diesel Migrations - Database Setup. (*The following assumes that docker cluster is running*)

    First, install the diesel cli [Diesel](https://diesel.rs):

    ```shell
    cargo install diesel_cli --no-default-features --features postgres
    ```

    Open a new command line window and navigate to the **solution/storage** folder. 
    
    Execute the command:

    ```shell
    diesel migration run
    ```
---------------

4. Run Async Worker Process:
   
    Open a new command line window and navigate to the **solution/async_worker** folder.

    Execute the command:

    ```shell
    cargo run
    ```

    Once Async Worker Process is running, keep it running in the terminal, and switch to a new terminal.
---------------

5. Run WebApp platform:
   
    Open a new command line window and navigate to the **solution/webapp** folder.

    Execute the command:

    ```shell
    cargo run
    ```


    Once WebApp platform is running, keep it running in the terminal

-----------------

 6. Invoke `search` endpoint from WebApp platform:
   
    **API SPEC**: http://localhost:8088/swagger-ui/

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

## How to run all tests

```bash
cargo test
```

## How to run type checking (lint)

```bash
cargo clippy
```

## How to run formatting

```bash
cargo fmt --all
```