use crate::error::Error;
use base64::{engine::general_purpose, Engine as _};
use regex::Regex;
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;

/// Handle X migration redirect and get the home page
pub fn handle_x_migration(client: &Client) -> Result<Html, Error> {
    let migration_regex = Regex::new(
        r"(https?://(?:www\.)?(twitter|x)\.com(/x)?/migrate([/?])?tok=[a-zA-Z0-9%\-_]+)+",
    )
    .expect("Migration regex is valid");

    let response = client.get("https://x.com").send()?;
    let html = Html::parse_document(&response.text()?);

    // Check for migration meta tag
    let meta_selector = Selector::parse("meta[http-equiv='refresh']").unwrap();
    if let Some(meta) = html.select(&meta_selector).next() {
        let content = meta.value().attr("content").unwrap_or("");
        if let Some(captures) = migration_regex.captures(content) {
            if let Some(url) = captures.get(0) {
                let migration_response = client.get(url.as_str()).send()?;
                return Ok(Html::parse_document(&migration_response.text()?));
            }
        }
    }

    // Check for a migration form
    let form_selector1 = Selector::parse("form[name='f']").unwrap();
    let form_selector2 = Selector::parse("form[action='https://x.com/x/migrate']").unwrap();

    let form = html
        .select(&form_selector1)
        .next()
        .or_else(|| html.select(&form_selector2).next());

    if let Some(form) = form {
        let url = form
            .value()
            .attr("action")
            .unwrap_or("https://x.com/x/migrate");
        let method = form.value().attr("method").unwrap_or("POST");

        let input_selector = Selector::parse("input").unwrap();
        let mut request_payload = HashMap::new();

        for input in form.select(&input_selector) {
            if let Some(name) = input.value().attr("name") {
                if let Some(value) = input.value().attr("value") {
                    request_payload.insert(name, value);
                }
            }
        }

        let migration_response = match method.to_uppercase().as_str() {
            "POST" => client.post(url).form(&request_payload).send()?,
            _ => client.get(url).query(&request_payload).send()?,
        };

        return Ok(Html::parse_document(&migration_response.text()?));
    }

    // If no migration needed, return the original HTML
    Ok(html)
}

/// Check if a number is odd
pub fn is_odd(num: i32) -> f64 {
    if num % 2 == 1 {
        -1.0
    } else {
        0.0
    }
}

/// Round a number in JavaScript style (ROUND_HALF_UP)
pub fn js_round(num: f64) -> f64 {
    let decimal_part = num - num.trunc();
    if decimal_part == -0.5 {
        num.ceil()
    } else {
        num.round()
    }
}

/// Convert a float to a hexadecimal string
pub fn float_to_hex(x: f64) -> String {
    if x == 0.0 {
        return "0".to_string();
    }

    let mut result = String::new();
    let mut quotient = x.floor() as i64;
    let mut fraction = x - quotient as f64;

    let parse_digit = |value: i64| {
        if value > 9 {
            std::char::from_u32((value as u32) + 55).unwrap()
        } else {
            std::char::from_digit(value as u32, 10).unwrap()
        }
    };

    // Convert integer part
    if quotient == 0 {
        result.push('0');
    } else {
        while quotient > 0 {
            let remainder = quotient % 16;
            quotient /= 16;

            result.insert(0, parse_digit(remainder));
        }
    }

    // Convert fraction part
    if fraction > 0.0 {
        result.push('.');

        while fraction > 0.0 {
            fraction *= 16.0;
            let integer = fraction.floor() as i64;
            fraction -= integer as f64;

            result.push(parse_digit(integer));

            // Avoid infinite loops due to precision issues
            if result.len() > 20 {
                break;
            }
        }
    }

    result
}

/// Base64 encode a byte array
pub fn base64_encode<T: AsRef<[u8]>>(data: T) -> String {
    general_purpose::STANDARD.encode(data)
}

/// Base64 decode a string
pub fn base64_decode<T: AsRef<[u8]>>(input: T) -> Result<Vec<u8>, base64::DecodeError> {
    general_purpose::STANDARD.decode(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_odd() {
        assert_eq!(is_odd(1), -1.0);
        assert_eq!(is_odd(2), 0.0);
        assert_eq!(is_odd(3), -1.0);
        assert_eq!(is_odd(4), 0.0);
    }

    #[test]
    fn test_js_round() {
        assert_eq!(js_round(0.0), 0.0);
        assert_eq!(js_round(0.4), 0.0);
        assert_eq!(js_round(0.5), 1.0);
        assert_eq!(js_round(0.6), 1.0);
        assert_eq!(js_round(1.5), 2.0);

        assert_eq!(js_round(-0.0), 0.0);
        assert_eq!(js_round(-0.4), 0.0);
        assert_eq!(js_round(-0.5), -0.0);
        assert_eq!(js_round(-0.6), -1.0);
        assert_eq!(js_round(-1.5), -1.0);
    }

    #[test]
    fn test_float_to_hex() {
        assert_eq!(float_to_hex(10.0), "A");
        assert_eq!(float_to_hex(16.0), "10");
        assert_eq!(float_to_hex(0.5), "0.8");
    }

    #[test]
    fn test_base64() {
        let original = "hello world";
        let encoded = base64_encode(original);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, original.as_bytes());
    }
}
