//! Small proxy-authentication helpers shared by HTTP and CONNECT clients.

use crate::i2pcontrol::auth::compare_passwords;

pub fn basic_authorization(username: &str, password: &str) -> String {
    format!(
        "Basic {}",
        standard_base64(format!("{username}:{password}").as_bytes())
    )
}

pub fn credentials_match(header: Option<&str>, username: &str, password: &str) -> bool {
    let Some(header) = header else {
        return false;
    };
    let Some(encoded) = header.strip_prefix("Basic ").or_else(|| header.strip_prefix("basic "))
    else {
        return false;
    };
    let Some(decoded) = standard_base64_decode(encoded.trim()) else {
        return false;
    };
    let Ok(value) = String::from_utf8(decoded) else {
        return false;
    };
    let Some((provided_user, provided_password)) = value.split_once(':') else {
        return false;
    };
    compare_passwords(provided_user, username) && compare_passwords(provided_password, password)
}

fn standard_base64(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let first = chunk[0] as u32;
        let second = chunk.get(1).copied().unwrap_or(0) as u32;
        let third = chunk.get(2).copied().unwrap_or(0) as u32;
        output.push(ALPHABET[((first >> 2) & 0x3f) as usize] as char);
        output.push(ALPHABET[(((first & 3) << 4) | (second >> 4)) as usize] as char);
        output.push(if chunk.len() > 1 {
            ALPHABET[(((second & 0xf) << 2) | (third >> 6)) as usize] as char
        } else {
            '='
        });
        output.push(if chunk.len() > 2 {
            ALPHABET[(third & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    output
}

fn standard_base64_decode(input: &str) -> Option<Vec<u8>> {
    if input.is_empty() || !input.len().is_multiple_of(4) {
        return None;
    }
    let mut output = Vec::with_capacity(input.len() / 4 * 3);
    for chunk in input.as_bytes().chunks_exact(4) {
        let values = [decode_byte(chunk[0])?, decode_byte(chunk[1])?];
        let third = if chunk[2] == b'=' {
            0
        } else {
            decode_byte(chunk[2])?
        };
        let fourth = if chunk[3] == b'=' {
            0
        } else {
            decode_byte(chunk[3])?
        };
        if chunk[2] == b'=' && chunk[3] != b'=' {
            return None;
        }
        output.push((values[0] << 2) | (values[1] >> 4));
        if chunk[2] != b'=' {
            output.push((values[1] << 4) | (third >> 2));
        }
        if chunk[3] != b'=' {
            output.push((third << 6) | fourth);
        }
    }
    Some(output)
}

fn decode_byte(byte: u8) -> Option<u8> {
    Some(match byte {
        b'A'..=b'Z' => byte - b'A',
        b'a'..=b'z' => byte - b'a' + 26,
        b'0'..=b'9' => byte - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_credentials_round_trip_without_exposing_values() {
        let header = basic_authorization("user", "secret");
        assert!(credentials_match(Some(&header), "user", "secret"));
        assert!(!credentials_match(Some(&header), "user", "wrong"));
        assert!(!format!("{header:?}").contains("secret"));
    }
}
