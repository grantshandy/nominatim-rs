use nominatim::{IdentificationMethod, NominatimClient};

#[async_std::main]
async fn main() {
    // Search for the statue of liberty.
    let identification = IdentificationMethod::UserAgent("Your Application".to_string());
    let c = NominatimClient { identification };
    let e = c.search("statue of liberty").await.unwrap();

    // Print the results.
    println!("{}", e.location);
}
