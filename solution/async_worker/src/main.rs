use storage::connections::db_connection::establish_connection;
use storage::provider::get_providers;

fn main() {
    println!("Hello, world!");

    let connection = establish_connection();
    let  pg_pool = connection.get().unwrap();

    let result = get_providers(&mut pg_pool);

    for provider in result
    {

        println!("{}", provider.id);
        println!("{}", provider.name);
    }
}
