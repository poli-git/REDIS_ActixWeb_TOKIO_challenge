# Storage: Persistent Layer

Abstraction over REDIS and Postgresql server using Diesel ORM.

- [Storage: Persistent Layer](#storage-persistent-layer)
  - [Rust](#rust)
  - [Setup](#setup)
  - [Structural Migrations](#structural-migrations)
  - [Tests](#tests)
  - [DataBase](#database)
    - [PROVIDERS](#providers)
    - [BASE\_PLANS](#base_plans)
    - [PLANS](#plans)
    - [ZONES](#zones)

## Rust

To install Rust, visit [Rust's installation page](https://www.rust-lang.org/tools/install).

## Setup

`The following assumes that docker cluster is running`

First, install the diesel cli ([Diesel]):

```shell
cargo install diesel_cli --no-default-features --features postgres
```

Next, run migrations to add the tables into the database:

```shell
diesel migration run
```

## Structural Migrations

To create a new migration:

```shell
diesel migration generate descriptive_name_of_migration
```

A new foder with a new set of `up` and `down` files will be created in the `/migrations` folder. Edit these files with SQL to create the new structure and to tear it down. You may also include other SQL to migrate data.

To apply the new migration:

```shell
diesel migration run
```

## Tests

To run tests:

```shell
cargo test
```

To run a specific test:

```shell
cargo test name_of_test_function
```

To view `stdout` when running tests, append `--nocapture` to the test command:

```shell
cargo test -- --nocapture
```
## DataBase

Note: the order of the fields in the model struct must match the order of the fields in the schema due to diesel. Since the schema is auto-generated from migrations, this typically means that when adding fields, they need to be added as the last field in the struct.

### PROVIDERS

**Provider Structure**:

```rust
pub struct Provider {
    pub providers_id: Uuid,
    pub name: String,    
    pub description: String,
    pub url: String,    
    pub is_active: bool,
    pub created_at: chrono::NaiveDateTime,    
    pub updated_at: chrono::NaiveDateTime,
}
```

### BASE_PLANS

**BasePlan Structure**:

```rust
pub struct BasePlan {
    pub base_plans_id: Uuid,
    pub providers_id: Uuid,
    pub event_base_id: String,
    pub title: String,
    pub sell_mode: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
```

### PLANS

**Plan Structure**:

```rust
pub struct Plan {
    pub plans_id: Uuid,
    pub base_plans_id: Uuid,
    pub event_plan_id: String,
    pub plan_start_date: chrono::NaiveDateTime,
    pub plan_end_date: chrono::NaiveDateTime,
    pub sell_from: chrono::NaiveDateTime,
    pub sell_to: chrono::NaiveDateTime,
    pub sold_out: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
```

### ZONES

**Zone Structure**:

```rust
pub struct Zone {
    pub zones_id: Uuid,
    pub plans_id: Uuid,
    pub event_zone_id: String,
    pub name: String,
    pub capacity: String,
    pub price: String,
    pub numbered: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
```

[Diesel]: https://diesel.rs/