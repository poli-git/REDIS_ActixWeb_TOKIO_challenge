# Solution Persistent Layer

Abstraction over a Postgresql server using Diesel.

-   [Rust](#rust)
-   [Setup](#setup)
-   [Tests](#tests)
-   [API](#api)
    -   [Games](#games)
    -   [Users](#users)
    -   [Contracts](#contracts)

## Rust

To install Rust, visit [Rust's installation page](https://www.rust-lang.org/tools/install).

## Setup

`The following assumes that the solution docker cluster is running`

First, install the diesel cli:

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
cargo test --nocapture
```

To run the debugger on tests:

```shell
RUST_LOG="storage=debug" cargo test -- --nocapture
```

## API

Note for contributors: the order of the fields in the model struct must match the order of the fields in the schema due to diesel. Since the schema is auto-generated from migrations, this typically means that when adding fields, they need to be added as the last field in the struct.

### Games

**Game Structure**:

```rust
pub struct Game {
    pub id: Uuid,
    pub name: String,
    pub api_key: String,
    pub dev_user_id: Uuid,

    pub wallet_app_name: String,
    pub wallet_app_publisher_name: String,
    pub wallet_app_username: String,
    pub wallet_app_password: String,
    pub wallet_app_uid: Uuid,

    pub created_by: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_by: Uuid,
    pub updated_at: chrono::NaiveDateTime,
}
```

#### get_games(connect: &PooledConn) -> Result\<Vec\<Game>, Error>

To retrieve all games:

```rust
use storage::game::get_games;

let connect = connect()?;
let games = get_games(&connect);
```

#### get_game(connect: &PooledConn, game_id: Uuid) -> Result<Game, Error>

To retrieve a single game:

```rust
use storage::game::get_game;

let connect = connect()?;
let game_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
let game = get_game(&connect, game_id);
```

#### get_full_game(connect: &PooledConn, game_id: Uuid) -> Result<Game, Error>

A full game contains the base game structure as well as all associations.
The structure of a full game is:

```rust
pub struct FullGame {
    pub id: Uuid,
    pub name: String,
    pub api_key: String,
    pub dev_user_id: Uuid,

    pub wallet_app_name: String,
    pub wallet_app_publisher_name: String,
    pub wallet_app_username: String,
    pub wallet_app_password: String,
    pub wallet_app_uid: Uuid,

    // associations
    pub users: Vec<User>,
    pub contracts: Vec<Contract>,

    pub created_by: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_by: Uuid,
    pub updated_at: chrono::NaiveDateTime,
}
```

To retrieve a full game:

```rust
use storage::game::get_full_game;

let connect = connect()?;
let game_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
let game = get_full_game(&connect, game_id);
```

#### create_game(connect: &PooledConn, game_id: Uuid) -> Result<Game, Error>

When creating a game, a dev user is also created.
The structure of a NewGame (which includes a NewUser) is:

```rust
pub struct NewGame {
    pub id: Uuid,
    pub name: String,
    pub api_key: String,
    pub wallet_app_name: String,
    pub wallet_app_publisher_name: String,
    pub wallet_app_username: String,
    pub wallet_app_password: String,
    pub wallet_app_uid: Uuid,
    pub dev_user: NewUser,
}

pub struct NewUser {
    pub id: Uuid,
    pub game_id: Uuid,
    pub email: String,
    pub wallet_address: String,
    pub wallet_user_uid: String,
    pub created_by: Uuid,
}
```

To create a game, send in a NewGame struct into `create_game`:

```rust
use storage::game::create_game;

let connect = connect()?;
let game_id = Uuid::new_v4();
let user_id = Uuid::new_v4();
let dev_user = NewUser {
    id: user_id.clone(),
    game_id: game_id.clone(),
    email: "test-user-email".into(),
    wallet_address: "test-user-wallet-address".into(),
    wallet_user_uid: "test-user-wallet_user_uid".into(),

    // for the dev user, use the same user id for `created by`
    created_by: user_id.clone(),
};

let game = NewGame {
    id: game_id.clone(),
    name: "test game".into(),
    api_key: "test-game-api-key".into(),

    wallet_app_name: "test-wallet-app-name".into(),
    wallet_app_publisher_name: "test-wallet-app-".into(),
    wallet_app_username: "test-wallet-app-username".into(),
    wallet_app_password: "test-wallet-app-password".into(),
    wallet_app_uid: Uuid::new_v4(),

    dev_user,
    blockchain_url: "http://test_url:8545".to_owned(),
};

let result = create_game(&connect, game);
```

### Users

**User Structure**:

```rust
pub struct User {
    pub id: Uuid,
    pub game_id: Uuid,
    pub email: String,

    pub wallet_address: String,
    pub wallet_user_uid: String,

    pub created_by: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_by: Uuid,
    pub updated_at: chrono::NaiveDateTime,
}
```

#### get_users(connect: &PooledConn, game_id: Uuid) -> Result\<Vec\<User>, Error>

To retrieve all users for a game:

```rust
use storage::game::get_games;
use storage::users::get_users;

let connect = connect()?;
let first_game = &get_games(&connect).unwrap().clone()[0];
let users = get_users(&connect, first_game.id);
```

#### get_user(connect: &PooledConn, user_id: Uuid) -> Result<Game, Error>

To retrieve a single user:

```rust
use storage::user::get_user;

let connect = connect()?;
let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
let user = get_user(&connect, user_id);
```

#### create_user(connect: &PooledConn, user: NewUser) -> Result<User, Error>

The structure of a NewUser is:

```rust
pub struct NewUser {
    pub id: Uuid,
    pub game_id: Uuid,
    pub email: String,
    pub wallet_address: String,
    pub wallet_user_uid: String,
    pub created_by: Uuid,
}
```

To create a game, send in a NewUser struct:

```rust
use storage::user::create_user;

let connect = connect()?;
let game_id = Uuid::new_v4();
let user_id = Uuid::new_v4();
let user = NewUser {
    id: user_id.clone(),
    game_id: Uuid::new_v4(),
    email: "test-user-email".into(),
    wallet_address: "test-user-wallet-address".into(),
    wallet_user_uid: "test-user-wallet_user_uid".into(),
    created_by: user_id.clone(),
};

let result = create_user(&connect, user);
```

### Contracts

**Contract Structure**:

```rust
pub struct Contract {
    pub id: Uuid,
    pub game_id: Uuid,

    pub kind: String,
    pub address: String,

    pub created_by: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_by: Uuid,
    pub updated_at: chrono::NaiveDateTime,
}
```

#### get_contracts(connect: &PooledConn, game_id: Uuid) -> Result\<Vec\<Contract>, Error>

To retrieve all contracts for a game:

```rust
use storage::game::get_games;
use storage::contract::get_contracts;

let connect = connect()?;
let first_game = &get_games(&connect).unwrap().clone()[0];
let contracts = get_contracts(&connect, first_game.id);
```

#### get_contract(connect: &PooledConn, contract_id: Uuid) -> Result<Contract, Error>

To retrieve a single contract:

```rust
use storage::contract::get_contract;

let connect = connect()?;
let contract_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
let contract = get_contract(&connect, contract_id);
```

#### create_contract(connect: &PooledConn, contract: NewContract) -> Result<Contract, Error>

The structure of a NewContract is:

```rust
pub struct NewContract {
    pub id: Uuid,
    pub game_id: Uuid,

    pub kind: String,
    pub address: String,

    pub created_by: Uuid,
}
```

#### get_contract_by_kind(connect: &PooledConn, contract_id: Uuid, kind: ContractType) -> Result<Game, Error>

To retrieve a single contract by kind:

```rust
use storage::contract::*;
use storage::models::contract::*;

let connect = connect()?;
let game_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
let kind = ContractType::erc20;
let contract = get_contract_by_kind(&connect, contract_id, kind);
```

### Assets

**Asset Structure**:

```rust
pub struct Asset {
    pub id: Uuid,
    pub game_id: Uuid,
    pub token_id: String,
    pub data: serde_json::Value,
    pub key: String,
    pub created_by: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_by: Uuid,
    pub updated_at: chrono::NaiveDateTime,
}
```

#### create_asset(connect: &PooledConn, asset_instance: NewAsset) -> Result\<Asset, Error>

Creates a new asset.

```rust
use storage::connect::*;
use storage::asset::create_asset;
let asset = NewAsset {
    id: Uuid::new_v4(),
    game_id: *game_id,
    token_id: item.id.to_string(),
    key: key,
    data: params.metadata.clone(),
    created_by: contract.created_by,
};

let result = create_asset(asset);
```

#### get_asset_by_token_id(connect: &PooledConn, \_token_id: String) -> Result\<Asset, Error> {

The retrieve an asset by the token_id:

```rust
use storage::asset::get_asset_by_token_id;

let connect = connect()?;
let token_id = "23479483317544753978972847912792006590464".to_string();
let asset = get_asset_by_token_id(&db_connect, token_id);
```

#### get_asset_by_game(connect: &PooledConn, game_id_param: Uuid) -> Result<Vec<Asset>, Error> {ContractType) -> Result<Game, Error>

To retrieve a single asset by game_id:

```rust
use storage::asset::get_asset_by_game;

let connect = connect()?;
let game_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
let asset = get_asset_by_game(&db_connect, game_id);

```
