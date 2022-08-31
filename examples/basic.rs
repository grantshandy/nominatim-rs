use nominatim::{Client, IdentificationMethod};

#[tokio::main]
async fn main() {
    let client = Client::new(IdentificationMethod::from_user_agent(
        "Example Application Name",
    ));

    
    println!("---- status ----");
    let status = client.status().await.unwrap();
    println!("{}", status.message);


    println!("---- search ----");
    let search_results = client.search("statue of liberty").await.unwrap();

    for place in search_results {
        println!("{}", place.display_name);
    }


    println!("---- reverse ----");
    let reverse_search = client
        .reverse("40.689249", "-74.044500", None)
        .await
        .unwrap();

    println!("{}", reverse_search.display_name);


    println!("---- lookup ----");
    let lookup_results = client.lookup(vec!["R146656", "W50637691"]).await.unwrap();

    for place in lookup_results {
        println!("{}", place.display_name);
    }
}
