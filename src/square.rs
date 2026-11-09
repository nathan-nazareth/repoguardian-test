//! Square helpers. Squares are `u8` indices with `0 == a1` and `63 == h8`.

/// The file (0 = `a`) of a square index.
pub const fn file(index: u8) -> i8 {
    (index % 8) as i8
}

/// The rank (0 = `1`) of a square index.
pub const fn rank(index: u8) -> i8 {
    (index / 8) as i8
}

/// Builds a square index from file and rank, returning `None` when out of bounds.
pub fn make(file: i8, rank: i8) -> Option<u8> {
    if (0..8).contains(&file) && (0..8).contains(&rank) {
        Some((rank * 8 + file) as u8)
    } else {
        None
    }
}

/// Converts an algebraic square such as `e4` to an index.
pub fn parse(s: &str) -> Option<u8> {
    let bytes = s.as_bytes();
    if bytes.len() != 2 {
        return None;
    }
    let file_byte = (bytes[0] as char).to_ascii_lowercase() as u8;
    let rank_byte = bytes[1];
    if !(b'a'..=b'h').contains(&file_byte) || !(b'1'..=b'8').contains(&rank_byte) {
        return None;
    }
    make((file_byte - b'a') as i8, (rank_byte - b'1') as i8)
}

/// Renders a square index as algebraic notation.
pub fn name(index: u8) -> String {
    let file_char = (b'a' + file(index) as u8) as char;
    let rank_char = (b'1' + rank(index) as u8) as char;
    format!("{file_char}{rank_char}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_and_parse_round_trip() {
        for index in 0..64u8 {
            assert_eq!(parse(&name(index)), Some(index));
        }
    }

    #[test]
    fn out_of_bounds_is_rejected() {
        assert_eq!(make(-1, 0), None);
        assert_eq!(make(0, 8), None);
        assert_eq!(parse("z9"), None);
        assert_eq!(parse("e"), None);
    }
}
