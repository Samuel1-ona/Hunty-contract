use soroban_sdk::{Env, String};

/// Stack buffer size for intermediate byte validation in [`StringSanitizer::sanitize`].
///
/// All caller-facing `max_bytes` values (e.g. `MAX_DESCRIPTION_BYTES`,
/// `MAX_QUESTION_LENGTH` in `lib.rs`) **must** stay `<= SANITIZE_STACK_CAP`.
/// Passing `max_bytes > SANITIZE_STACK_CAP` is a programming error and returns
/// [`SanitizeError::LimitTooLarge`], not [`SanitizeError::ExceedsMaxBytes`].
pub const SANITIZE_STACK_CAP: usize = 2048;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SanitizeError {
    Empty,
    /// Input byte length is greater than the caller-supplied `max_bytes`.
    ExceedsMaxBytes,
    /// Caller requested `max_bytes` larger than [`SANITIZE_STACK_CAP`].
    LimitTooLarge,
    InvalidUtf8,
    ControlCharacter,
}

pub struct StringSanitizer;

impl StringSanitizer {
    /// Validates UTF-8, rejects disallowed control characters, and enforces a byte limit.
    pub fn sanitize<const MAX_BYTES: u32>(
        env: &Env,
        input: &String,
        allow_empty: bool,
    ) -> Result<String, SanitizeError> {
        Self::sanitize_runtime(env, input, MAX_BYTES, allow_empty)
    }

    pub fn sanitize_runtime(
        env: &Env,
        input: &String,
        max_bytes: u32,
        allow_empty: bool,
    ) -> Result<String, SanitizeError> {
        // Distinguish programming errors (limit > stack CAP) from oversized user input.
        if (max_bytes as usize) > SANITIZE_STACK_CAP {
            return Err(SanitizeError::LimitTooLarge);
        }

        let byte_len = input.len();

        if byte_len > max_bytes {
            return Err(SanitizeError::ExceedsMaxBytes);
        }

        let len = byte_len as usize;
        // Safe: max_bytes <= CAP and byte_len <= max_bytes, so len <= CAP.
        let mut buf = [0u8; SANITIZE_STACK_CAP];
        input.copy_into_slice(&mut buf[..len]);

        if !is_valid_utf8(&buf[..len]) {
            return Err(SanitizeError::InvalidUtf8);
        }

        for &b in &buf[..len] {
            if is_disallowed_control(b) {
                return Err(SanitizeError::ControlCharacter);
            }
        }

        let mut start = 0;
        let mut end = len;

        while start < end && is_ascii_whitespace(buf[start]) {
            start += 1;
        }

        while end > start && is_ascii_whitespace(buf[end - 1]) {
            end -= 1;
        }

        if start == end {
            if allow_empty {
                return Ok(String::from_str(env, ""));
            }

            return Err(SanitizeError::Empty);
        }

        Ok(String::from_bytes(env, &buf[start..end]))
    }
}

fn is_ascii_whitespace(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r')
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
        let result = StringSanitizer::sanitize_runtime(&env, &input, 100, false);
        assert_eq!(result, Err(SanitizeError::ControlCharacter));
    }

    #[test]
    fn test_sanitize_rejects_whitespace_only() {
        let env = Env::default();
        let input = String::from_str(&env, " ");

        let result = StringSanitizer::sanitize_runtime(&env, &input, 200, false);

        assert_eq!(result, Err(SanitizeError::Empty));
    }

    #[test]
    fn test_sanitize_trims_ascii_whitespace() {
        let env = Env::default();
        let input = String::from_str(&env, " hi ");

        let result = StringSanitizer::sanitize_runtime(&env, &input, 200, false);

        assert_eq!(result, Ok(String::from_str(&env, "hi")));
    }

    #[test]
    fn test_sanitize_enforces_byte_limit() {
        let env = Env::default();
        let input = String::from_str(&env, &"a".repeat(201));
        let result = StringSanitizer::sanitize_runtime(&env, &input, 200, false);
        assert_eq!(result, Err(SanitizeError::ExceedsMaxBytes));
    }

    #[test]
    fn test_sanitize_allows_whitespace_controls() {
        let env = Env::default();
        let input = String::from_str(&env, "line\nbreak");
        let result = StringSanitizer::sanitize_runtime(&env, &input, 100, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_max_bytes_above_cap_is_limit_too_large() {
        let env = Env::default();
        let input = String::from_str(&env, "ok");
        let over = (SANITIZE_STACK_CAP as u32).saturating_add(1);
        let result = StringSanitizer::sanitize_runtime(&env, &input, over, false);
        assert_eq!(result, Err(SanitizeError::LimitTooLarge));
    }

    #[test]
    fn test_input_over_max_bytes_is_exceeds_not_limit_too_large() {
        let env = Env::default();
        let input = String::from_str(&env, &"a".repeat(50));
        let result = StringSanitizer::sanitize_runtime(&env, &input, 40, false);
        assert_eq!(result, Err(SanitizeError::ExceedsMaxBytes));
    }
}
