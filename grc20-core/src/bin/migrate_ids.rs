use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use grc20_core::ids::base58::decode_base58_to_uuid;

/// Parses a line for a base58-encoded ID and returns the decoded UUID if found.
/// Returns (prefix, uuid, suffix) if a base58 ID is found, otherwise None.
fn parse_id_line(line: &str) -> Option<(String, String, String)> {
    // Match lines like: pub const PROPERTY_TYPE: &str = "GscJ2GELQjmLoaVrYyR3xm";
    let parts: Vec<&str> = line.split('=').collect();
    if parts.len() != 2 {
        return None;
    }
    let prefix = parts[0].to_string();
    let value_part = parts[1].trim();

    // Find quoted string
    if let Some(start) = value_part.find('"') {
        if let Some(end) = value_part[start + 1..].find('"') {
            let base58_id = &value_part[start + 1..start + 1 + end];
            // Try to decode
            match decode_base58_to_uuid(base58_id) {
                Ok(uuid) => {
                    let suffix = &value_part[start + 1 + end + 1..];
                    return Some((prefix, uuid, suffix.to_string()));
                }
                Err(_) => return None,
            }
        }
    }
    None
}

fn main() {
    let input_path = Path::new("src/ids/indexer_ids.rs");
    let output_path = Path::new("src/ids/indexer_ids_v2.rs");

    let input_file = File::open(&input_path).expect("Failed to open indexer_ids.rs for reading");
    let reader = BufReader::new(input_file);

    let mut output_file =
        File::create(&output_path).expect("Failed to create indexer_ids_v2.rs for writing");

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        if let Some((prefix, uuid, suffix)) = parse_id_line(&line) {
            // Write the migrated line with UUID
            writeln!(
                output_file,
                "{} = \"{}\"{}",
                prefix.trim_end(),
                uuid,
                suffix
            )
            .expect("Failed to write migrated line");
        } else {
            // Write the line as-is
            writeln!(output_file, "{}", line).expect("Failed to write line");
        }
    }

    println!("Migration complete. Output written to indexer_ids_v2.rs");
}
