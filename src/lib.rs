//! Deterministic, memorable, BIP39-encoded fingerprints of digests.
//!
//! Same input always produces the same dash-separated word string. This is a
//! lossy fingerprint — it does NOT preserve every input bit, and the original
//! digest is NOT recoverable from the output. The point is a short, memorable
//! label for a hash you can paste into Slack or read aloud on a call.
//!
//! ```
//! use humanhash::humanize;
//! let tag = humanize("550e8400-e29b-41d4-a716-446655440000").unwrap();
//! assert_eq!(tag.split('-').count(), 4);
//! ```
use std::fmt;
use std::sync::LazyLock;

const BITS_PER_WORD: usize = 11;
const WORDLIST_LEN: usize = 1 << BITS_PER_WORD;

/// Maximum supported word count. Bounded by the 64-bit FNV-1a accumulator
/// (5 * 11 = 55 bits ≤ 64).
pub const MAX_WORDS: u8 = 5;

/// Default word count. 4 words = 44 bits ≈ 17.6 trillion fingerprints —
/// plenty for human-recognition use cases.
pub const DEFAULT_WORDS: u8 = 4;

const BIP39_RAW: &str = include_str!("../dicts/bip39.txt");

static WORDLIST: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    let words: Vec<&'static str> = BIP39_RAW
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    assert_eq!(
        words.len(),
        WORDLIST_LEN,
        "BIP-0039 wordlist must have exactly {WORDLIST_LEN} entries (got {})",
        words.len(),
    );
    words
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HumanhashError {
    /// Input contained no usable bytes (empty or whitespace-only).
    EmptyInput,
    /// Input contained a character that isn't a hex digit (after stripping
    /// `0x`, `urn:uuid:`, dashes, and whitespace).
    InvalidHexCharacter(char),
    /// Requested word count was 0 or greater than [`MAX_WORDS`].
    InvalidWordCount(u8),
}

impl fmt::Display for HumanhashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "input has zero usable bytes"),
            Self::InvalidHexCharacter(c) => write!(f, "invalid hex character: {c:?}"),
            Self::InvalidWordCount(n) => write!(
                f,
                "word count must be in 1..={MAX_WORDS}, got {n}",
                MAX_WORDS = MAX_WORDS,
            ),
        }
    }
}

impl std::error::Error for HumanhashError {}

#[derive(Debug, Clone)]
pub struct HumanizeOptions<'a> {
    /// Number of words in the output. Must be in `1..=MAX_WORDS`.
    pub words: u8,
    pub separator: &'a str,
}

impl Default for HumanizeOptions<'_> {
    fn default() -> Self {
        Self {
            words: DEFAULT_WORDS,
            separator: "-",
        }
    }
}

/// Render a memorable fingerprint of `input`, with default options
/// (4 words, `-` separator).
///
/// `input` is treated as a hex string. The following are tolerated and
/// stripped before parsing:
/// - leading `0x` / `0X` prefix
/// - leading `urn:uuid:` prefix
/// - any `-` characters (UUID dashes)
/// - any whitespace
///
/// An odd hex-character count is padded with a trailing `0` nibble.
pub fn humanize(input: &str) -> Result<String, HumanhashError> {
    humanize_with(input, HumanizeOptions::default())
}

/// Like [`humanize`] but with caller-controlled options.
pub fn humanize_with(input: &str, opts: HumanizeOptions<'_>) -> Result<String, HumanhashError> {
    let bytes = parse(input)?;
    render(&bytes, &opts)
}

/// Render a memorable fingerprint directly from raw bytes.
pub fn humanize_bytes(bytes: &[u8]) -> Result<String, HumanhashError> {
    humanize_bytes_with(bytes, HumanizeOptions::default())
}

/// Like [`humanize_bytes`] but with caller-controlled options.
pub fn humanize_bytes_with(
    bytes: &[u8],
    opts: HumanizeOptions<'_>,
) -> Result<String, HumanhashError> {
    if bytes.is_empty() {
        return Err(HumanhashError::EmptyInput);
    }
    render(bytes, &opts)
}

