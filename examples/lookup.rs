use nominatim::Nominatim;

#[async_std::main]
async fn main() {
    // Search the OSM ID of an object
    let e = Nominatim::lookup("W39240305").await.unwrap();

    // Print the results.
    println!("{}", e.location);
}
