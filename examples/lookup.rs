use nominatim::{IdentificationMethod, NominatimClient};

#[async_std::main]
async fn main() {
    // Search the OSM ID of an object
    let identification = IdentificationMethod::UserAgent("Your Application".to_string());
    let c = NominatimClient { identification };
    let e = c.lookup("W39240305").await.unwrap();

    // Print the results.
    println!("{}", e.location);
}
