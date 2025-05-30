use storage::connections::db_connection::establish_connection;
use storage::provider::get_providers;

fn main() {
    println!("Hello, world!");

    let mut con = establish_connection();
    let providers = get_providers(&mut con);

    for provider in providers {
        println!("{:?}", provider);
    }
}
