use std::fmt;
use std::sync::LazyLock;

const BITS_PER_WORD: usize = 11;
const WORDLIST_LEN: usize = 1 << BITS_PER_WORD;

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
    EmptyInput,
    InvalidLength {
        expected_chars: usize,
        got_chars: usize,
    },
    InvalidUuidShape,
    InvalidHexCharacter(char),
}

impl fmt::Display for HumanhashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "input has zero bytes"),
            Self::InvalidLength {
                expected_chars,
                got_chars,
            } => write!(
                f,
                "expected exactly {expected_chars} hex characters after normalization, got {got_chars}",
            ),
            Self::InvalidUuidShape => {
                write!(
                    f,
                    "input is not a valid UUID (expected 8-4-4-4-12 hex form)"
                )
            }
            Self::InvalidHexCharacter(c) => write!(f, "invalid hex character: {c:?}"),
        }
    }
}

impl std::error::Error for HumanhashError {}

#[derive(Debug, Clone, Copy)]
pub enum HashInput<'a> {
    /// 7-hex-char abbreviated git SHA (28 bits).
    GitShort7(&'a str),
    /// 32-hex-char MD5 digest (128 bits).
    Md5(&'a str),
    /// 40-hex-char SHA-1 digest (160 bits).
    Sha1(&'a str),
    /// 40-hex-char full git SHA-1 (160 bits). Byte-identical to `Sha1`; the
    /// distinction is purely caller intent.
    GitLong(&'a str),
    /// 64-hex-char SHA-256 digest (256 bits).
    Sha256(&'a str),
    /// UUID v1-v5 in canonical 8-4-4-4-12 form (128 bits).
    Uuid(&'a str),
}

#[derive(Debug, Clone)]
pub struct HumanizeOptions<'a> {
    pub separator: &'a str,
}

impl Default for HumanizeOptions<'_> {
    fn default() -> Self {
        Self { separator: "-" }
    }
}

pub fn humanize(input: HashInput<'_>) -> Result<String, HumanhashError> {
    humanize_with(input, HumanizeOptions::default())
}

pub fn humanize_with(
    input: HashInput<'_>,
    opts: HumanizeOptions<'_>,
) -> Result<String, HumanhashError> {
    let parsed = parse(input)?;
    Ok(render(&parsed.bytes, parsed.bit_count, &opts))
}

pub fn humanize_bytes(bytes: &[u8]) -> Result<String, HumanhashError> {
    humanize_bytes_with(bytes, HumanizeOptions::default())
}

pub fn humanize_bytes_with(
    bytes: &[u8],
    opts: HumanizeOptions<'_>,
) -> Result<String, HumanhashError> {
    if bytes.is_empty() {
        return Err(HumanhashError::EmptyInput);
    }
    Ok(render(bytes, bytes.len() * 8, &opts))
}

struct Parsed {
    bytes: Vec<u8>,
    bit_count: usize,
}

fn parse(input: HashInput<'_>) -> Result<Parsed, HumanhashError> {
    match input {
        HashInput::GitShort7(s) => parse_strict_hex(s, 7),
        HashInput::Md5(s) => parse_strict_hex(s, 32),
        HashInput::Sha1(s) | HashInput::GitLong(s) => parse_strict_hex(s, 40),
        HashInput::Sha256(s) => parse_strict_hex(s, 64),
        HashInput::Uuid(s) => parse_uuid(s),
    }
}

fn parse_strict_hex(input: &str, expected_chars: usize) -> Result<Parsed, HumanhashError> {
    let body = input
        .strip_prefix("0x")
        .or_else(|| input.strip_prefix("0X"))
        .unwrap_or(input);
    let cleaned: String = body.chars().filter(|c| !c.is_whitespace()).collect();
    if cleaned.len() != expected_chars {
        return Err(HumanhashError::InvalidLength {
            expected_chars,
            got_chars: cleaned.len(),
        });
    }
    let bit_count = expected_chars * 4;
    let padded = if cleaned.len().is_multiple_of(2) {
        cleaned
    } else {
        format!("{cleaned}0")
    };
    let bytes = hex_to_bytes(&padded)?;
    Ok(Parsed { bytes, bit_count })
}

fn parse_uuid(input: &str) -> Result<Parsed, HumanhashError> {
    let body = input.strip_prefix("urn:uuid:").unwrap_or(input);
    let cleaned: String = body.chars().filter(|c| !c.is_whitespace()).collect();
    let groups: Vec<&str> = cleaned.split('-').collect();
    if groups.len() == 5 && groups.iter().map(|g| g.len()).eq([8, 4, 4, 4, 12]) {
        let joined: String = groups.concat();
        let bytes = hex_to_bytes(&joined)?;
        return Ok(Parsed {
            bit_count: bytes.len() * 8,
            bytes,
        });
    }
    if cleaned.len() == 32 && !cleaned.contains('-') {
        let bytes = hex_to_bytes(&cleaned)?;
        return Ok(Parsed {
            bit_count: bytes.len() * 8,
            bytes,
        });
    }
    Err(HumanhashError::InvalidUuidShape)
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

fn render(bytes: &[u8], total_bits: usize, opts: &HumanizeOptions<'_>) -> String {
    let words_count = total_bits / BITS_PER_WORD;
    let remainder_bits = total_bits - words_count * BITS_PER_WORD;
    let wordlist = &*WORDLIST;

    let mut parts: Vec<String> = Vec::with_capacity(words_count + 1);
    for i in 0..words_count {
        let idx = read_bits(bytes, i * BITS_PER_WORD, BITS_PER_WORD) as usize;
        parts.push(wordlist[idx].to_string());
    }
    if remainder_bits > 0 {
        let value = read_bits(bytes, words_count * BITS_PER_WORD, remainder_bits);
        let hex_chars = remainder_bits.div_ceil(4);
        parts.push(format!("{value:0width$x}", width = hex_chars));
    }
    parts.join(opts.separator)
}

fn read_bits(bytes: &[u8], bit_offset: usize, n: usize) -> u32 {
    debug_assert!((1..=32).contains(&n));
    let mut value: u64 = 0;
    let mut bits_collected = 0usize;
    let mut cursor = bit_offset;
    while bits_collected < n {
        let byte_idx = cursor / 8;
        let bit_idx_in_byte = cursor % 8;
        let byte = bytes[byte_idx] as u64;
        let bits_left_in_byte = 8 - bit_idx_in_byte;
        let bits_needed = n - bits_collected;
        let take = bits_left_in_byte.min(bits_needed);
        let shift_right = bits_left_in_byte - take;
        let mask: u64 = (1u64 << take) - 1;
        let chunk = (byte >> shift_right) & mask;
        value = (value << take) | chunk;
        bits_collected += take;
        cursor += take;
    }
    value as u32
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
    fn git_short_7_renders_two_words_plus_tail() {
        let out = humanize(HashInput::GitShort7("ac84a4a")).unwrap();
        let parts: Vec<&str> = out.split('-').collect();
        // 28 bits = 2*11 words + 6 tail bits = 3 tokens. Tail = ceil(6/4) = 2 hex chars.
        assert_eq!(parts.len(), 3, "got {out}");
        assert_eq!(parts[2].len(), 2);
    }

    #[test]
    fn md5_renders_eleven_words_plus_tail() {
        let out = humanize(HashInput::Md5("0123456789abcdef0123456789abcdef")).unwrap();
        let parts: Vec<&str> = out.split('-').collect();
        // 128 bits = 11*11 + 7 tail bits = 12 tokens. Tail = ceil(7/4) = 2 hex chars.
        assert_eq!(parts.len(), 12, "got {out}");
        assert_eq!(parts[11].len(), 2);
    }

    #[test]
    fn sha1_renders_fourteen_words_plus_tail() {
        let out = humanize(HashInput::Sha1("0123456789abcdef0123456789abcdef01234567")).unwrap();
        let parts: Vec<&str> = out.split('-').collect();
        // 160 bits = 14*11 + 6 tail bits = 15 tokens. Tail = ceil(6/4) = 2 hex chars.
        assert_eq!(parts.len(), 15, "got {out}");
        assert_eq!(parts[14].len(), 2);
    }

    #[test]
    fn sha256_renders_twentythree_words_plus_tail() {
        let out = humanize(HashInput::Sha256(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        ))
        .unwrap();
        let parts: Vec<&str> = out.split('-').collect();
        // 256 bits = 23*11 + 3 tail bits = 24 tokens. Tail = ceil(3/4) = 1 hex char.
        assert_eq!(parts.len(), 24, "got {out}");
        assert_eq!(parts[23].len(), 1);
    }

    #[test]
    fn uuid_with_dashes_matches_md5_byte_shape() {
        let with_dashes =
            humanize(HashInput::Uuid("550e8400-e29b-41d4-a716-446655440000")).unwrap();
        let as_md5_bytes = humanize(HashInput::Md5("550e8400e29b41d4a716446655440000")).unwrap();
        assert_eq!(with_dashes, as_md5_bytes);
    }

    #[test]
    fn determinism_per_input_type() {
        let sha = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let a = humanize(HashInput::Sha256(sha)).unwrap();
        let b = humanize(HashInput::Sha256(sha)).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn case_insensitive_hex_input() {
        let lower = humanize(HashInput::Md5("0123456789abcdef0123456789abcdef")).unwrap();
        let upper = humanize(HashInput::Md5("0123456789ABCDEF0123456789ABCDEF")).unwrap();
        assert_eq!(lower, upper);
    }

    #[test]
    fn rejects_wrong_length_for_type() {
        assert_eq!(
            humanize(HashInput::Sha256("deadbeef")),
            Err(HumanhashError::InvalidLength {
                expected_chars: 64,
                got_chars: 8
            })
        );
    }

    #[test]
    fn rejects_invalid_uuid_shape() {
        assert_eq!(
            humanize(HashInput::Uuid("not-a-uuid")),
            Err(HumanhashError::InvalidUuidShape)
        );
    }

    #[test]
    fn sha1_and_git_long_produce_identical_output() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        assert_eq!(
            humanize(HashInput::Sha1(sha)).unwrap(),
            humanize(HashInput::GitLong(sha)).unwrap(),
        );
    }

    #[test]
    fn custom_separator() {
        let out = humanize_with(
            HashInput::Md5("0123456789abcdef0123456789abcdef"),
            HumanizeOptions { separator: " " },
        )
        .unwrap();
        assert!(!out.contains('-'));
        assert_eq!(out.matches(' ').count(), 11);
    }

    #[test]
    fn humanize_bytes_matches_typed_input_on_same_bytes() {
        let bytes = [
            0x55, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4, 0xa7, 0x16, 0x44, 0x66, 0x55, 0x44,
            0x00, 0x00,
        ];
        let from_bytes = humanize_bytes(&bytes).unwrap();
        let from_uuid = humanize(HashInput::Uuid("550e8400-e29b-41d4-a716-446655440000")).unwrap();
        assert_eq!(from_bytes, from_uuid);
    }

    #[test]
    fn humanize_bytes_rejects_empty() {
        assert_eq!(humanize_bytes(&[]), Err(HumanhashError::EmptyInput));
    }
}
