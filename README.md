# X Client Transaction

A Rust implementation of the X Client Transaction library for generating transaction IDs required by the X (formerly
Twitter) API.

## Features

- Handles X migration redirects
- Extracts verification key from X home page
- Generates transaction IDs for X API endpoints
- Cubic curve interpolation and animation for client transaction signatures
- JavaScript-compatible float to hex conversion

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
x_client_transaction = "0.1"
```

## Usage

```rust
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
```

## How It Works

This library replicates the client-side transaction ID generation used by X to authenticate API requests. It:

1. Fetches the X home page and handles any migration redirects
2. Extracts the site verification key
3. Decode the key to get key bytes
4. Find the animation parameters from SVG elements in the page
5. Uses cubic bezier interpolation to calculate animation values
6. Combine these with the request method and path to generate a transaction ID
7. Returns a base64-encoded ID that can be used in API requests

## License

MIT
