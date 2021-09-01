use nominatim::Nominatim;

#[async_std::main]
async fn main() {
    // Get status of the Nominatim server.
    match Nominatim::status().await {
        Ok(_) => println!("Server working properly..."),
        Err(error) => println!("Error: {}", error.to_string()),
    }
}
