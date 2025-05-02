use crate::cubic_curve::Cubic;
use crate::error::Error;
use crate::interpolate::interpolate;
use crate::rotation::convert_rotation_to_matrix;
use crate::utils::{
    base64_decode, base64_encode, float_to_hex, handle_x_migration, is_odd, js_round,
};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

lazy_static! {
    static ref ON_DEMAND_FILE_REGEX: Regex =
        Regex::new(r#"['|\"]ondemand\.s['|\"]:\s*['|\"]([\w]*)['|\"]"#).unwrap();
    static ref INDICES_REGEX: Regex = Regex::new(r"(\(\w{1}\[(\d{1,2})\],\s*16\))+").unwrap();
}

pub struct ClientTransaction {
    additional_random_number: u8,
    default_keyword: String,
    key_bytes: Vec<u8>,
    animation_key: String,
}

impl ClientTransaction {
    pub fn new(client: &Client) -> Result<Self, Error> {
        let home_page = handle_x_migration(client)?;

        let (row_index, key_bytes_indices) = Self::get_indices(&home_page, client)?;
        let key = Self::get_key(&home_page)?;
        let key_bytes = Self::get_key_bytes(&key)?;
        let animation_key =
            Self::get_animation_key(&key_bytes, &home_page, row_index, &key_bytes_indices)?;

        Ok(Self {
            additional_random_number: 3,
            default_keyword: String::from("obfiowerehiring"),
            key_bytes,
            animation_key,
        })
    }

    fn get_indices(home_page: &Html, client: &Client) -> Result<(usize, Vec<usize>), Error> {
        let mut key_byte_indices = Vec::new();

        // Find ondemand file
        let html_content = home_page.html();
        let on_demand_file = ON_DEMAND_FILE_REGEX
            .captures(&html_content)
            .ok_or_else(|| Error::ParseError("Couldn't find ondemand file".into()))?;

        let on_demand_file_url = format!(
            "https://abs.twimg.com/responsive-web/client-web/ondemand.s.{}a.js",
            on_demand_file.get(1).unwrap().as_str()
        );

        // Fetch ondemand file
        let on_demand_response = client.get(&on_demand_file_url).send()?;
        let on_demand_content = on_demand_response.text()?;

        // Extract indices
        for captures in INDICES_REGEX.captures_iter(&on_demand_content) {
            if let Some(index_match) = captures.get(2) {
                if let Ok(index) = index_match.as_str().parse::<usize>() {
                    key_byte_indices.push(index);
                }
            }
        }

        if key_byte_indices.is_empty() {
            return Err(Error::ParseError("Couldn't get KEY_BYTE indices".into()));
        }

        Ok((key_byte_indices[0], key_byte_indices[1..].to_vec()))
    }

    fn get_key(page: &Html) -> Result<String, Error> {
        let selector = Selector::parse("[name='twitter-site-verification']").unwrap();
        let element = page
            .select(&selector)
            .next()
            .ok_or_else(|| Error::MissingKey("Couldn't get key from the page source".into()))?;

        let key = element
            .value()
            .attr("content")
            .ok_or_else(|| Error::MissingKey("Missing content attribute".into()))?;

        Ok(key.to_string())
    }

    fn get_key_bytes(key: &str) -> Result<Vec<u8>, Error> {
        Ok(base64_decode(key)?)
    }

    fn get_frames(page: &Html) -> Vec<scraper::ElementRef> {
        let selector = Selector::parse("[id^='loading-x-anim']").unwrap();
        page.select(&selector).collect()
    }

    fn get_2d_array(
        key_bytes: &[u8],
        page: &Html,
        frames: Option<Vec<scraper::ElementRef>>,
    ) -> Result<Vec<Vec<i32>>, Error> {
        let frames = frames.unwrap_or_else(|| Self::get_frames(page));

        let frame_index = (key_bytes[5] % 4) as usize;
        if frame_index >= frames.len() {
            return Err(Error::ParseError("Invalid frame index".into()));
        }

        let frame = frames[frame_index];

        let mut outer_children = frame.children();
        let first_child = outer_children
            .next()
            .ok_or_else(|| Error::ParseError("No first child in frame".into()))?;
        let first_child = scraper::ElementRef::wrap(first_child)
            .ok_or_else(|| Error::ParseError("First child is not an element".into()))?;

        let mut inner_children = first_child.children();
        let path_node = inner_children
            .nth(1)
            .ok_or_else(|| Error::ParseError("No second child in an inner group".into()))?;
        let path_elem = scraper::ElementRef::wrap(path_node)
            .ok_or_else(|| Error::ParseError("Second child is not an element".into()))?;

        let d_attr = path_elem
            .value()
            .attr("d")
            .ok_or_else(|| Error::ParseError("Missing 'd' attribute".into()))?;
        let d_content = d_attr
            .get(9..)
            .ok_or_else(|| Error::ParseError("Path data too short".into()))?;

        let segments = d_content.split('C');

        let mut result = Vec::new();
        for segment in segments {
            let numbers: Vec<i32> = segment
                .replace(|c: char| !c.is_ascii_digit() && c != '-', " ")
                .split_whitespace()
                .filter_map(|s| s.parse::<i32>().ok())
                .collect();

            result.push(numbers);
        }

        Ok(result)
    }

    fn solve(value: f64, min_val: f64, max_val: f64, rounding: bool) -> f64 {
        let result = value * (max_val - min_val) / 255.0 + min_val;
        if rounding {
            result.floor()
        } else {
            (result * 100.0).round() / 100.0
        }
    }

    fn animate(frames: &[i32], target_time: f64) -> String {
        let from_color: Vec<f64> = frames[..3]
            .iter()
            .map(|&i| i as f64)
            .chain(std::iter::once(1.0))
            .collect();
        let to_color: Vec<f64> = frames[3..6]
            .iter()
            .map(|&i| i as f64)
            .chain(std::iter::once(1.0))
            .collect();
        let from_rotation = vec![0.0];
        let to_rotation = vec![Self::solve(frames[6] as f64, 60.0, 360.0, true)];

        let curves: Vec<f64> = frames[7..]
            .iter()
            .enumerate()
            .map(|(i, &val)| Self::solve(val as f64, is_odd(i as i32), 1.0, false))
            .collect();

        let cubic = Cubic::new(curves);
        let val = cubic.get_value(target_time);

        let color = interpolate(&from_color, &to_color, val).unwrap();
        let color: Vec<f64> = color.iter().map(|&v| v.clamp(0.0, 255.0)).collect();

        let rotation = interpolate(&from_rotation, &to_rotation, val).unwrap();
        let matrix = convert_rotation_to_matrix(rotation[0]);

        let mut str_arr = Vec::new();

        // Add color values as hex
        for value in &color[..color.len() - 1] {
            str_arr.push(format!("{:x}", value.round() as i32));
        }

        // Add matrix values as hex
        for value in matrix {
            let rounded = (value * 100.0).round() / 100.0;
            let abs_value = rounded.abs();
            let hex_value = float_to_hex(abs_value);

            if hex_value.starts_with('.') {
                str_arr.push(format!("0{}", hex_value.to_lowercase()));
            } else if hex_value.is_empty() {
                str_arr.push("0".to_string());
            } else {
                str_arr.push(hex_value.to_lowercase());
            }
        }

        // Add final zeros
        str_arr.push("0".to_string());
        str_arr.push("0".to_string());

        // Join and remove dots and dashes
        let animation_key = str_arr.join("");
        animation_key.replace(['.', '-'], "")
    }

    fn get_animation_key(
        key_bytes: &[u8],
        page: &Html,
        row_index: usize,
        key_bytes_indices: &[usize],
    ) -> Result<String, Error> {
        let total_time = 4096.0;

        let row_index_value = (key_bytes[row_index] % 16) as usize;

        let frame_time = key_bytes_indices
            .iter()
            .map(|&index| (key_bytes[index] % 16) as f64)
            .fold(1.0, |acc, val| acc * val);

        let frame_time = js_round(frame_time / 10.0) * 10.0;

        let arr = Self::get_2d_array(key_bytes, page, None)?;

        if row_index_value >= arr.len() {
            return Err(Error::ParseError("Invalid row index".into()));
        }

        let frame_row = &arr[row_index_value];

        let target_time = frame_time / total_time;
        let animation_key = Self::animate(frame_row, target_time);

        Ok(animation_key)
    }

    pub fn generate_transaction_id(&self, method: &str, path: &str) -> Result<String, Error> {
        let time_now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .saturating_sub(1682924400) as u32;

        let time_now_bytes = [
            (time_now & 0xFF) as u8,
            ((time_now >> 8) & 0xFF) as u8,
            ((time_now >> 16) & 0xFF) as u8,
            ((time_now >> 24) & 0xFF) as u8,
        ];

        let hash_input = format!(
            "{}!{}!{}{}{}",
            method, path, time_now, self.default_keyword, self.animation_key
        );

        let mut hasher = Sha256::new();
        hasher.update(hash_input.as_bytes());
        let hash_result = hasher.finalize();

        // Convert the first 16 bytes of the hash to a vector
        let hash_bytes: Vec<u8> = hash_result[..16].to_vec();

        // Generate a random number between 0 and 255
        let random_num = rand::random::<u8>();

        // Combine all bytes and XOR with random number
        let mut bytes_arr =
            Vec::with_capacity(self.key_bytes.len() + time_now_bytes.len() + hash_bytes.len() + 1);
        bytes_arr.extend_from_slice(&self.key_bytes);
        bytes_arr.extend_from_slice(&time_now_bytes);
        bytes_arr.extend_from_slice(&hash_bytes);
        bytes_arr.push(self.additional_random_number);

        // Create an output array starting with the random number
        let mut out = vec![random_num];

        // XOR all bytes with the random number
        out.extend(bytes_arr.iter().map(|&b| b ^ random_num));

        // Base64 encode and remove padding
        let encoded = base64_encode(&out);
        let result = encoded.trim_end_matches('=');

        Ok(result.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::blocking::Client;

    #[test]
    #[ignore = "Ignoring network-dependent test"]
    fn test_transaction_id_generation() -> Result<(), Error> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36")
            .build()?;

        let transaction = ClientTransaction::new(&client)?;
        let transaction_id =
            transaction.generate_transaction_id("GET", "/i/api/1.1/jot/client_event.json")?;

        // Verify we get a non-empty string
        assert!(!transaction_id.is_empty());

        Ok(())
    }
}
