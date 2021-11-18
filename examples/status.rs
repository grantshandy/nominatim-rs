use nominatim::{IdentificationMethod, NominatimClient};

#[async_std::main]
async fn main() {
    // Get status of the Nominatim server.
    let identification = IdentificationMethod::UserAgent("Your Application".to_string());
    let c = NominatimClient { identification };
    match c.status().await {
        Ok(_) => println!("Server working properly..."),
        Err(error) => println!("Error: {}", error.to_string()),
    }
}
