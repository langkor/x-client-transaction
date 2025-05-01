use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use url::Url;
use x_client_transaction::ClientTransaction;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up headers
    let mut headers = HeaderMap::new();
    headers.insert("Authority", HeaderValue::from_static("x.com"));
    headers.insert(
        "Accept-Language",
        HeaderValue::from_static("en-US,en;q=0.9"),
    );
    headers.insert("Cache-Control", HeaderValue::from_static("no-cache"));
    headers.insert("Referer", HeaderValue::from_static("https://x.com"));
    headers.insert("X-Twitter-Active-User", HeaderValue::from_static("yes"));
    headers.insert("X-Twitter-Client-Language", HeaderValue::from_static("en"));

    // Create client with headers
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36")
        .default_headers(headers)
        .build()?;

    // Create ClientTransaction instance
    let ct = ClientTransaction::new(&client)?;

    // Example 1
    let url = "https://x.com/i/api/1.1/jot/client_event.json";
    let method = "POST";
    let path = Url::parse(url)?.path().to_string();
    println!("Path 1: {}", path);

    // Example 2
    let user_by_screen_name_url =
        "https://x.com/i/api/graphql/1VOOyvKkiI3FMmkeDNxM9A/UserByScreenName";
    let user_by_screen_name_http_method = "GET";
    let user_by_screen_name_path = Url::parse(user_by_screen_name_url)?.path().to_string();
    println!("Path 2: {}", user_by_screen_name_path);

    // Generate transaction IDs
    let transaction_id = ct.generate_transaction_id(method, &path)?;
    let transaction_id_for_user_by_screen_name_endpoint =
        ct.generate_transaction_id(user_by_screen_name_http_method, &user_by_screen_name_path)?;

    println!("Transaction ID 1: {}", transaction_id);
    println!(
        "Transaction ID 2: {}",
        transaction_id_for_user_by_screen_name_endpoint
    );

    Ok(())
}
