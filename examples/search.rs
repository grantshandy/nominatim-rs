use nominatim::Nominatim;

#[async_std::main]
async fn main() {
    // Search for the statue of liberty.
    let e = Nominatim::search("statue of liberty").await.unwrap();

    // Print the results.
    println!("{}", e.location);
}
