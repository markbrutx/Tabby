/// Extracts valid UTF-8 from raw bytes, carrying incomplete sequences across reads.
///
/// Prepends any leftover `carry` bytes from the previous read, scans for an
/// incomplete multi-byte sequence at the tail, moves it into `carry`, and
/// returns the valid UTF-8 prefix as a `String`.
pub fn extract_valid_utf8(carry: &mut Vec<u8>, raw: &[u8]) -> String {
    let mut buf = std::mem::take(carry);
    buf.extend_from_slice(raw);

    if buf.is_empty() {
        return String::new();
    }

    // Find the boundary: walk backwards to detect an incomplete trailing sequence.
    let valid_up_to = match std::str::from_utf8(&buf) {
        Ok(_) => buf.len(),
        Err(error) => {
            // If there's an error length, the bytes are truly invalid (not just
            // incomplete). We still split at the valid boundary — the invalid
            // bytes become carry and will be re-evaluated with the next read.
            error.valid_up_to()
        }
    };

    let remainder = buf.split_off(valid_up_to);
    *carry = remainder;

    // SAFETY: we split at a validated UTF-8 boundary.
    unsafe { String::from_utf8_unchecked(buf) }
}

#[cfg(test)]
mod tests {
    use super::extract_valid_utf8;

    #[test]
    fn ascii_only_no_carry() {
        let mut carry = Vec::new();
        let result = extract_valid_utf8(&mut carry, b"hello world");
        assert_eq!(result, "hello world");
        assert!(carry.is_empty());
    }

    #[test]
    fn complete_multibyte_passes_through() {
        let mut carry = Vec::new();
        let emoji = "\u{1F600}"; // 4-byte char
        let result = extract_valid_utf8(&mut carry, emoji.as_bytes());
        assert_eq!(result, emoji);
        assert!(carry.is_empty());
    }

    #[test]
    fn split_at_multibyte_boundary() {
        let mut carry = Vec::new();
        // \u{00E9} = 0xC3 0xA9 (2 bytes). Send only first byte.
        let result = extract_valid_utf8(&mut carry, &[0xC3]);
        assert_eq!(result, "");
        assert_eq!(carry, vec![0xC3]);
    }

    #[test]
    fn carry_prepended_to_next_read() {
        let mut carry = vec![0xC3]; // leftover from previous read
        let result = extract_valid_utf8(&mut carry, &[0xA9]); // completes \u{00E9}
        assert_eq!(result, "\u{00E9}");
        assert!(carry.is_empty());
    }

    #[test]
    fn empty_buffer_returns_empty() {
        let mut carry = Vec::new();
        let result = extract_valid_utf8(&mut carry, &[]);
        assert_eq!(result, "");
        assert!(carry.is_empty());
    }

    #[test]
    fn three_byte_char_split_after_first_byte() {
        // \u{4E16} = 0xE4 0xB8 0x96
        let mut carry = Vec::new();
        let result = extract_valid_utf8(&mut carry, &[b'A', 0xE4]);
        assert_eq!(result, "A");
        assert_eq!(carry, vec![0xE4]);

        let result = extract_valid_utf8(&mut carry, &[0xB8, 0x96, b'B']);
        assert_eq!(result, "\u{4E16}B");
        assert!(carry.is_empty());
    }

    #[test]
    fn four_byte_char_split_across_three_reads() {
        // \u{1F600} = 0xF0 0x9F 0x98 0x80
        let mut carry = Vec::new();

        let r1 = extract_valid_utf8(&mut carry, &[0xF0, 0x9F]);
        assert_eq!(r1, "");
        assert_eq!(carry.len(), 2);

        let r2 = extract_valid_utf8(&mut carry, &[0x98]);
        assert_eq!(r2, "");
        assert_eq!(carry.len(), 3);

        let r3 = extract_valid_utf8(&mut carry, &[0x80]);
        assert_eq!(r3, "\u{1F600}");
        assert!(carry.is_empty());
    }
}