fn parse(input: &str) -> Result<Vec<u8>, HumanhashError> {
    let trimmed = input.trim();
    let body = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    let body = body.strip_prefix("urn:uuid:").unwrap_or(body);
    let cleaned: String = body
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect();
    if cleaned.is_empty() {
        return Err(HumanhashError::EmptyInput);
    }
    let padded = if cleaned.len().is_multiple_of(2) {
        cleaned
    } else {
        format!("{cleaned}0")
    };
    hex_to_bytes(&padded)
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, HumanhashError> {
    let lower: String = hex.chars().map(|c| c.to_ascii_lowercase()).collect();
    debug_assert_eq!(lower.len() % 2, 0);
    let mut out = Vec::with_capacity(lower.len() / 2);
    let bytes_in = lower.as_bytes();
    let mut i = 0;
    while i < bytes_in.len() {
        let hi = nibble(bytes_in[i] as char)?;
        let lo = nibble(bytes_in[i + 1] as char)?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(out)
}

fn nibble(c: char) -> Result<u8, HumanhashError> {
    match c {
        '0'..='9' => Ok(c as u8 - b'0'),
        'a'..='f' => Ok(c as u8 - b'a' + 10),
        _ => Err(HumanhashError::InvalidHexCharacter(c)),
    }
}

fn render(bytes: &[u8], opts: &HumanizeOptions<'_>) -> Result<String, HumanhashError> {
    if !(1..=MAX_WORDS).contains(&opts.words) {
        return Err(HumanhashError::InvalidWordCount(opts.words));
    }
    let n_words = opts.words as usize;
    let total_bits = n_words * BITS_PER_WORD;
    let value = fold(bytes, total_bits);
    let wordlist = &*WORDLIST;
    let mut parts: Vec<&str> = Vec::with_capacity(n_words);
    for i in 0..n_words {
        let shift = (n_words - 1 - i) * BITS_PER_WORD;
        let idx = ((value >> shift) & ((1u64 << BITS_PER_WORD) - 1)) as usize;
        parts.push(wordlist[idx]);
    }
    Ok(parts.join(opts.separator))
}

/// FNV-1a 64-bit deterministic hash, masked to `target_bits`.
///
/// Used as a uniform mixer so that small input differences avalanche into
/// large fingerprint differences, and short inputs don't degenerate.
fn fold(bytes: &[u8], target_bits: usize) -> u64 {
    debug_assert!((1..=64).contains(&target_bits));
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let prime: u64 = 0x0000_0100_0000_01b3;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(prime);
    }
    if target_bits == 64 {
        h
    } else {
        h & ((1u64 << target_bits) - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wordlist_loads_at_2048() {
        assert_eq!(WORDLIST.len(), 2048);
        assert_eq!(WORDLIST[0], "abandon");
        assert_eq!(WORDLIST[2047], "zoo");
    }

    #[test]
    fn default_is_four_words_dash_separated() {
        let out = humanize("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(out.split('-').count(), 4);
    }

    #[test]
    fn determinism_same_input_same_output() {
        let a = humanize("e3b0c44298fc1c149afbf4c8996fb924").unwrap();
        let b = humanize("e3b0c44298fc1c149afbf4c8996fb924").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn case_insensitive_hex() {
        let lower = humanize("0123456789abcdef0123456789abcdef").unwrap();
        let upper = humanize("0123456789ABCDEF0123456789ABCDEF").unwrap();
        assert_eq!(lower, upper);
    }

    #[test]
    fn zero_x_prefix_stripped() {
        let with_prefix = humanize("0x0123456789abcdef").unwrap();
        let without = humanize("0123456789abcdef").unwrap();
        assert_eq!(with_prefix, without);
    }

    #[test]
    fn uuid_with_dashes_matches_concatenated() {
        let dashed = humanize("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let flat = humanize("550e8400e29b41d4a716446655440000").unwrap();
        assert_eq!(dashed, flat);
    }

    #[test]
    fn urn_uuid_prefix_stripped() {
        let urn = humanize("urn:uuid:550e8400-e29b-41d4-a716-446655440000").unwrap();
        let plain = humanize("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(urn, plain);
    }

    #[test]
    fn whitespace_tolerated() {
        let messy = humanize("  0123 4567 89ab cdef  ").unwrap();
        let clean = humanize("0123456789abcdef").unwrap();
        assert_eq!(messy, clean);
    }

    #[test]
    fn git_short_sha_seven_hex_chars_renders_four_words() {
        let out = humanize("ac84a4a").unwrap();
        assert_eq!(out.split('-').count(), 4);
    }

    #[test]
    fn custom_separator() {
        let out = humanize_with(
            "0123456789abcdef0123456789abcdef",
            HumanizeOptions {
                words: 4,
                separator: " ",
            },
        )
        .unwrap();
        assert!(!out.contains('-'));
        assert_eq!(out.matches(' ').count(), 3);
    }

    #[test]
    fn three_words_works() {
        let out = humanize_with(
            "0123456789abcdef0123456789abcdef",
            HumanizeOptions {
                words: 3,
                separator: "-",
            },
        )
        .unwrap();
        assert_eq!(out.split('-').count(), 3);
    }

    #[test]
    fn rejects_zero_words() {
        let err = humanize_with(
            "0123456789abcdef",
            HumanizeOptions {
                words: 0,
                separator: "-",
            },
        );
        assert_eq!(err, Err(HumanhashError::InvalidWordCount(0)));
    }

    #[test]
    fn rejects_too_many_words() {
        let err = humanize_with(
            "0123456789abcdef",
            HumanizeOptions {
                words: 6,
                separator: "-",
            },
        );
        assert_eq!(err, Err(HumanhashError::InvalidWordCount(6)));
    }

    #[test]
    fn rejects_invalid_hex_character() {
        let err = humanize("xyz123");
        assert!(matches!(err, Err(HumanhashError::InvalidHexCharacter(_))));
    }

    #[test]
    fn rejects_empty_input() {
        assert_eq!(humanize(""), Err(HumanhashError::EmptyInput));
        assert_eq!(humanize("   "), Err(HumanhashError::EmptyInput));
        assert_eq!(humanize("---"), Err(HumanhashError::EmptyInput));
    }

    #[test]
    fn humanize_bytes_matches_hex_string_on_same_bytes() {
        let bytes = [
            0x55, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4, 0xa7, 0x16, 0x44, 0x66, 0x55, 0x44,
            0x00, 0x00,
        ];
        let from_bytes = humanize_bytes(&bytes).unwrap();
        let from_hex = humanize("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(from_bytes, from_hex);
    }

    #[test]
    fn humanize_bytes_rejects_empty() {
        assert_eq!(humanize_bytes(&[]), Err(HumanhashError::EmptyInput));
    }

    #[test]
    fn single_bit_input_change_avalanches_output() {
        let a = humanize("0123456789abcdef0123456789abcdef").unwrap();
        let b = humanize("0123456789abcdef0123456789abcdee").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn each_word_is_in_wordlist() {
        let out = humanize("0123456789abcdef0123456789abcdef").unwrap();
        for word in out.split('-') {
            assert!(
                WORDLIST.contains(&word),
                "word {word:?} not in BIP-0039 wordlist",
            );
        }
    }
}
