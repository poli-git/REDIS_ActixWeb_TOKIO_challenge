// use crate::helpers::{not_found, Param};
use crate::models::provider::*;
use crate::schema::providers::dsl::*;

use diesel::prelude::*;

/// Get all users for game
///
/// ```
/// use storage::connect::*;
/// use storage::game::get_games_blocking as get_games;
/// use storage::user::get_users_blocking as get_users;
///
/// let connect = connect().unwrap();
/// let first_game = &get_games(&connect).unwrap().clone()[0];
/// let results = get_users(&connect, first_game.id, 1, 5).unwrap();
/// assert!(results.len() > 0);
/// ```


