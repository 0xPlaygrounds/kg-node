const BASE58_ALLOWED_CHARS: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

pub fn encode_uuid_to_base58(val: &str) -> String {
    let val = val.replace("-", "");

    let hex = match u128::from_str_radix(&val, 16) {
        Ok(num) => num,
        Err(_) => return String::new(),
    };
    let mut remainder = hex;
    let mut result = Vec::new();

    while remainder > 0 {
        let mod_val = remainder % 58;
        if let Some(&base58_char) = BASE58_ALLOWED_CHARS.get(mod_val as usize) {
            result.push(base58_char as char);
        }
        remainder /= 58;
    }

    result.reverse();
    result.iter().collect()
}

pub fn decode_base58_to_uuid(encoded: &str) -> Result<String, &'static str> {
    let mut decoded: u128 = 0;

    for char in encoded.chars() {
        let index = BASE58_ALLOWED_CHARS.iter().position(|&c| c == char as u8);
        if let Some(index) = index {
            decoded = decoded * 58 + index as u128;
        } else {
            return Err("Invalid Base58 character");
        }
    }

    let hex_str = format!("{:032x}", decoded);
    Ok(format!(
        "{}-{}-{}-{}-{}",
        &hex_str[0..8],
        &hex_str[8..12],
        &hex_str[12..16],
        &hex_str[16..20],
        &hex_str[20..32]
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base58_encoding() {
        assert_eq!(
            "4Z6VLmpipszCVZb21Fey5F",
            encode_uuid_to_base58("1cc6995f-6cc2-4c7a-9592-1466bf95f6be")
        )
    }

    #[test]
    fn test_base58_decoding() {
        assert_eq!(
            "1cc6995f-6cc2-4c7a-9592-1466bf95f6be",
            decode_base58_to_uuid("4Z6VLmpipszCVZb21Fey5F").unwrap()
        )
    }

    #[test]
    fn test_encode_decode() {
        let uuid = "1cc6995f-6cc2-4c7a-9592-1466bf95f6be";
        let encoded = encode_uuid_to_base58(uuid);
        let decoded = decode_base58_to_uuid(&encoded).unwrap();
        assert_eq!(uuid, decoded);
    }
}
