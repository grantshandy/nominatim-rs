use nominatim::Nominatim;

#[async_std::main]
async fn main() {
    // Search the coordinates of the Willis Tower.
    let e = Nominatim::reverse(41.87873, -87.63558).await.unwrap();

    // Print the address of the Willis Tower.
    println!("{}", e.location);
}
