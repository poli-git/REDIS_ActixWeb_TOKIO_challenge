use storage::connections::db::establish_connection;
use storage::provider::get_providers;

fn main() {
    println!("Hello, world!");

    let connection = establish_connection();
    let mut pg_pool = connection.get().unwrap();

    match get_providers(&mut pg_pool) {
        Ok(providers) => {
            for provider in providers {
                println!("{}", provider.id);
                println!("{}", provider.name);
            }
        }
        Err(e) => {
            eprintln!("Error fetching providers: {}", e);
        }
    }
}
