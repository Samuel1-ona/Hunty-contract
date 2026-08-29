use soroban_sdk::{Env, String};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SanitizeError {
    Empty,
    ExceedsMaxBytes,
    InvalidUtf8,
    ControlCharacter,
}

pub struct StringSanitizer;

impl StringSanitizer {
    /// Validates UTF-8, rejects disallowed control characters, and enforces a byte limit.
    pub fn sanitize(
        env: &Env,
        input: &String,
        max_bytes: u32,
        allow_empty: bool,
    ) -> Result<String, SanitizeError> {
        let byte_len = input.len();
        if byte_len == 0 {
            if allow_empty {
                return Ok(String::from_str(env, ""));
            }
            return Err(SanitizeError::Empty);
        }
        if byte_len > max_bytes {
            return Err(SanitizeError::ExceedsMaxBytes);
        }

        const CAP: usize = 2048;
        let len = byte_len as usize;
        if len > CAP {
            return Err(SanitizeError::ExceedsMaxBytes);
        }

        let mut buf = [0u8; CAP];
        input.copy_into_slice(&mut buf[..len]);

        if !is_valid_utf8(&buf[..len]) {
            return Err(SanitizeError::InvalidUtf8);
        }

        for &b in &buf[..len] {
            if is_disallowed_control(b) {
                return Err(SanitizeError::ControlCharacter);
            }
        }

        Ok(String::from_bytes(env, &buf[..len]))
    }
}

fn is_disallowed_control(b: u8) -> bool {
    b < 0x20 && b != b'\t' && b != b'\n' && b != b'\r'
}

fn is_valid_utf8(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b <= 0x7F {
            i += 1;
            continue;
        }
        let remaining = bytes.len() - i;
        if (b & 0xE0) == 0xC0 {
            if remaining < 2 || !is_utf8_continuation(bytes[i + 1]) {
                return false;
            }
            i += 2;
        } else if (b & 0xF0) == 0xE0 {
            if remaining < 3
                || !is_utf8_continuation(bytes[i + 1])
                || !is_utf8_continuation(bytes[i + 2])
            {
                return false;
            }
            i += 3;
        } else if (b & 0xF8) == 0xF0 {
            if remaining < 4
                || !is_utf8_continuation(bytes[i + 1])
                || !is_utf8_continuation(bytes[i + 2])
                || !is_utf8_continuation(bytes[i + 3])
            {
                return false;
            }
            i += 4;
        } else {
            return false;
        }
    }
    true
}

fn is_utf8_continuation(b: u8) -> bool {
    (b & 0xC0) == 0x80
}

#[cfg(test)]
mod test {
    extern crate std;

    use super::*;

    #[test]
    fn test_sanitize_rejects_control_characters() {
        let env = Env::default();
        let input = String::from_str(&env, "hello\x07world");
        let result = StringSanitizer::sanitize(&env, &input, 100, false);
        assert_eq!(result, Err(SanitizeError::ControlCharacter));
    }

    #[test]
    fn test_sanitize_enforces_byte_limit() {
        let env = Env::default();
        let input = String::from_str(&env, &"a".repeat(201));
        let result = StringSanitizer::sanitize(&env, &input, 200, false);
        assert_eq!(result, Err(SanitizeError::ExceedsMaxBytes));
    }

    #[test]
    fn test_sanitize_allows_whitespace_controls() {
        let env = Env::default();
        let input = String::from_str(&env, "line\nbreak");
        let result = StringSanitizer::sanitize(&env, &input, 100, false);
        assert!(result.is_ok());
    }
}
