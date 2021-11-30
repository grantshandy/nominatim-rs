use nominatim::{IdentificationMethod, NominatimClient};

#[async_std::main]
async fn main() {
    // Search the coordinates of the Willis Tower.
    let identification = IdentificationMethod::UserAgent("Your Application".to_string());
    let c = NominatimClient { identification };
    let e = c.reverse(41.87873, -87.63558).await.unwrap();

    // Print the address of the Willis Tower.
    println!("{}", e.location);
}
